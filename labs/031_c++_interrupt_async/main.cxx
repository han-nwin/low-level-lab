#include "uart.hpp"
#include <algorithm>
#include <array>
#include <chrono>
#include <exception>
#include <iostream>
#include <memory>
#include <string>
#include <string_view>
#include <vector>

using namespace std::chrono_literals;

namespace {
// Application-owned target for the UART interrupt vector.
Uart *uartIrqTarget = nullptr;
} // namespace

// Simulated vector-table entry. On a board, startup/platform code would route
// the UART interrupt here. No completion-token or Asio logic belongs here.
extern "C" void UART_IRQHandler() {
    if (uartIrqTarget)
        uartIrqTarget->handleInterrupt();
}

// Test-bench wire: give hardware a turn between staging-FIFO-sized bursts.
asio::awaitable<void> injectBurst(Hardware &hw, std::vector<std::uint8_t> bytes) {
    asio::steady_timer pacing(co_await asio::this_coro::executor);
    for (std::size_t i = 0; i < bytes.size(); ++i) {
        if (!hw.injectRx(static_cast<char>(bytes[i])))
            throw std::runtime_error("RX rejected a byte: implement background capture/rearming");
        if ((i + 1) % Hardware::FIFO_CAPACITY == 0 || i + 1 == bytes.size()) {
            pacing.expires_after(1ms);
            co_await pacing.async_wait(asio::use_awaitable);
        }
    }
}

asio::awaitable<void> exercise(Uart &uart, Hardware &hw, bool receive) {
    if (receive) {
        hw.write(Reg::RX_TIMEOUT_MS, 10);
        std::vector<std::uint8_t> incoming(50);
        for (std::size_t i = 0; i < incoming.size(); ++i)
            incoming[i] = static_cast<std::uint8_t>(i);
        std::cout << "[app] Peer sends 50 bytes before the application calls read()...\n";
        co_await injectBurst(hw, incoming);
        asio::steady_timer settle(co_await asio::this_coro::executor, 20ms);
        co_await settle.async_wait(asio::use_awaitable);
        if (uart.rxBuffered() != 50)
            throw std::runtime_error("Completed hardware chunk was not queued in the ring");

        std::array<std::uint8_t, 10> first;
        if (co_await uart.readSome(first) != 10 || !std::equal(first.begin(), first.end(), incoming.begin()) ||
            uart.rxBuffered() != 40)
            throw std::runtime_error("read(10) must return 10 bytes and retain 40");
        std::array<std::uint8_t, 100> rest;
        rest.fill(0xa5);
        if (co_await uart.readSome(rest) != 40 || !std::equal(incoming.begin() + 10, incoming.end(), rest.begin()) ||
            rest[40] != 0xa5)
            throw std::runtime_error("read(100) must return the 40 available bytes");
        std::cout << "[app] PASS: 50 queued bytes consumed as 10 + 40\n";

        // Another capture proceeds during TX, without an active application read.
        auto accepted = std::make_shared<unsigned>(0);
        asio::steady_timer peer(co_await asio::this_coro::executor, 5ms);
        peer.async_wait([&hw, accepted](boost::system::error_code ec) {
            if (!ec)
                for (auto byte : {0x00, 0x80, 0xff})
                    if (hw.injectRx(static_cast<char>(byte)))
                        ++*accepted;
        });
        const std::uint8_t reply[] = {'O', 'K'};
        co_await uart.send(reply);
        std::array<std::uint8_t, 8> binary{};
        const auto count = co_await uart.readSome(binary);
        if (*accepted != 3 || count != 3 || binary[0] != 0 || binary[1] != 0x80 || binary[2] != 0xff)
            throw std::runtime_error("Background RX during TX lost bytes");
        if (!(hw.read(Reg::CONTROL_0) & control0::RX_BUSY) || uart.rxDropped() != 0)
            throw std::runtime_error("RX must remain armed after read(), without drops");
        if (co_await uart.readSome({}) != 0)
            throw std::runtime_error("Empty read must return zero");
        std::cout << "[app] PASS: background binary RX during TX; receiver still armed\n";
        co_return;
    }
    std::cout << "[app] Sending hello; wait for FIFO empty AND count == length...\n";

    const std::uint8_t hello[] = {'h', 'e', 'l', 'l', 'o'};
    co_await uart.send(hello);
    if (hw.read(Reg::TX_BYTE_COUNT) != 5 || !(hw.read(Reg::CONTROL_0) & control0::TX_EMPTY))
        throw std::runtime_error("send() returned before TX transfer completed");
    std::cout << "[app] PASS: hello consumed; final byte may still be on the wire\n";

    co_await uart.send({}); // Empty span: zero-length transfer.

    const std::uint8_t punctuation[] = {'!'};
    co_await uart.send(punctuation);
    if (hw.read(Reg::TX_BYTE_COUNT) != 1 || !(hw.read(Reg::CONTROL_0) & control0::TX_EMPTY))
        throw std::runtime_error("Repeated send() completed early");
    std::cout << "[app] PASS: empty send and repeated send\n";

    // Bigger than the FIFO: the first empty IRQ must NOT finish this send.
    std::array<std::uint8_t, 100> longer;
    longer.fill('x');
    co_await uart.send(longer);
    if (hw.read(Reg::TX_BYTE_COUNT) != 100 || !(hw.read(Reg::CONTROL_0) & control0::TX_EMPTY))
        throw std::runtime_error("100-byte send completed on an intermediate FIFO-empty IRQ");
    std::cout << "[app] PASS: 100 bytes consumed across multiple FIFO-empty IRQs\n";

    const std::uint8_t binary[] = {0x01, 0x00, 0xff};
    co_await uart.send(binary);
    if (hw.read(Reg::TX_BYTE_COUNT) != sizeof(binary) || !(hw.read(Reg::CONTROL_0) & control0::TX_EMPTY))
        throw std::runtime_error("Binary send completed early");
    std::cout << "[app] PASS: binary payload consumed\n";
}

