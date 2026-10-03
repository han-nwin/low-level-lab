#include "uart.hpp"
#include <algorithm>
#include <iostream>
#include <ranges>
#include <stdexcept>

// NOTE:
// UART wire
//    ↓
// hardware RX memory          ← armRxChunk() manages this
//    ↓
// software rxRing_            ← handleRxInterrupt() fills this
//    ↓
// application buffer          ← read() drains this
Uart::Uart(asio::any_io_executor executor, Hardware &hw) : hw_(hw), executor_(executor) {
    // Main owns interrupt-vector wiring.
    hw_.write(Reg::CONTROL_0, control0::UART_RESET);

    hw_.write(Reg::CONTROL_0, control0::ENABLE);

    // Background RX must start even if the application never calls read().
    armRxChunk();
}

// Destructor, reset control register
Uart::~Uart() {
    const auto config = hw_.read(Reg::CONTROL_1);
    hw_.write(Reg::CONTROL_1, config & ~control1::IRQ_MASK);
}

asio::awaitable<void> Uart::send(std::span<const std::uint8_t> bytes) {
    // Stage bytes, start TX, then wait for the final FIFO drain (not wire idle).
    // if bytes sending more than memory capacity, throw
    if (bytes.size() > Hardware::TX_MEM_BYTES) {
        throw std::length_error("bytes are too big, limit to 64*4 bytes");
    }

    // 1. Set offset to 0
    hw_.write(Reg::TX_OFFSET, 0);

    // 2. Set num of bytes we about to send
    hw_.write(Reg::TX_BYTE_NUMBER, static_cast<std::uint32_t>(bytes.size()));

    // 3. Write to tx mem
    // TX MEM has 64 words, each holding four bytes:
    // bits [31:24] [23:16] [15:8] [7:0]
    //      byte 3  byte 2  byte 1 byte 0 (first byte sent)
    //```
    // Input bytes (hex): 11 22 33 44 55 66 77 88
    //
    // TX_MEM[0] = 0x44332211
    // TX_MEM[1] = 0x88776655
    // ```
    // The simulator extracts the **lowest byte first** from each word:
    //
    // ```
    // 0x44332211 → 11 → 22 → 33 → 44
    // 0x88776655 → 55 → 66 → 77 → 88
    // ```
    // Loop through bytes, every 4 words collaps to a uint32 as little endian
    uint32_t word = 0;
    for (std::size_t idx : std::views::iota(size_t{0}, bytes.size())) {
        const size_t bytePos = idx % 4; // byte position within this word

        // little endian
        word |= static_cast<uint32_t>(bytes[idx]) << (8 * bytePos);

        // write if it's the last position of the word, or end of bytes array
        if (bytePos == 3 || idx + 1 == bytes.size()) {
            hw_.getTxMemory(idx / 4).setTxMem(word);
            word = 0;
        }
    }

    // 4. trigger the send
    hw_.write(Reg::TX_COMMAND, txcommand::START);

    std::cout << "Tx send command triggered! Suspend until interrupt + conditions are met" << std::endl;
    // Hardware start sending here .... //
    // suspend..
    co_await asyncTxWait(asio::use_awaitable); // use_awaitable is a completion token

    std::cout << "Harware interrupt, conditions are met. ALL Bytes sent" << std::endl;

    co_return;
}

asio::awaitable<std::size_t> Uart::readSome(std::span<uint8_t> buffer) {
    if (buffer.empty())
        co_return 0;

    if (readPending_)
        throw std::logic_error("A UART read is already pending");

    // Keep ownership through the posted wakeup until these bytes are consumed.
    readPending_ = true;

    // if asyncRxWait throw make sure we reset readPending_ to false
    try {
        if (rxSize_ == 0)
            co_await asyncRxWait(asio::use_awaitable);

        // read upto requested size or whatever available
        const auto count = std::min(buffer.size(), rxSize_);
        for (size_t i = 0; i < count; ++i) {
            buffer[i] = rxRing_[rxHead_];
            rxHead_ = (rxHead_ + 1) % RX_RING_CAPACITY;
        }

        rxSize_ -= count;
        readPending_ = false;

        co_return count;
    } catch (...) {
        readPending_ = false;
        throw;
    }
}

asio::awaitable<void> Uart::readExact(std::span<uint8_t> buffer) {
    if (buffer.empty())
        co_return;

    if (readPending_)
        throw std::logic_error("A UART read is already pending");

    readPending_ = true;

    try {
        std::size_t total = 0;

        while (total < buffer.size()) {

            // Nothing buffered yet -> sleep until RX puts data in the ring.
            if (rxSize_ == 0)
                co_await asyncRxWait(asio::use_awaitable);

            // Take as much as we currently have, but no more than we still need.
            const auto count = std::min(buffer.size() - total, rxSize_);

            for (size_t i = 0; i < count; ++i) {
                buffer[total + i] = rxRing_[rxHead_];
                rxHead_ = (rxHead_ + 1) % RX_RING_CAPACITY;
            }

            rxSize_ -= count;
            total += count;
        }

        readPending_ = false;
        co_return;
    } catch (...) {
        readPending_ = false;
        throw;
    }
}

