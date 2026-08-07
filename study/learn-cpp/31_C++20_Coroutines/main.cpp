#include <coroutine>
#include <exception>
#include <iostream>
#include <utility>

// C++20 coroutines are functions that can suspend and later resume. A coroutine
// uses at least one of these keywords:
// - co_yield value: produce a value, then suspend
// - co_await value: suspend until an awaitable operation is ready
// - co_return: finish the coroutine
//
// The standard library provides the low-level coroutine machinery, not a
// general generator type in C++20, so this lesson implements a small integer
// generator.
class Generator {
  public:
    struct promise_type;
    using Handle = std::coroutine_handle<promise_type>;

    struct promise_type {
        int current_value{};
        std::exception_ptr exception{};

        Generator get_return_object() {
            return Generator{Handle::from_promise(*this)};
        }

        // Suspend before executing the body, making this generator lazy.
        std::suspend_always initial_suspend() noexcept { return {}; }

        // Keep the coroutine frame alive after completion until Generator
        // destroys it.
        std::suspend_always final_suspend() noexcept { return {}; }

        std::suspend_always yield_value(int value) noexcept {
            current_value = value;
            return {}; // suspend after making the value available to the caller
        }

        void return_void() noexcept {}

        void unhandled_exception() noexcept {
            exception = std::current_exception();
        }
    };

    explicit Generator(Handle handle) : handle_{handle} {}

    // A coroutine handle owns access to one coroutine frame. This type is
    // move-only so two Generator objects cannot accidentally destroy the same
    // frame.
    Generator(const Generator &) = delete;
    Generator &operator=(const Generator &) = delete;

    Generator(Generator &&other) noexcept
        : handle_{std::exchange(other.handle_, {})} {}

    Generator &operator=(Generator &&other) noexcept {
        if (this != &other) {
            if (handle_) {
                handle_.destroy();
            }
            handle_ = std::exchange(other.handle_, {});
        }
        return *this;
    }

    ~Generator() {
        if (handle_) {
            handle_.destroy(); // release the compiler-created coroutine frame
        }
    }

    bool next() {
        if (!handle_ || handle_.done()) {
            return false;
        }

        handle_.resume();

        if (handle_.promise().exception) {
            std::rethrow_exception(handle_.promise().exception);
        }
        return !handle_.done();
    }

    [[nodiscard]] int value() const { return handle_.promise().current_value; }

  private:
    Handle handle_{};
};

Generator count_to(int maximum) {
    for (int number = 1; number <= maximum; ++number) {
        co_yield number; // save local state and return control to the caller
    }
    co_return;
}

int main() {
    Generator numbers = count_to(5);

    // Each next() resumes execution immediately after the previous co_yield.
    while (numbers.next()) {
        std::cout << numbers.value() << ' ';
    }
    std::cout << '\n';

    // Coroutines do not automatically create threads or run concurrently. They
    // are a control-flow mechanism; an async runtime or scheduler supplies
    // concurrency.
}