int main(int argc, char **argv) {
    const bool receive = argc == 2 && std::string_view(argv[1]) == "--rx";
    if (argc > 2 || (argc == 2 && !receive)) {
        std::cerr << "Usage: ./uart_lab [--rx]\n";
        return 2;
    }
    asio::io_context io;
    Hardware hw(io.get_executor());
    Uart uart(io.get_executor(), hw);

    // Simulate installing UART_IRQHandler in the platform's vector table.
    // Only the desktop test bench uses this hardware callback API.
    uartIrqTarget = &uart;
    hw.registerInterruptHandler(UART_IRQHandler);

    auto work = asio::make_work_guard(io);
    asio::steady_timer watchdog(io, 5s);
    int result = 1;
    watchdog.async_wait([&](boost::system::error_code ec) {
        if (ec)
            return;
        std::cerr << "[app] FAIL: timed out. Check ENABLE, mask, IRQ acknowledge, "
                     "and whether you called the saved completion handler.\n";
        io.stop();
    });
    asio::co_spawn(io, exercise(uart, hw, receive), [&](std::exception_ptr error) {
        if (error) {
            try {
                std::rethrow_exception(error);
            } catch (const std::exception &e) {
                std::cerr << "[app] " << e.what() << '\n';
            }
        } else
            result = 0;
        watchdog.cancel();
        work.reset();
        // On success let the hardware's last wire timer finish before exit.
        if (error)
            io.stop();
    });
    io.run();

    const auto expectedWire =
        receive ? std::string("OK") : "hello!" + std::string(100, 'x') + std::string("\x01\x00\xff", 3);
    if (result == 0 && hw.transmitted() != expectedWire) {
        std::cerr << "[app] FAIL: wire output lost or duplicated bytes\n";
        result = 1;
    }

    // Disconnect the vector before the application-owned Uart is destroyed.
    hw.write(Reg::CONTROL_1, hw.read(Reg::CONTROL_1) & ~control1::IRQ_MASK);
    hw.registerInterruptHandler({});
    uartIrqTarget = nullptr;
    return result;
}
