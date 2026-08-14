#include <condition_variable>
#include <iostream>
#include <memory>
#include <mutex>
#include <queue>
#include <tuple>
#include <utility>

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
    void send() {}

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
    [[nodiscard]] T recv() {}

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
    sender.state_->messages->push(1);
    sender.state_->messages->push(2);
    sender.state_->messages->push(3);
    sender.state_->messages->push(4);
    sender.state_->messages->push(5);

    while (true) {
        if (receiver.state_->sender_count == 0) {
            break;
        }
        auto message = receiver.state_->messages->front();
        receiver.state_->messages->pop();
        std::cout << message << std::endl;
    }
}
