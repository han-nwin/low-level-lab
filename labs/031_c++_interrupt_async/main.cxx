#include "uart.hpp"
#include <array>
#include <chrono>
#include <exception>
#include <iostream>
#include <string>
#include <string_view>

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

asio::awaitable<void> exercise(Uart &uart, Hardware &hw, bool receive) {
    if (receive) {
        std::cout << "[app] Waiting for a byte from the simulated peer...\n";

        const char byte = co_await uart.receive();
        if (byte != 'R' || !(hw.read(Reg::CONTROL_0) & control0::RX_EMPTY))
            throw std::runtime_error("Expected receive() to consume byte 'R'");
        std::cout << "[app] PASS: received 'R' through the interrupt bridge\n";

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
    asio::steady_timer watchdog(io, 5s), peer(io, 100ms);
    int result = 1;
    watchdog.async_wait([&](boost::system::error_code ec) {
        if (ec)
            return;
        std::cerr << "[app] FAIL: timed out. Check ENABLE, mask, IRQ acknowledge, "
                     "and whether you called the saved completion handler.\n";
        io.stop();
    });
    if (receive)
        peer.async_wait([&](boost::system::error_code ec) {
            if (!ec) {
                std::cout << "[peer] Injecting 'R'\n";
                if (!hw.injectRx('R'))
                    std::cerr << "[peer] Byte rejected: UART disabled or RX full\n";
            }
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
        peer.cancel();
        work.reset();
        // On success let the hardware's last wire timer finish before exit.
        if (error) io.stop();
    });
    io.run();

    const auto expectedWire = "hello!" + std::string(100, 'x') + std::string("\x01\x00\xff", 3);
    if (result == 0 && !receive && hw.transmitted() != expectedWire) {
        std::cerr << "[app] FAIL: wire output lost or duplicated bytes\n";
        result = 1;
    }

    // Disconnect the vector before the application-owned Uart is destroyed.
    hw.write(Reg::CONTROL_1, hw.read(Reg::CONTROL_1) & ~control1::IRQ_MASK);
    hw.registerInterruptHandler({});
    uartIrqTarget = nullptr;
    return result;
}
