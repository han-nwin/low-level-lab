#include <chrono>
#include <exception>
#include <future>
#include <iostream>
#include <stdexcept>
#include <string>
#include <thread>
#include <utility>
#include <vector>

using namespace std::chrono_literals;

// NOTE:
// clang++ -std=c++20 -Wall -Wextra -Wpedantic -pthread -g main.cpp -o main

// =============================================================================
// 1. A normal function that performs slow work
// =============================================================================

int calculate_square(int value) {
    std::cout << "Calculating " << value << " on thread "
              << std::this_thread::get_id() << '\n';

    std::this_thread::sleep_for(500ms);

    return value * value;
}

// =============================================================================
// 2. Exceptions are transported through std::future
// =============================================================================

int risky_calculation(bool should_fail) {
    std::this_thread::sleep_for(200ms);

    if (should_fail) {
        throw std::runtime_error{"calculation failed"};
    }

    return 42;
}

// =============================================================================
// 3. A function taking ownership of a value
// =============================================================================

std::size_t process_text(std::string text) {
    std::this_thread::sleep_for(200ms);
    return text.size();
}

int main() {
    std::cout << "Main thread: " << std::this_thread::get_id() << "\n\n";

    // =========================================================================
    // 1. Launch asynchronous work and obtain its result
    // =========================================================================

    {
        std::cout << "=== 1. std::async + std::future ===\n";

        std::future<int> result =
            std::async(std::launch::async, calculate_square, 7);

        // The async task and main can now run concurrently.
        std::cout << "Main continues doing other work\n";
        std::this_thread::sleep_for(100ms);

        // get() waits if the result is not ready and then returns it.
        int value = result.get();

        std::cout << "Result: " << value << "\n\n";
    }

    // =========================================================================
    // 2. Polling without blocking
    // =========================================================================

    {
        std::cout << "=== 2. Check whether a result is ready ===\n";

        std::future<int> result =
            std::async(std::launch::async, calculate_square, 8);

        while (result.wait_for(100ms) != std::future_status::ready) {
            std::cout << "Main is still waiting...\n";
        }

        std::cout << "Result: " << result.get() << "\n\n";
    }

    // =========================================================================
    // 3. Run several tasks concurrently
    // =========================================================================

    {
        std::cout << "=== 3. Multiple async tasks ===\n";

        std::vector<std::future<int>> results;

        for (int number : {2, 3, 4, 5}) {
            results.push_back(
                std::async(std::launch::async, calculate_square, number));
        }

        // Each get() waits for that corresponding task.
        for (std::future<int> &result : results) {
            std::cout << "Received: " << result.get() << '\n';
        }

        std::cout << '\n';
    }

    // =========================================================================
    // 4. Move ownership into an async task
    // =========================================================================

    {
        std::cout << "=== 4. Move ownership into async work ===\n";

        std::string text{"C++ asynchronous programming"};

        std::future<std::size_t> length =
            std::async(std::launch::async, process_text, std::move(text));

        // text still exists in C++, but is moved-from.
        // Do not depend on its old contents.
        std::cout << "Processed length: " << length.get() << "\n\n";
    }

    // =========================================================================
    // 5. Exceptions travel through the future
    // =========================================================================

    {
        std::cout << "=== 5. Exception propagation ===\n";

        std::future<int> result =
            std::async(std::launch::async, risky_calculation, true);

        try {
            // The worker's exception is rethrown here.
            std::cout << "Result: " << result.get() << '\n';
        } catch (const std::exception &error) {
            std::cout << "Async error: " << error.what() << '\n';
        }

        std::cout << '\n';
    }

    // =========================================================================
    // 6. Deferred execution
    // =========================================================================

    {
        std::cout << "=== 6. Deferred execution ===\n";

        std::future<int> result =
            std::async(std::launch::deferred, calculate_square, 9);

        // The function has not run yet.
        // It runs synchronously on the thread that calls get().
        std::cout << "Task created but not started\n";

        std::cout << "Result: " << result.get() << "\n\n";
    }

    // =========================================================================
    // 7. Promise and future: manually provide a result
    // =========================================================================

    {
        std::cout << "=== 7. std::promise + std::future ===\n";

        std::promise<std::string> sender;
        std::future<std::string> receiver = sender.get_future();

        std::jthread worker{[sender = std::move(sender)]() mutable {
            std::this_thread::sleep_for(200ms);
            sender.set_value("message from worker");
        }};

        // Blocks until set_value() is called.
        std::cout << "Received: " << receiver.get() << "\n\n";
    }

    // =========================================================================
    // 8. shared_future: allow several consumers
    // =========================================================================

    {
        std::cout << "=== 8. std::shared_future ===\n";

        std::shared_future<int> shared_result =
            std::async(std::launch::async, calculate_square, 10).share();

        std::jthread reader_one{[shared_result] {
            std::cout << "Reader one: " << shared_result.get() << '\n';
        }};

        std::jthread reader_two{[shared_result] {
            std::cout << "Reader two: " << shared_result.get() << '\n';
        }};

        // shared_future can be copied, and get() can be called repeatedly.
        // jthreads automatically join when leaving scope.
    }

    std::cout << "\nAll async examples finished\n";
}
