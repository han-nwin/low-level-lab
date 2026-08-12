#include <chrono>
#include <condition_variable>
#include <cstdint>
#include <iostream>
#include <memory>
#include <mutex>
#include <optional>
#include <queue>
#include <stop_token>
#include <string>
#include <thread>
#include <utility>
#include <vector>

using namespace std::chrono_literals;

// NOTE:
// clang++ -std=c++20 -Wall -Wextra -Wpedantic -pthread -g main.cpp -o main

// =============================================================================
// A small Rust-style MPSC channel.
//
// Usage:
//
//     auto [sender, receiver] = make_channel<std::string>();
//
//     auto another_sender = sender; // Similar to tx.clone() in Rust
//
//     sender.send("hello");
//
//     while (auto message = receiver.receive()) {
//         std::cout << *message << '\n';
//     }
//
// MPSC means:
//
//     Multiple Producer
//     Single Consumer
//
// The channel combines:
//
//     std::queue<T>             stores messages
//     std::mutex                protects shared state
//     std::condition_variable   lets the receiver sleep and wake
//     std::shared_ptr           shares the channel state
//     std::optional<T>          represents message or end-of-stream
//
// The channel automatically closes when the last Sender is destroyed.
// =============================================================================
// NOTE:
// Type(const Type&)            // copy constructor
// Type(Type&&)                 // move constructor
// Type& operator=(const Type&) // copy assignment
// Type& operator=(Type&&)      // move assignment
// NOTE:
// = default; // compiler, generate the normal implementation
// = delete;  // compiler, forbid this operation
// noexcept   // this function promises not to throw

template <typename T> struct ChannelState {
    std::queue<T> messages;
    std::mutex mutex;
    std::condition_variable condition;
    std::size_t sender_count{0};
};

template <typename T> struct Sender {
    explicit Sender(std::shared_ptr<ChannelState<T>> state)
        : state_{std::move(state)} {
        add_sender();
    }

    // Copying a Sender creates another active producer.
    //
    // Similar to:
    //
    //     let another_sender = sender.clone();
    Sender(const Sender &other) : state_{other.state_} { add_sender(); }

    // Moving transfers this sender without creating a new sender.
    Sender(Sender &&other) noexcept : state_{std::move(other.state_)} {}

    // Copy-and-swap handles both copy assignment and move assignment.
    Sender &operator=(Sender other) noexcept {
        state_.swap(other.state_);
        return *this;
    }

    ~Sender() { remove_sender(); }

    void send(T value) const {
        {
            std::lock_guard lock{state_->mutex};
            state_->messages.push(std::move(value));
        }

        // Wake one receiver because one message became available.
        state_->condition.notify_one();
    }

  private:
    void add_sender() {
        if (!state_) {
            return;
        }

        std::lock_guard lock{state_->mutex};
        ++state_->sender_count;
    }

    void remove_sender() {
        if (!state_) {
            return;
        }

        bool was_last_sender{false};

        {
            std::lock_guard lock{state_->mutex};

            --state_->sender_count;
            was_last_sender = state_->sender_count == 0;
        }

        if (was_last_sender) {
            // The receiver might be sleeping on an empty queue.
            //
            // Wake it so it can observe that no senders remain and return
            // std::nullopt.
            state_->condition.notify_all();
        }
    }

    std::shared_ptr<ChannelState<T>> state_;
};

template <typename T> struct Receiver {
    explicit Receiver(std::shared_ptr<ChannelState<T>> state)
        : state_{std::move(state)} {}

    // This teaching channel is MPSC, so Receiver cannot be copied.
    Receiver(const Receiver &) = delete;
    Receiver &operator=(const Receiver &) = delete;

    Receiver(Receiver &&) noexcept = default;
    Receiver &operator=(Receiver &&) noexcept = default;

    std::optional<T> receive() {
        std::unique_lock lock{state_->mutex};

        // A condition variable lets this thread sleep without repeatedly
        // checking the queue and wasting CPU.
        //
        // wait() conceptually:
        //
        //     1. Checks the predicate while mutex is locked.
        //     2. If false, unlocks mutex and sleeps.
        //     3. A sender changes the shared state and calls notify_one().
        //     4. Receiver wakes and locks mutex again.
        //     5. Receiver checks the predicate again.
        //
        // We wake when:
        //
        //     - a message exists, OR
        //     - every sender has been destroyed
        //
        // The predicate is required because condition variables may wake
        // spuriously even when no notification occurred.
        state_->condition.wait(lock, [this] {
            return !state_->messages.empty() || state_->sender_count == 0;
        });

        // If the queue is empty and no senders remain, no future message can
        // ever arrive. This is end-of-stream.
        if (state_->messages.empty()) {
            return std::nullopt;
        }

        T value{std::move(state_->messages.front())};
        state_->messages.pop();

        return value;
    }

  private:
    std::shared_ptr<ChannelState<T>> state_;
};

template <typename T> auto make_channel() {
    auto state = std::make_shared<ChannelState<T>>();

    return std::pair{
        Sender<T>{state},
        Receiver<T>{std::move(state)},
    };
}

// =============================================================================
// Example functions
// =============================================================================

