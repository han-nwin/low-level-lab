#include "hardware.hpp"
#include <cassert>
#include <iostream>
#include <stdexcept>
#include <vector>

int main() {
    asio::io_context io;
    Hardware hw(io.get_executor());
    int interrupts = 0;
    std::uint32_t observed = 0;
    std::vector<std::uint32_t> emptyCounts;
    std::vector<std::size_t> wireCounts;
    std::vector<bool> busyAtEmpty;
    hw.registerInterruptHandler([&] {
        ++interrupts;
        const auto active = hw.read(Reg::IRQ_STATUS) & hw.read(Reg::CONTROL_1);
        observed |= active;
        if (active & irq::TX_EMPTY) {
            emptyCounts.push_back(hw.read(Reg::TX_BYTE_COUNT));
            wireCounts.push_back(hw.transmitted().size());
            busyAtEmpty.push_back((hw.read(Reg::CONTROL_0) & control0::TX_BUSY) != 0);
        }
        hw.write(Reg::IRQ_STATUS, active);
    });
    const auto drain = [&] { io.restart(); io.run(); };

    assert(hw.read(Reg::CONTROL_0) == (control0::RX_EMPTY | control0::TX_EMPTY));
    assert(hw.read(Reg::IRQ_STATUS) == 0);
    assert(!hw.injectRx('x'));
    const auto rejects = [](auto action) {
        bool rejected = false;
        try { action(); } catch (const std::exception &) { rejected = true; }
        assert(rejected);
    };
    rejects([&] { hw.write(Reg::TX_COMMAND, txcommand::START); });
    hw.write(Reg::CONTROL_0, control0::ENABLE);
    hw.write(Reg::RX_TIMEOUT_MS, 0);

    // Configure offset and length BEFORE writing payload. No step starts TX.
    hw.write(Reg::TX_OFFSET, 3);
    hw.write(Reg::TX_BYTE_NUMBER, 5);
    auto txMem = hw.getTxMemory();
    assert(txMem.size() == 64);
    txMem[0].setTxMem(0x68000000u); // h at byte offset 3
    txMem[1].setTxMem(0x6f6c6c65u); // e l l o
    assert(txMem[0].getTxMem() == 0x68000000u);
    assert(hw.getTxMemory()[1].getTxMem() == 0x6f6c6c65u);
    assert(hw.read(Hardware::txMemWord(0)) == 0x68000000u);
    assert(hw.read(Hardware::txMemWord(1)) == 0x6f6c6c65u);
    drain();
    assert(hw.transmitted().empty());
    assert(hw.read(Reg::TX_BYTE_COUNT) == 0);
    assert(hw.read(Reg::CONTROL_0) & control0::TX_EMPTY);
    assert(hw.read(Reg::IRQ_STATUS) == 0);
    hw.write(Reg::TX_COMMAND, 0); // Zero does not start TX either
    drain();
    assert(hw.transmitted().empty());

    hw.write(Reg::TX_COMMAND, txcommand::START);
    assert(hw.read(Reg::TX_COMMAND) == 0);
    assert(!(hw.read(Reg::CONTROL_0) & control0::TX_EMPTY));
    rejects([&] { hw.write(Reg::TX_COMMAND, txcommand::START); });
    rejects([&] { txMem[0].setTxMem('!'); });
    rejects([&] { hw.write(Reg::TX_OFFSET, 0); });
    rejects([&] { hw.write(Reg::TX_BYTE_NUMBER, 1); });
    // START consumes the short message into FIFO, before any wire output.
    io.restart();
    assert(io.run_one() == 1);
    assert(hw.read(Reg::TX_BYTE_COUNT) == 5);
    assert(hw.transmitted().empty()); // First byte is in the shift register
    assert(hw.read(Reg::CONTROL_0) & control0::TX_BUSY);
    drain();
    assert(hw.transmitted() == "hello");
    assert(hw.read(Reg::TX_BYTE_COUNT) == 5);
    assert(hw.read(Reg::TX_OFFSET) == 3);
    assert(interrupts == 0); // Masked events still latch
    assert(hw.read(Reg::IRQ_STATUS) & irq::TX_EMPTY);
    hw.write(Reg::IRQ_STATUS, 0);
    assert(hw.read(Reg::IRQ_STATUS) & irq::TX_EMPTY);
    hw.write(Reg::CONTROL_1, irq::TX_EMPTY);
    drain();
    assert(interrupts == 1 && observed == irq::TX_EMPTY);
    assert(hw.read(Reg::IRQ_STATUS) == 0);

    for (std::size_t i = 0; i < Hardware::FIFO_CAPACITY; ++i) assert(hw.injectRx('R'));
    assert(!hw.injectRx('!'));
    assert(hw.read(Reg::CONTROL_0) & control0::RX_FULL);
    assert((hw.read(Reg::IRQ_STATUS) & (irq::RX_READY | irq::RX_FULL)) ==
           (irq::RX_READY | irq::RX_FULL));
    hw.write(Reg::IRQ_STATUS, irq::RX_READY);
    assert(hw.read(Reg::IRQ_STATUS) == irq::RX_FULL);
    hw.write(Reg::CONTROL_0, control0::ENABLE | control0::CLEAR);
    assert(hw.read(Reg::IRQ_STATUS) == 0);
    assert(hw.read(Reg::CONTROL_0) & control0::RX_FULL); // CLEAR preserves FIFO
    for (std::size_t i = 0; i < Hardware::FIFO_CAPACITY; ++i) assert(hw.read(Reg::RX_DATA) == 'R');
    assert(hw.read(Reg::CONTROL_0) & control0::RX_EMPTY);

    hw.write(Reg::CONTROL_1, irq::RX_TIMEOUT);
    hw.write(Reg::RX_TIMEOUT_MS, 1);
    assert(hw.injectRx('T'));
    drain();
    assert(observed & irq::RX_TIMEOUT);
    assert(!(hw.read(Reg::CONTROL_0) & control0::RX_TIMEOUT));
    assert(hw.read(Reg::RX_DATA) == 'T');

    hw.write(Reg::CONTROL_1, 0);
    // Pause/resume a started transfer. Staging alone never transmits.
    hw.write(Reg::TX_OFFSET, 0);
    hw.write(Reg::TX_BYTE_NUMBER, 1);
    txMem[0].setTxMem('X');
    hw.write(Reg::TX_COMMAND, txcommand::START);
    hw.write(Reg::CONTROL_0, 0);
    drain();
    assert(hw.transmitted() == "hello");
    hw.write(Reg::CONTROL_0, control0::ENABLE);
    drain();
    assert(hw.transmitted() == "helloX");

    // Bounds, read-only count, and offset + length validation.
    rejects([&] { txMem[Hardware::TX_MEM_WORDS].setTxMem(0); });
    rejects([&] { Hardware::txMemWord(Hardware::TX_MEM_WORDS); });
    rejects([&] { hw.write(Reg::TX_BYTE_COUNT, 0); });
    rejects([&] { hw.write(Reg::TX_BYTE_NUMBER, 257); });
    rejects([&] { hw.write(Reg::TX_OFFSET, 257); });
    hw.write(Reg::TX_OFFSET, 255);
    hw.write(Reg::TX_BYTE_NUMBER, 2);
    rejects([&] { hw.write(Reg::TX_COMMAND, txcommand::START); });
    hw.write(Reg::TX_BYTE_NUMBER, 1);
    txMem[63].setTxMem(0x5a000000u);
    hw.write(Reg::TX_COMMAND, txcommand::START);
    assert(!(hw.read(Reg::IRQ_STATUS) & irq::TX_EMPTY)); // START cleared stale latch
    drain();
    assert(hw.transmitted() == "helloXZ");

    // Zero-length START resets count and completes without transmitting.
    hw.write(Reg::TX_BYTE_NUMBER, 0);
    hw.write(Reg::TX_COMMAND, txcommand::START);
    drain();
    assert(hw.read(Reg::TX_BYTE_COUNT) == 0);
    assert(hw.read(Reg::IRQ_STATUS) & irq::TX_EMPTY);
    assert(hw.transmitted() == "helloXZ");

    // Full-size transfer and progress; the final byte of the window is valid.
    hw.write(Reg::TX_OFFSET, 0);
    hw.write(Reg::TX_BYTE_NUMBER, Hardware::TX_MEM_BYTES);
    for (std::size_t i = 0; i < Hardware::TX_MEM_WORDS; ++i)
        txMem[i].setTxMem(0x61616161u);
    hw.write(Reg::TX_COMMAND, txcommand::START);
    assert(hw.read(Reg::CONTROL_0) & control0::TX_FULL);
    drain();
    assert(hw.read(Reg::TX_BYTE_COUNT) == Hardware::TX_MEM_BYTES);
    assert(hw.transmitted() == "helloXZ" + std::string(256, 'a'));

    // Reset cancels old timing, clears memory/configuration, and allows reuse.
    hw.write(Reg::TX_COMMAND, txcommand::START);
    hw.write(Reg::CONTROL_0, control0::UART_RESET);
    assert(hw.read(Reg::TX_BYTE_COUNT) == 0);
    assert(hw.read(Reg::TX_BYTE_NUMBER) == 0);
    assert(hw.read(Reg::TX_OFFSET) == 0);
    assert(hw.read(Hardware::txMemWord(0)) == 0);
    assert(hw.read(Hardware::txMemWord(63)) == 0);
    hw.write(Reg::CONTROL_0, control0::ENABLE);
    hw.write(Reg::TX_BYTE_NUMBER, 1);
    txMem[0].setTxMem('B');
    hw.write(Reg::TX_COMMAND, txcommand::START);
    drain();
    assert(hw.transmitted() == "helloXZ" + std::string(256, 'a') + "B");
    assert(hw.read(Reg::TX_BYTE_COUNT) == 1);
    assert(hw.read(Reg::CONTROL_1) == 0);
    assert(hw.read(Reg::RX_TIMEOUT_MS) == 100);
    // 100-byte transfer through a 16-byte FIFO: six intermediate empty IRQs.
    const auto wireBefore = hw.transmitted().size();
    emptyCounts.clear(); wireCounts.clear(); busyAtEmpty.clear();
    hw.write(Reg::TX_OFFSET, 0);
    hw.write(Reg::TX_BYTE_NUMBER, 100);
    for (std::size_t i = 0; i < 25; ++i) txMem[i].setTxMem(0x71717171u);
    hw.write(Reg::TX_COMMAND, txcommand::START);
    assert(hw.read(Reg::TX_BYTE_COUNT) == 16); // Memory consumed, not bytes on wire
    assert(hw.transmitted().size() == wireBefore);
    hw.write(Reg::CONTROL_1, irq::TX_EMPTY);
    drain();
    assert((emptyCounts == std::vector<std::uint32_t>{16, 32, 48, 64, 80, 96, 100}));
    for (std::size_t i = 0; i < emptyCounts.size(); ++i) {
        assert(wireCounts[i] == wireBefore + emptyCounts[i] - 1);
        assert(busyAtEmpty[i]); // Includes count == 100: last byte still shifting
    }
    assert(hw.transmitted().substr(wireBefore) == std::string(100, 'q'));
    assert(!(hw.read(Reg::CONTROL_0) & control0::TX_BUSY));
    assert(hw.read(Reg::CONTROL_0) & control0::TX_EMPTY);
    std::cout << "Hardware tests PASS\n";
}
