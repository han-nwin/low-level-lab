#include <condition_variable>
#include <iostream>
#include <memory>
#include <mutex>
#include <optional>
#include <queue>
#include <thread>
#include <tuple>
#include <utility>

using namespace std::chrono_literals;

template <typename T> struct ChannelState {
    // NOTE: using `{}` to initialize default values
    // so that make_shared can create new object with default values
    std::queue<T> messages{};
    std::mutex mutex{};
    std::condition_variable condition{};
    int sender_count{0};
};

template <typename T> struct Sender {
    // NOTE: the param is copied in, but then we move it -> new object
    Sender(std::shared_ptr<ChannelState<T>> state) : state_(std::move(state)) {
        {
            std::lock_guard lock{state_->mutex};
            state_->sender_count++; // increase when a sender created
        }
    }

    // Copy constructer
    Sender(const Sender &other) = delete;
    // Copy opertor
    Sender &operator=(const Sender &other) = delete;

    // Move constructor
    Sender(Sender &&other) noexcept : state_{std::move(other.state_)} {}
    // Move opertor
    Sender &operator=(Sender &&other) = delete;

    ~Sender() {

        // return if the state_ already destroyed
        if (!state_) {
            return;
        }

        {
            std::lock_guard lock{state_->mutex};
            state_->sender_count--; // decrease when a sender destroyed

            // Noti that sender_count is 0
            if (state_->sender_count == 0) {
                // notify all receivers
                state_->condition.notify_all();
            }
        }
    }

    // clone() is like a copy constructor
    [[nodiscard]] Sender clone() {
        // NOTE:
        // state_ copied in as param, but then move to new_sender
        // and the increment count already increased by the constructor
        Sender new_sender = Sender{state_};
        return new_sender;
    }

    // TODO:
    void send(T msg) {
        if (!state_) {
            return;
        }
        {
            std::lock_guard lock{state_->mutex};
            state_->messages.push(std::move(msg));
        }
        // Wake one receiver because one message became available.
        state_->condition.notify_one();
    }

  private:
    std::shared_ptr<ChannelState<T>> state_;
};

template <typename T> struct Receiver {
    // move the state in not copy
    Receiver(std::shared_ptr<ChannelState<T>> state)
        : state_(std::move(state)) {}

    // Copy constructor
    Receiver(const Receiver &other) = delete;
    // Copy opertor
    Receiver &operator=(const Receiver &other) = delete;

    // Move constructor
    Receiver(Receiver &&other) noexcept : state_{std::move(other.state_)} {}
    // Move opertor
    Receiver &operator=(Receiver &&other) = delete;

    ~Receiver() {}

    // TODO:
    [[nodiscard]] std::optional<T> recv() {
        if (!state_) {
            return std::nullopt;
        }
        {
            std::unique_lock lock{state_->mutex};
            // NOTE: We wake when:
            // - a message exists, OR
            // - every sender has been destroyed
            // [this]: mean passing the Receiver object itself into the lambda
            state_->condition.wait(lock, [this] {
                // wake up and lock if messages is not empty or sender count = 0
                return !state_->messages.empty() || state_->sender_count == 0;
            });

            if (state_->messages.empty()) {
                return std::nullopt;
            }

            T received_message = state_->messages.front();
            state_->messages.pop();
            return received_message;
        }
    }

  private:
    std::shared_ptr<ChannelState<T>> state_;
};

template <typename T> std::pair<Sender<T>, Receiver<T>> make_channel() {

    // NOTE:
    // make_shared with no argument here will rely on the constructor of the
    // ChannelState to create a new object
    std::shared_ptr<ChannelState<T>> state =
        std::make_shared<ChannelState<T>>();

    Sender<T> sender = Sender{state};
    Receiver<T> receiver = Receiver{std::move(state)};

    return std::pair<Sender<T>, Receiver<T>>{std::move(sender),
                                             std::move(receiver)};
}

int main() {
    auto [sender, receiver] = make_channel<int>();
    auto sender_2 = sender.clone();

    std::jthread worker{[sender = std::move(sender)]() mutable {
        for (int i = 1; i <= 5; i++) {
            std::println("worker: working on job {}", i);
            std::this_thread::sleep_for(100ms); // simulate work
            sender.send(i);
        }
    }};

    std::jthread worker_2{[sender_2 = std::move(sender_2)]() mutable {
        for (int i = 6; i <= 10; i++) {
            std::println("worker_2: working on job {}", i);
            std::this_thread::sleep_for(100ms); // simulate work
            sender_2.send(i);
        }
    }};

    std::jthread observer{[receiver = std::move(receiver)]() mutable {
        while (auto message = receiver.recv()) {
            std::println("observer: got {}", *message);
        }
    }};
}