void basic_thread_example() {
    std::cout << "\n=== 1. Spawn and join ===\n";

    std::thread worker{[] {
        for (int i{1}; i <= 5; ++i) {
            std::cout << "Worker: " << i << '\n';
            std::this_thread::sleep_for(50ms);
        }
    }};

    for (int i{1}; i <= 3; ++i) {
        std::cout << "Main:   " << i << '\n';
        std::this_thread::sleep_for(50ms);
    }

    // Like Rust:
    //
    //     handle.join().unwrap();
    //
    // join() blocks until the worker finishes.
    worker.join();

    std::cout << "Worker has finished\n";
}

void move_capture_example() {
    std::cout << "\n=== 2. Move data into a thread ===\n";

    std::vector<int> numbers{10, 20, 30, 40};

    // Move the vector into the lambda.
    //
    // mutable allows the lambda to modify its captured vector.
    std::thread worker{[numbers = std::move(numbers)]() mutable {
        numbers.push_back(50);

        std::cout << "Worker owns:";

        for (int number : numbers) {
            std::cout << ' ' << number;
        }

        std::cout << '\n';
    }};

    worker.join();

    // The original C++ vector still exists but is moved-from.
    std::cout << "Original vector size after move: " << numbers.size() << '\n';
}

void message_passing_example() {
    std::cout << "\n=== 3. Message passing ===\n";

    auto [sender, receiver] = make_channel<std::string>();

    // Moving sender into the lambda transfers this Sender object into the
    // worker. Main no longer owns that Sender object.
    std::jthread producer{[sender = std::move(sender)] {
        sender.send("Hello through the channel");
    }};

    // The producer's Sender is destroyed when its lambda/thread finishes.
    //
    // receive() returns:
    //
    //     optional containing string -> received message
    //     std::nullopt                -> all senders are gone
    if (auto message = receiver.receive()) {
        std::cout << "Received: " << *message << '\n';
    }
}

void multiple_producers_example() {
    std::cout << "\n=== 4. Multiple producers ===\n";

    auto [sender_one, receiver] = make_channel<std::string>();

    // Copying creates another active sender.
    auto sender_two = sender_one;

    std::jthread producer_one{[sender = std::move(sender_one)] {
        const std::vector<std::string> messages{
            "one: hello",
            "one: from",
            "one: thread",
        };

        for (const std::string &message : messages) {
            sender.send(message);
            std::this_thread::sleep_for(40ms);
        }

        // Captured sender is destroyed when this thread finishes.
    }};

    std::jthread producer_two{[sender = std::move(sender_two)] {
        const std::vector<std::string> messages{
            "two: more",
            "two: messages",
            "two: here",
        };

        for (const std::string &message : messages) {
            sender.send(message);
            std::this_thread::sleep_for(60ms);
        }

        // Captured sender is destroyed when this thread finishes.
    }};

    // We do not need to know how many messages will be sent.
    //
    // receive() keeps returning messages until:
    //
    //     1. Every Sender has been destroyed
    //     2. The queue has been completely drained
    while (auto message = receiver.receive()) {
        std::cout << "Received: " << *message << '\n';
    }

    std::cout << "All senders disconnected\n";

    // jthread automatically joins.
}

struct Counter {
    std::mutex mutex;
    std::uint64_t value{0};
};

void shared_state_example() {
    std::cout << "\n=== 5. Shared state with a mutex ===\n";

    // Rust:
    //
    //     Arc<Mutex<u64>>
    //
    // C++:
    //
    //     shared_ptr<struct containing mutex and value>
    auto counter = std::make_shared<Counter>();

    std::vector<std::thread> workers;

    constexpr std::uint32_t worker_count{4};
    constexpr std::uint32_t increments_per_worker{100'000};

    for (std::uint32_t id{0}; id < worker_count; ++id) {
        workers.emplace_back([counter, id] {
            for (std::uint32_t i{0}; i < increments_per_worker; ++i) {
                std::lock_guard lock{counter->mutex};
                ++counter->value;
            }

            std::cout << "Worker " << id << " finished\n";
        });
    }

    for (std::thread &worker : workers) {
        worker.join();
    }

    {
        std::lock_guard lock{counter->mutex};

        std::cout << "Expected counter: "
                  << worker_count * increments_per_worker << '\n';

        std::cout << "Actual counter:   " << counter->value << '\n';
    }
}

void cancellation_example() {
    std::cout << "\n=== 6. jthread and cooperative cancellation ===\n";

    std::jthread worker{[](std::stop_token stop_token) {
        std::uint32_t iteration{0};

        while (!stop_token.stop_requested()) {
            std::cout << "Working: " << iteration++ << '\n';
            std::this_thread::sleep_for(75ms);
        }

        std::cout << "Worker noticed the stop request\n";
    }};

    std::this_thread::sleep_for(300ms);

    // Cancellation is cooperative. This sets a flag; it does not forcibly
    // kill the thread.
    worker.request_stop();

    // Explicit join is optional because jthread joins automatically.
}

int main() {
    std::cout << "Main thread ID: " << std::this_thread::get_id() << '\n';

    basic_thread_example();
    move_capture_example();
    message_passing_example();
    multiple_producers_example();
    shared_state_example();
    cancellation_example();

    std::cout << "\nAll examples completed\n";
}
