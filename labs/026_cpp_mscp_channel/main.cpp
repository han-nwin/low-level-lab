#include <condition_variable>
#include <iostream>
#include <memory>
#include <mutex>
#include <queue>
#include <tuple>

template <typename T> struct ChannelState {
    std::shared_ptr<std::queue<T>> messages_;
    std::mutex mutex_;
    std::condition_variable condition_;
    int sender_count_;
};

template <typename T> struct Sender {
    // move the state in not copy
    Sender(std::shared_ptr<ChannelState<T>> state) : state_(std::move(state)) {
        {
            std::lock_guard lock{state_->mutex_};
            state_->sender_count_++; // increase when a sender created
        }
    }

    // Copy constructer
    Sender(const Sender &other) = delete;

    // Move constructor
    Sender(Sender &&other) noexcept : state_{std::move(other.state_)} {}

    // Copy opertor
    Sender &operator=(const Sender &other) = delete;
    // Move opertor
    Sender &operator=(Sender &&other) = delete;

    ~Sender() {
        {
            std::lock_guard lock{state_->mutex_};
            state_->sender_count_--; // decrease when a sender destroyed

            if (state_->sender_count_ == 0) {
                // notify all receivers
                state_->condition_.notify_all();
            }
        }
    }

    [[nodiscard]] Sender clone() {}

  private:
    std::shared_ptr<ChannelState<T>> state_;
};

template <typename T> struct Receiver {
    // move the state in not copy
    Receiver(std::shared_ptr<ChannelState<T>> state)
        : state_(std::move(state)) {}

    // Copy constructor
    Receiver(const Receiver &other) = delete;
    // Move constructor
    Receiver(Receiver &&other) = delete;
    // Copy opertor
    Receiver &operator=(const Receiver &other) = delete;
    // Move opertor
    Receiver &operator=(Receiver &&other) = delete;

    ~Receiver() {}

  private:
    std::shared_ptr<ChannelState<T>> state_;
};

template <typename T> std::pair<Sender<T>, Receiver<T>> make_channel() {
    std::shared_ptr<ChannelState<T>> state =
        std::make_shared<ChannelState<T>>(ChannelState<T>{
            .messages_ = std::make_shared<std::queue<T>>(),
            .sender_count_ = 0,
        });
    Sender<T> sender = Sender{state};
    Receiver<T> receiver = Receiver{std::move(state)};

    return std::pair{sender, receiver};
}

int main() {
    auto [sender, receiver] = make_channel<int>();
    sender.state_->messages_->push(1);
    sender.state_->messages_->push(2);
    sender.state_->messages_->push(3);
    sender.state_->messages_->push(4);
    sender.state_->messages_->push(5);

    while (true) {
        if (receiver.state_->sender_count_ == 0) {
            break;
        }
        auto message = receiver.state_->messages_->front();
        receiver.state_->messages_->pop();
        std::cout << message << std::endl;
    }
}
