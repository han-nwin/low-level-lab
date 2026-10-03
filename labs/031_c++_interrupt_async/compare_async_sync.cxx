#include "uart.hpp"
#include "uart_no_async.hpp"

#include <algorithm>
#include <array>
#include <chrono>
#include <iostream>
#include <thread>

using Clock = std::chrono::steady_clock;
using namespace std::chrono_literals;

namespace {
constexpr std::size_t kBytes = 20; // Longer than the 16-byte FIFO.
constexpr auto kMiddleTaskDelay = 50ms;
constexpr auto kMiddleTaskDuration = 50ms;

struct Timings {
    long long middleStartMs;
    long long middleFinishMs;
    long long sendWaitMs;
    long long totalMs;
};

long long milliseconds(Clock::duration duration) {
    return std::chrono::duration_cast<std::chrono::milliseconds>(duration).count();
}

void prepare(Hardware &hw) {
    hw.write(Reg::CONTROL_0, control0::UART_RESET);
    hw.write(Reg::CONTROL_0, control0::ENABLE);
}

Timings asyncCase() {
    asio::io_context io;
    Hardware hw(io.get_executor());
    prepare(hw);
    Uart uart(io.get_executor(), hw);
    hw.registerInterruptHandler([&uart] { uart.handleInterrupt(); });

    std::array<std::uint8_t, kBytes> payload{};
    payload.fill('A');
    asio::steady_timer middleTask(io, kMiddleTaskDelay);
    asio::steady_timer middleWork(io);
    const auto start = Clock::now();
    auto middleStarted = start;
    auto middleFinished = start;
    auto sendFinished = start;
    bool sendDone = false;
    bool middleDone = false;
    middleTask.async_wait([&](boost::system::error_code ec) {
        if (!ec) {
            middleStarted = Clock::now();
            std::cout << "async: middle task started while send was suspended\n";
            middleWork.expires_after(kMiddleTaskDuration);
            middleWork.async_wait([&](boost::system::error_code workEc) {
                if (!workEc) {
                    middleFinished = Clock::now();
                    middleDone = true;
                }
            });
        }
    });
    asio::co_spawn(io, uart.send(payload), [&](std::exception_ptr error) {
        if (error) std::rethrow_exception(error);
        sendFinished = Clock::now();
        sendDone = true;
    });
    io.run();

    if (!sendDone || !middleDone) throw std::runtime_error("async work did not complete");
    const auto totalFinished = std::max(middleFinished, sendFinished);
    Timings result{milliseconds(middleStarted - start), milliseconds(middleFinished - start),
                   milliseconds(sendFinished - start), milliseconds(totalFinished - start)};
    std::cout << "async: middle task ran from " << result.middleStartMs << " to "
              << result.middleFinishMs << " ms; send wait " << result.sendWaitMs << " ms\n";
    return result;
}

Timings syncCase() {
    // This io_context represents the hardware/interrupt side. It keeps running
    // while the application thread blocks inside UartNoAsync::send().
    asio::io_context hardwareIo;
    Hardware hw(hardwareIo.get_executor());
    prepare(hw);
    UartNoAsync uart(hardwareIo.get_executor(), hw);
    auto hardwareWork = asio::make_work_guard(hardwareIo);
    std::thread hardwareThread([&hardwareIo] { hardwareIo.run(); });

    std::array<std::uint8_t, kBytes> payload{};
    payload.fill('A');

    // This timer belongs to the blocked caller's event loop. Its callback can
    // only run after send() returns, so the scheduled middle task is delayed.
    asio::io_context appIo;
    asio::steady_timer middleTask(appIo, kMiddleTaskDelay);
    asio::steady_timer middleWork(appIo);
    const auto start = Clock::now();
    auto middleStarted = start;
    auto middleFinished = start;
    middleTask.async_wait([&](boost::system::error_code ec) {
        if (!ec) {
            middleStarted = Clock::now();
            middleWork.expires_after(kMiddleTaskDuration);
            middleWork.async_wait([&](boost::system::error_code workEc) {
                if (!workEc) middleFinished = Clock::now();
            });
        }
    });

    uart.send(payload); // Blocks this application thread until TX completion IRQ.
    const auto sendFinished = Clock::now();
    appIo.run();

    const auto totalFinished = std::max(middleFinished, sendFinished);
    Timings result{milliseconds(middleStarted - start), milliseconds(middleFinished - start),
                   milliseconds(sendFinished - start), milliseconds(totalFinished - start)};
    std::cout << "sync:   middle task ran from " << result.middleStartMs << " to "
              << result.middleFinishMs << " ms; send wait " << result.sendWaitMs << " ms\n";

    hardwareIo.stop();
    hardwareWork.reset();
    hardwareThread.join();
    return result;
}
} // namespace

int main() {
    std::cout << "20-byte simulated send; UART byte timing is 10 ms\n";
    const auto async = asyncCase();
    const auto sync = syncCase();
    std::cout << "\nSummary (milliseconds)\n"
              << "  async: middle task " << async.middleStartMs << "-" << async.middleFinishMs
              << ", send wait " << async.sendWaitMs << ", total " << async.totalMs << '\n'
              << "  sync:  middle task " << sync.middleStartMs << "-" << sync.middleFinishMs
              << ", send wait " << sync.sendWaitMs << ", total " << sync.totalMs << '\n'
              << "  middle task is scheduled at 50 ms and takes 50 ms asynchronously.\n"
              << "  Async overlaps that work with the send; blocking send runs it afterward.\n";
}