void Uart::armRxChunk() {
    hw_.write(Reg::RX_OFFSET, 0);
    hw_.write(Reg::RX_BYTE_NUMBER, RX_CHUNK_BYTES);
    hw_.write(Reg::RX_COMMAND, rxcommand::START);

    // Enable rx empty and rx timeout interrupt
    const auto config = hw_.read(Reg::CONTROL_1);
    const auto mask = control1::irqMask(config) | irq::RX_EMPTY | irq::RX_TIMEOUT;
    hw_.write(Reg::CONTROL_1, (config & ~control1::IRQ_MASK) | (mask << control1::IRQ_SHIFT));
}

void Uart::handleRxInterrupt(std::uint32_t active) {
    const auto count = hw_.read(Reg::RX_BYTE_COUNT);
    const bool stopped = !(hw_.read(Reg::CONTROL_0) & control0::RX_BUSY);

    // complete means the FIFO is empty and the RX byte count matches the requested length
    const bool complete = stopped && ((active & irq::RX_TIMEOUT) || count == hw_.read(Reg::RX_BYTE_NUMBER));
    if (!complete) {
        // FIFO-empty can be an intermediate drain; do not publish bytes twice.
        hw_.write(Reg::IRQ_CLEAR, active & irq::RX_EMPTY);
        return;
    }

    // Drain the Rx Mem to ring buffer
    for (std::size_t i = 0; i < count; ++i) {
        const auto word = hw_.read(Hardware::rxMemWord(i / 4));
        const auto byte = static_cast<std::uint8_t>(word >> (8 * (i % 4)));

        // If full, discard the oldest unread byte to make room
        // for the newest incoming byte.
        if (rxSize_ == RX_RING_CAPACITY) {
            rxHead_ = (rxHead_ + 1) % RX_RING_CAPACITY;
            --rxSize_;
            ++rxDropped_;
        }

        // Append newest byte at the logical end of the ring.
        rxRing_[(rxHead_ + rxSize_) % RX_RING_CAPACITY] = byte;
        ++rxSize_;
    }

    // Intermediate Clear the interrupt
    hw_.write(Reg::IRQ_CLEAR, irq::RX_EMPTY | irq::RX_TIMEOUT);

    // Resume capturing incoming bytes to rx mem before posting the application wakeup.
    armRxChunk();

    //
    if (rxSize_ > 0)
        tryCompleteRxWait({});
}

void Uart::handleInterrupt() {
    // Latched sources select the branch; live status/counts decide completion.
    const auto mask = control1::irqMask(hw_.read(Reg::CONTROL_1));
    const auto active = hw_.read(Reg::IRQ_STATUS) & mask;

    // TX Empty
    if (txHandler_ && (active & irq::TX_EMPTY)) {

        // Intermediate FIFO-empty: acknowledge it, doesn't mean conditions are met
        hw_.write(Reg::IRQ_CLEAR, irq::TX_EMPTY);

        // If Tx fifo empty and conditions are met (byte count = num bytes)
        if ((hw_.read(Reg::CONTROL_0) & control0::TX_EMPTY) &&
            (hw_.read(Reg::TX_BYTE_COUNT) == hw_.read(Reg::TX_BYTE_NUMBER))) {

            completeTxWait({}); // Continue: RX may also be pending in this IRQ.
        }
    }

    // Do not return early after TX: both directions can be active together.
    if (active & (irq::RX_EMPTY | irq::RX_TIMEOUT))
        handleRxInterrupt(active);
}

void Uart::completeTxWait(boost::system::error_code ec) {
    if (!txHandler_)
        return;

    // Disable TX_EMPTY while preserving other masks and serial settings.
    const auto config = hw_.read(Reg::CONTROL_1);
    const auto mask = control1::irqMask(config) & ~irq::TX_EMPTY;
    hw_.write(Reg::CONTROL_1, (config & ~control1::IRQ_MASK) | (mask << control1::IRQ_SHIFT));

    // Final Acknowledge only TX_EMPTY
    hw_.write(Reg::IRQ_CLEAR, irq::TX_EMPTY);

    // Move the handler out before posting so it can only be completed once.
    asio::post(executor_, [handler = std::move(txHandler_), ec]() mutable { handler(ec); });
}

void Uart::tryCompleteRxWait(boost::system::error_code ec) {
    // RX data was added to the ring buffer.
    // If no handler exists, no read() is currently suspended waiting for data;
    // leave the bytes buffered for a future read().
    if (!rxHandler_)
        return;

    // Wake the pending read() on the executor.
    // Background RX remains enabled and continues collecting data.
    asio::post(executor_, [handler = std::move(rxHandler_), ec]() mutable { handler(ec); });
}
