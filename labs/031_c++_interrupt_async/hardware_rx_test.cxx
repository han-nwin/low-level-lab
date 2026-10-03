#include "hardware.hpp"
#include <iostream>
#include <stdexcept>

namespace {
void check(bool ok, const char *message) {
    if (!ok) throw std::runtime_error(message);
}
template <typename F> void rejects(F action) {
    bool rejected = false;
    try { action(); } catch (const std::exception &) { rejected = true; }
    check(rejected, "expected invalid hardware operation to throw");
}
struct Bench {
    asio::io_context io;
    Hardware hw{io.get_executor()};
    Bench() { hw.write(Reg::CONTROL_0, control0::ENABLE); }
    void drain() { io.restart(); io.run(); }
    void arm(unsigned offset, unsigned size) {
        hw.write(Reg::RX_OFFSET, offset);
        hw.write(Reg::RX_BYTE_NUMBER, size);
        hw.write(Reg::RX_COMMAND, rxcommand::START);
    }
    unsigned byte(unsigned offset) {
        return (hw.read(Hardware::rxMemWord(offset / 4)) >> (8 * (offset % 4))) & 0xffu;
    }
};

void memoryEngine() {
    Bench b;
    auto &hw = b.hw;
    hw.write(Reg::RX_TIMEOUT_MS, 0);
    hw.write(Reg::RX_OFFSET, 3);
    hw.write(Reg::RX_BYTE_NUMBER, 100);
    check(!hw.injectRx('!'), "configuration alone must not start RX");
    hw.write(Reg::RX_COMMAND, 0);
    check(!hw.injectRx('!'), "zero command must not start RX");
    hw.write(Reg::RX_COMMAND, rxcommand::START);
    check(hw.read(Reg::RX_COMMAND) == 0, "command register must read zero");
    rejects([&] { hw.write(Reg::RX_OFFSET, 0); });
    rejects([&] { hw.write(Reg::RX_BYTE_NUMBER, 1); });
    rejects([&] { hw.write(Reg::RX_COMMAND, rxcommand::START); });
    for (unsigned i = 0; i < 100; ++i) {
        check(hw.injectRx(static_cast<char>(i)), "RX byte rejected");
        check(hw.read(Reg::RX_BYTE_COUNT) == i, "arrival counted before memory commit");
        check(!(hw.read(Reg::CONTROL_0) & control0::RX_EMPTY), "staging FIFO should contain byte");
        b.drain();
        check(hw.read(Reg::RX_BYTE_COUNT) == i + 1, "memory commit count incorrect");
        check(b.byte(3 + i) == i, "RX offset/byte packing incorrect");
        check(hw.read(Reg::IRQ_STATUS) & irq::RX_EMPTY, "missing FIFO drain interrupt");
        check(bool(hw.read(Reg::CONTROL_0) & control0::RX_BUSY) == (i != 99), "incorrect RX_BUSY");
        hw.write(Reg::IRQ_CLEAR, irq::RX_EMPTY);
    }
    check(b.byte(2) == 0 && b.byte(103) == 0, "RX write crossed buffer boundary");
    check(!hw.injectRx('!'), "completed transfer must reject late bytes");
    check(hw.read(Hardware::rxMemWord(1)) == 0x04030201u, "RX words are not low-byte-first");
    check(hw.read(Hardware::rxMemWord(1)) == 0x04030201u, "memory read must not consume data");
    rejects([&] { hw.write(Hardware::rxMemWord(0), 1); });
    rejects([&] { hw.write(Reg::RX_BYTE_COUNT, 0); });
    rejects([&] { hw.read(static_cast<Reg>(0x0c)); }); // Old FIFO pop register removed.
    rejects([&] { Hardware::rxMemWord(Hardware::RX_MEM_WORDS); });
    rejects([&] { hw.write(Reg::RX_OFFSET, 257); });
    rejects([&] { hw.write(Reg::RX_BYTE_NUMBER, 257); });
    hw.write(Reg::RX_OFFSET, 255);
    hw.write(Reg::RX_BYTE_NUMBER, 2);
    rejects([&] { hw.write(Reg::RX_COMMAND, rxcommand::START); });
    b.arm(255, 1);
    check(hw.injectRx(static_cast<char>(0xff)), "last memory byte rejected");
    b.drain();
    check(b.byte(255) == 0xff, "last memory byte incorrect");
    b.arm(256, 0);
    check(hw.read(Reg::RX_BYTE_COUNT) == 0, "zero START must reset count");
    check(!(hw.read(Reg::CONTROL_0) & control0::RX_BUSY), "zero START must complete immediately");
    check(hw.read(Reg::IRQ_STATUS) & irq::RX_EMPTY, "zero START missing notification");
    b.arm(0, Hardware::RX_MEM_BYTES);
    for (unsigned i = 0; i < Hardware::RX_MEM_BYTES; ++i) {
        check(hw.injectRx(static_cast<char>(i)), "full memory transfer rejected byte");
        if (i % Hardware::FIFO_CAPACITY == Hardware::FIFO_CAPACITY - 1) b.drain();
    }
    check(hw.read(Reg::RX_BYTE_COUNT) == 256, "full memory count incorrect");
    for (unsigned i = 0; i < 256; ++i) check(b.byte(i) == i, "full memory payload incorrect");
}

void timeoutAndPause() {
    Bench b;
    auto &hw = b.hw;
    hw.write(Reg::RX_TIMEOUT_MS, 1);
    b.arm(2, 100);
    b.drain();
    check(hw.read(Reg::CONTROL_0) & control0::RX_BUSY, "timeout must not start before first byte");
    check(hw.injectRx('A') && hw.injectRx('B'), "partial bytes rejected");
    b.drain();
    check(hw.read(Reg::RX_BYTE_COUNT) == 2, "partial timeout lost staged data");
    check(hw.read(Reg::IRQ_STATUS) & irq::RX_TIMEOUT, "missing partial timeout");
    check(!(hw.read(Reg::CONTROL_0) & control0::RX_BUSY), "timeout must stop capture");
    check(!hw.injectRx('C'), "timeout snapshot must reject late arrivals");
    hw.write(Reg::IRQ_CLEAR, irq::RX_TIMEOUT);
    b.drain();
    check(b.byte(2) == 'A' && b.byte(3) == 'B', "timeout result not stable");
    b.arm(0, 2);
    check(hw.read(Reg::IRQ_STATUS) == 0, "START must clear stale RX notifications");
    check(hw.injectRx('x'), "pause setup failed");
    hw.write(Reg::CONTROL_0, 0); // Pause before the queued FIFO drain.
    b.drain();
    check(hw.read(Reg::RX_BYTE_COUNT) == 0, "disabled engine committed data");
    check(!hw.injectRx('y'), "disabled receiver accepted data");
    rejects([&] { hw.write(Reg::RX_OFFSET, 1); });
    hw.write(Reg::CONTROL_0, control0::ENABLE);
    check(hw.injectRx('y'), "resumed receiver rejected data");
    b.drain();
    check(hw.read(Reg::RX_BYTE_COUNT) == 2 && b.byte(0) == 'x' && b.byte(1) == 'y', "resume lost data");
    check(!(hw.read(Reg::IRQ_STATUS) & irq::RX_TIMEOUT), "full completion must cancel timeout");
    b.arm(0, 5);
    check(hw.injectRx('z'), "reset setup failed");
    hw.write(Reg::CONTROL_0, control0::UART_RESET);
    b.drain();
    check(hw.read(Reg::RX_BYTE_COUNT) == 0 && hw.read(Reg::RX_BYTE_NUMBER) == 0 &&
          hw.read(Reg::RX_OFFSET) == 0 && hw.read(Reg::IRQ_STATUS) == 0, "reset left RX state");
    for (unsigned i = 0; i < Hardware::RX_MEM_WORDS; ++i)
        check(hw.read(Hardware::rxMemWord(i)) == 0, "reset left RX memory");
    rejects([&] { hw.write(Reg::RX_COMMAND, rxcommand::START); });
}

void selectiveClear() {
    Bench b;
    auto &hw = b.hw;
    hw.write(Reg::RX_TIMEOUT_MS, 1);
    b.arm(0, 32);
    for (unsigned i = 0; i < 16; ++i) check(hw.injectRx('r'), "FIFO fill failed");
    check(!hw.injectRx('!'), "FIFO overrun should reject byte");
    check(hw.read(Reg::CONTROL_0) & control0::RX_FULL, "missing live RX_FULL");
    check(hw.injectRxErrors(irq::FRAME_ERROR), "framing injection failed");
    rejects([&] { hw.injectRxErrors(irq::TX_EMPTY); });
    hw.write(Reg::TX_BYTE_NUMBER, 16);
    hw.write(Reg::TX_COMMAND, txcommand::START);
    b.drain();
    check(hw.read(Reg::IRQ_STATUS) == irq::ALL, "all seven IRQ sources should be latched");
    const auto config = hw.read(Reg::CONTROL_1);
    hw.write(Reg::IRQ_CLEAR, 0);
    hw.write(Reg::IRQ_CLEAR, ~irq::ALL);
    hw.write(Reg::CONTROL_0, control0::ENABLE | (1u << 4));
    check(hw.read(Reg::IRQ_STATUS) == irq::ALL, "zero/reserved bits changed latches");
    auto pending = irq::ALL;
    for (unsigned bit = 0; bit < 7; ++bit) {
        hw.write(Reg::IRQ_CLEAR, 1u << bit);
        pending &= ~(1u << bit);
        check(hw.read(Reg::IRQ_STATUS) == pending, "clear touched another source");
        check(hw.read(Reg::IRQ_CLEAR) == 0, "CLEAR must read zero");
        check(hw.read(Reg::CONTROL_1) == config && hw.read(Reg::CONTROL_0) & control0::ENABLE,
              "clear changed configuration/enable");
        check(hw.read(Reg::RX_BYTE_COUNT) == 16 && hw.read(Reg::TX_BYTE_COUNT) == 16,
              "clear changed counts");
        check(b.byte(0) == 'r' && b.byte(15) == 'r', "clear changed RX memory");
    }
    unsigned calls = 0;
    hw.registerInterruptHandler([&] {
        ++calls;
        check(hw.read(Reg::IRQ_STATUS) == irq::FRAME_ERROR, "queued IRQ lost source isolation");
        hw.write(Reg::IRQ_CLEAR, irq::FRAME_ERROR);
    });
    hw.write(Reg::CONTROL_1, config | ((irq::RX_ERROR | irq::FRAME_ERROR) << control1::IRQ_SHIFT));
    check(hw.injectRxErrors(irq::RX_ERROR | irq::FRAME_ERROR), "error injection failed");
    hw.write(Reg::IRQ_CLEAR, irq::RX_ERROR);
    b.drain();
    check(calls == 1, "remaining queued source not delivered");
    check(hw.injectRxErrors(irq::FRAME_ERROR), "error injection failed");
    hw.write(Reg::IRQ_CLEAR, irq::ALL);
    b.drain();
    check(calls == 1, "cleared queued source was delivered");
    hw.write(Reg::CONTROL_0, 0);
    check(!hw.injectRxErrors(irq::RX_ERROR), "disabled error injection accepted");
    hw.write(Reg::IRQ_CLEAR, irq::ALL);
    check(!(hw.read(Reg::CONTROL_0) & control0::ENABLE), "clear enabled disabled UART");
}
} // namespace

int main() {
    try {
        memoryEngine();
        timeoutAndPause();
        selectiveClear();
        std::cout << "RX memory engine and selective IRQ clear tests PASS\n";
    } catch (const std::exception &e) {
        std::cerr << "FAIL: " << e.what() << '\n';
        return 1;
    }
}
