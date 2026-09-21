#include <boost/asio.hpp>
#include <boost/asio/deferred.hpp>
#include <boost/asio/experimental/coro.hpp>
#include <utility>

#include <chrono>
#include <exception>
#include <future>
#include <iostream>
#include <string>
#include <thread>

namespace asio = boost::asio;
using namespace std::chrono_literals;

// 1. SYNC: caller waits for the entire startup.
std::string startup_sync() {
    std::this_thread::sleep_for(500ms); // Simulate powering on.
    std::this_thread::sleep_for(500ms); // Simulate calibration.

    return "Ready";
}

// 2. AWAITABLE: one final result, without blocking during waits.
asio::awaitable<std::string> startup_awaitable() {
    auto ex = co_await asio::this_coro::executor;
    asio::steady_timer timer(ex);

    timer.expires_after(500ms);
    co_await timer.async_wait(asio::use_awaitable); // Power on.

    timer.expires_after(500ms);
    co_await timer.async_wait(asio::use_awaitable); // Calibrate.

    co_return "Ready";
}

// 3. CORO: report progress before the entire startup finishes.
asio::experimental::coro<std::string> startup_coro(asio::any_io_executor ex) {
    asio::steady_timer timer(ex);

    timer.expires_after(500ms);
    co_await timer.async_wait(asio::deferred);
    co_yield "Powered on"; // Pause until caller resumes us.

    timer.expires_after(500ms);
    co_await timer.async_wait(asio::deferred);
    co_yield "Calibrated"; // Pause again.

    co_yield "Ready"; // Still pauses; completion needs another resume.
    co_return;
}

// Caller for both asynchronous versions.
asio::awaitable<void> demo() {
    std::cout << "\n--- awaitable ---" << std::endl;

    auto result = co_await startup_awaitable();
    std::cout << result << std::endl; // One result after both waits.

    std::cout << "\n--- coro ---" << std::endl;

    auto ex = co_await asio::this_coro::executor;
    auto startup = startup_coro(ex); // Create; body hasn't run yet.

    // Each resume produces one status, or empty optional at completion.
    while (auto status = co_await startup.async_resume(asio::use_awaitable)) {
        std::cout << *status << std::endl;

        // startup is paused here.
        // The next iteration resumes it after its previous co_yield.
    }

    std::cout << "Startup coroutine finished." << std::endl;
}

int main() {
    try {
        std::cout << "--- sync ---" << std::endl;
        std::cout << startup_sync() << std::endl;

        asio::io_context io;

        // Schedule demo; the future lets main retrieve any exception.
        // create executor then spawn a thread
        // auto ex = io.get_executor(); // Get a handle for scheduling work on io.
        // asio::co_spawn(ex, deno(), asio::use_future);
        // SHORT HAND
        auto done = asio::co_spawn(io, demo(), asio::use_future);

        io.run();   // Drive coroutine execution and timer completions.
        done.get(); // Rethrow any exception from demo.
    } catch (const std::exception &error) {
        std::cerr << "Error: " << error.what() << '\n';
        return 1;
    }
}
