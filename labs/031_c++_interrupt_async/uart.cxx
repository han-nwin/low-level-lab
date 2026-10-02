#include "uart.hpp"
#include <ranges>
#include <stdexcept>

Uart::Uart(asio::any_io_executor executor, Hardware &hw) : executor_(executor), hw_(hw) {
    // TODO 1: reset and enable UART. Main owns interrupt-vector wiring.

    // Reset Uart
    hw_.write(Reg::CONTROL_0, control0::UART_RESET);

    // Enable Uart
    hw_.write(Reg::CONTROL_0, control0::ENABLE);

    // Reset leaves CONTROL_1 masked. TODO 2 unmasks the event being awaited
    // after saving its completion handler.
    hw_.write(Reg::CONTROL_1, irq::ALL); // turn on all interrupt
}

Uart::~Uart() { hw_.write(Reg::CONTROL_1, 0); }

asio::awaitable<void> Uart::send(std::span<const std::uint8_t> bytes) {
    // TODO 3: enforce bytes.size() <= Hardware::TX_MEM_BYTES.
    // Set TX_OFFSET to 0, then TX_BYTE_NUMBER to bytes.size().
    // Pack up to four bytes into each 32-bit TX MEM word, then write that word.
    // Signal TX transfer: write txcommand::START to Reg::TX_COMMAND.
    // Then co_await asyncWait(irq::TX_EMPTY, asio::use_awaitable).
    // In this single-threaded simulator, IRQ callbacks are queued; asyncWait
    // must check TX_EMPTY AND TX_BYTE_COUNT == TX_BYTE_NUMBER, including on
    // the already-ready path. FIFO empty alone does not mean transfer complete.

    // if bytes sending more than memory capacity, throw
    if (bytes.size() > Hardware::TX_MEM_BYTES) {
        throw std::length_error("bytes are too big, limit to 64*4 bytes");
    }

    // Set offset to 0
    hw_.write(Reg::TX_OFFSET, 0);

    // Set num of bytes we about to send
    hw_.write(Reg::TX_BYTE_NUMBER, static_cast<std::uint32_t>(bytes.size()));

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
        word |= static_cast<uint32_t>(bytes[idx] << 8 * bytePos);

        // write if it's the last position of the word, or end of bytes array
        if (bytePos == 3 || idx + 1 == bytes.size()) {
            hw_.getTxMemory(idx / 4).setTxMem(word);
            word = 0;
        }
    }

    co_return;
}

void Uart::handleInterrupt() {
    // TODO 4a: read IRQ_STATUS and CONTROL_1. Is your awaited event active?
    // Acknowledge each active event with a write-one-to-clear write.
    // TX_EMPTY: only completeWait when TX_BYTE_COUNT == TX_BYTE_NUMBER AND
    // live TX_EMPTY is set. Otherwise keep the handler and interrupt enabled:
    // hardware will refill its FIFO and interrupt again when it drains.
    // RX_READY: complete the receive wait as before.
    (void)waitingFor_;
}

void Uart::completeWait(boost::system::error_code ec) {
    // TODO 4b: mask the awaited interrupt; clear its stale latch even on the
    // already-ready path. Move the saved handler out and clear waitingFor_.
    // Post a lambda to executor_ that invokes the moved handler with ec.
    // If no handler is pending, don't complete anything.
    (void)ec;
}

asio::awaitable<char> Uart::receive() {
    // BONUS: await RX_READY, then read RX_DATA. RX_READY means !RX_EMPTY.
    throw std::logic_error("BONUS: implement receive() in uart.cxx");
    co_return '\0';
}
