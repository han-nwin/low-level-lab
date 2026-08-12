# C++20/23 Concurrency Notes

This note follows roughly the same path as the Rust Book concurrency chapter: spawning threads, moving data into threads, message passing, shared-state concurrency, and thread-safety rules.

Everything here requires **C++20 or newer**. C++23 still does not include a standard channel.

## Build command

```bash
clang++ -std=c++20 -Wall -Wextra -Wpedantic -pthread -g main.cpp -o concurrency
./concurrency
```

`-pthread` enables the platform's threading support for both compilation and linking.

## 1. Spawning and joining a thread

```cpp
#include <iostream>
#include <thread>

int main() {
    std::thread worker{[] {
        std::cout << "Hello from worker\n";
    }};

    std::cout << "Hello from main\n";
    worker.join();
}
```

`std::thread` starts running immediately. The main and worker threads may print in either order because the scheduler decides when each runs.

`join()` blocks the calling thread until the worker finishes:

```text
main creates worker
main and worker run concurrently
main calls join()
main waits if worker is still running
main continues after worker finishes
```

Rust comparison:

```rust
let handle = std::thread::spawn(|| {
    println!("Hello from worker");
});

handle.join().unwrap();
```

Important C++ rule: destroying a joinable `std::thread` calls `std::terminate()`. Therefore, call `join()` before it leaves scope. Avoid `detach()` while learning because a detached thread may outlive objects it references.

## 2. `std::jthread`: the safer C++20 thread

`std::jthread` automatically joins when it leaves scope:

```cpp
#include <iostream>
#include <thread>

int main() {
    std::jthread worker{[] {
        std::cout << "Working\n";
    }};

    // Automatically joins at the end of main().
}
```

Prefer `std::jthread` when automatic joining and cancellation are useful. Use `std::thread` when you specifically need its lower-level lifetime behavior.

## 3. Capturing data in a thread lambda

A lambda capture determines how the new thread accesses surrounding variables.

### Copy capture

```cpp
int number{42};

std::thread worker{[number] {
    std::cout << number << '\n';
}};

worker.join();
```

The closure owns a copy of `number`.

### Reference capture

```cpp
int number{0};

std::thread worker{[&number] {
    number = 42;
}};

worker.join();
std::cout << number << '\n';
```

This is safe here because `number` stays alive and main reads it only after `join()`. Reference capture becomes dangerous if the referenced object dies too early or another thread accesses it concurrently without synchronization.

### Move capture: closest to a Rust `move` closure

```cpp
#include <utility>
#include <vector>

std::vector<int> numbers{1, 2, 3};

std::thread worker{
    [numbers = std::move(numbers)] {
        // The closure now owns the vector.
    }
};

worker.join();
```

Rust equivalent:

```rust
let numbers = vec![1, 2, 3];

let handle = std::thread::spawn(move || {
    println!("{numbers:?}");
});
```

After `std::move`, the original C++ object remains valid but is in a moved-from state. Do not assume it still contains its old data.

## 4. Passing function arguments

Thread arguments are stored by value by default:

```cpp
void print_number(int number) {
    std::cout << number << '\n';
}

std::thread worker{print_number, 42};
worker.join();
```

Use `std::ref()` to intentionally pass an lvalue reference:

```cpp
#include <functional>

void increment(int& value) {
    ++value;
}

int value{0};
std::thread worker{increment, std::ref(value)};
worker.join();
```

## 5. Data races

A data race happens when:

- two or more threads access the same memory concurrently,
- at least one access writes, and
- the accesses are not synchronized.

In C++, a data race is **undefined behavior**.

```cpp
int counter{0};

// Wrong: both threads modify counter without synchronization.
std::thread a{[&] { ++counter; }};
std::thread b{[&] { ++counter; }};
```

`counter++` is a read-modify-write operation, not one indivisible action. The threads can overwrite each other's updates.

## 6. Shared-state concurrency with `std::mutex`

```cpp
#include <cstdint>
#include <memory>
#include <mutex>
#include <thread>
#include <vector>

struct Counter {
    std::mutex mutex;
    std::uint64_t value{0};
};

int main() {
    auto counter = std::make_shared<Counter>();
    std::vector<std::thread> workers;

    for (int id{0}; id < 4; ++id) {
        workers.emplace_back([counter] {
            for (int i{0}; i < 100'000; ++i) {
                std::lock_guard lock{counter->mutex};
                ++counter->value;
            }
        });
    }

    for (std::thread& worker : workers) {
        worker.join();
    }
}
```

`std::lock_guard` uses RAII:

```text
construct lock_guard  -> lock mutex
leave scope           -> destroy lock_guard -> unlock mutex
```

You normally do not call `unlock()` manually. This is similar to Rust's mutex guard being dropped.

### Rust versus C++ shared state

| Rust | C++20/23 |
|---|---|
| `Arc<T>` | `std::shared_ptr<T>` |
| `Arc::clone(&x)` | Copy the `shared_ptr` |
| `Mutex<T>` contains `T` | `std::mutex` is normally stored beside `T` |
| `x.lock().unwrap()` | `std::lock_guard lock{x}` |
| Guard drop unlocks | Guard destructor unlocks |
| `Arc<Mutex<T>>` | `shared_ptr` to a struct containing `mutex` and `T` |

`shared_ptr` makes shared **lifetime ownership** safe. It does not automatically make the pointed-to object safe for concurrent mutation.

## 7. Keep critical sections small

The critical section is the code executed while holding a lock:

```cpp
{
    std::lock_guard lock{mutex};
    ++counter; // critical section
}
```

Only protect the operations that need protection. Holding a mutex during slow I/O, sleeping, or expensive work prevents other threads from progressing.

Also avoid acquiring multiple mutexes in inconsistent orders; that can cause deadlock.

## 8. `lock_guard` versus `unique_lock`

Use `std::lock_guard` for simple scope-based locking:

```cpp
std::lock_guard lock{mutex};
```

Use `std::unique_lock` when you need to unlock/relock manually or work with a condition variable:

```cpp
std::unique_lock lock{mutex};
lock.unlock();
lock.lock();
```

`unique_lock` is more flexible but slightly heavier.

## 9. Message passing: a Rust-style MPSC channel

Rust includes `std::sync::mpsc`. Neither C++20 nor C++23 has a standard reusable channel, so this example builds one with separate sending and receiving endpoints:

```cpp
auto [sender, receiver] = make_channel<std::string>();
```

MPSC means **multiple producer, single consumer**. `Sender<T>` may be copied to create more producers, but `Receiver<T>` cannot be copied. The channel closes automatically when the last sender is destroyed.

The implementation combines:

```text
std::queue<T>              stores buffered messages
std::mutex                 protects all shared channel state
std::condition_variable    sleeps/wakes the receiver
std::shared_ptr            shares one ChannelState between endpoints
std::optional<T>           represents a message or end-of-stream
sender_count               detects when the final sender disappears
```

### Complete channel implementation

```cpp
#include <condition_variable>
#include <memory>
#include <mutex>
#include <optional>
#include <queue>
#include <utility>

template <typename T>
struct ChannelState {
    std::queue<T> messages;
    std::mutex mutex;
    std::condition_variable condition;
    std::size_t sender_count{0};
};

template <typename T>
struct Sender {
    explicit Sender(std::shared_ptr<ChannelState<T>> state)
        : state_{std::move(state)} {
        add_sender();
    }

    // Copy constructor: creates a NEW sender from an existing sender.
    // Similar to Rust's tx.clone(), so sender_count is incremented.
    //
    //     auto sender_two = sender_one;
    Sender(const Sender& other)
        : state_{other.state_} {
        add_sender();
    }

    // Move constructor: transfers an existing sender endpoint into a NEW
    // object. No additional logical sender is created, so sender_count stays
    // unchanged.
    //
    //     auto sender_two = std::move(sender_one);
    //
    // `= default` asks the compiler to generate member-wise move behavior.
    // `noexcept` promises that the operation will not throw.
    Sender(Sender&& other) noexcept = default;

    // Copy assignment would replace an ALREADY EXISTING sender:
    //
    //     sender_two = sender_one;
    //
    // `= delete` forbids this syntax at compile time. Implementing it would
    // require correctly releasing sender_two's old count and acquiring the
    // count represented by sender_one.
    Sender& operator=(const Sender&) = delete;

    // Move assignment would transfer into an ALREADY EXISTING sender:
    //
    //     sender_two = std::move(sender_one);
    //
    // It is deleted in this teaching implementation to keep sender-count
    // ownership straightforward.
    Sender& operator=(Sender&&) = delete;

    ~Sender() {
        remove_sender();
    }

    void send(T value) const {
        {
            std::lock_guard lock{state_->mutex};
            state_->messages.push(std::move(value));
        }

        // One message became available, so wake one waiting receiver.
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
            // Wake a receiver waiting on an empty queue so it can observe
            // end-of-stream instead of sleeping forever.
            state_->condition.notify_all();
        }
    }

    std::shared_ptr<ChannelState<T>> state_;
};

template <typename T>
struct Receiver {
    explicit Receiver(std::shared_ptr<ChannelState<T>> state)
        : state_{std::move(state)} {}

    // Copy construction would create a second receiver, violating MPSC's
    // single-consumer design, so it is forbidden.
    //
    //     auto receiver_two = receiver; // compile error
    Receiver(const Receiver&) = delete;

    // Also forbid copy assignment between existing receivers.
    Receiver& operator=(const Receiver&) = delete;

    // Move construction transfers the one receiving endpoint into a NEW
    // Receiver. The source becomes moved-from.
    Receiver(Receiver&&) noexcept = default;

    // Move assignment transfers the endpoint into an ALREADY EXISTING
    // Receiver. The compiler-generated shared_ptr move is sufficient here.
    Receiver& operator=(Receiver&&) noexcept = default;

    std::optional<T> receive() {
        std::unique_lock lock{state_->mutex};

        state_->condition.wait(lock, [this] {
            return !state_->messages.empty() ||
                   state_->sender_count == 0;
        });

        // Empty queue plus zero senders means no future message can arrive.
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

template <typename T>
auto make_channel() {
    auto state = std::make_shared<ChannelState<T>>();

    return std::pair{
        // Copy the shared_ptr because Receiver still needs another ownership
        // handle to the same ChannelState.
        Sender<T>{state},

        // This is the local shared_ptr's final use, so move it into Receiver
        // instead of performing one extra reference-count increment/decrement.
        Receiver<T>{std::move(state)},
    };
}
```

### Is `ChannelState` shared state?

Yes. `make_shared` creates one state object on the heap, and both endpoints hold a `shared_ptr` to that exact object:

```text
Sender ────┐
           ├──> ChannelState<T>
Receiver ──┘
```

This is conceptually similar to Rust's `Arc<Mutex<ChannelState<T>>>`, except C++ conventionally stores the mutex beside the data it protects.

The factory copies `state` into `Sender` because `Receiver` still needs it. It then moves the final local ownership handle into `Receiver` because the local variable is no longer needed:

```cpp
Sender<T>{state}                 // shared_ptr copy: another owner is needed
Receiver<T>{std::move(state)}   // shared_ptr move: final local use
```

Moving the `shared_ptr` does not move the `ChannelState`; the state stays at the same heap address. Only an ownership handle is transferred.

### What the special member syntax means

```cpp
Type(const Type&)             // copy constructor: create new from existing
Type(Type&&)                  // move constructor: create new by transfer
Type& operator=(const Type&)  // copy assignment: replace existing from existing
Type& operator=(Type&&)       // move assignment: replace existing by transfer

= default;                    // compiler generates the normal implementation
= delete;                     // operation is forbidden at compile time
noexcept                      // function promises not to throw
```

Construction creates a new object; assignment operates on an object that already exists. That difference matters for `Sender`, because assignment may need to release one existing logical sender before acquiring another.

### How the condition variable works

Without a condition variable, a receiver might repeatedly inspect the queue:

```cpp
while (messages.empty()) {
    // Busy-waiting: wastes CPU.
}
```

`condition_variable::wait()` lets it sleep efficiently. It needs a `unique_lock` because it must temporarily unlock and later relock the mutex:

```text
receiver locks mutex
receiver checks predicate
predicate is false
wait atomically unlocks mutex and sleeps

sender locks mutex
sender changes queue or sender_count
sender unlocks mutex
sender notifies

receiver wakes
receiver locks mutex again
receiver checks predicate again
```

The predicate is the actual condition being awaited:

```cpp
!state_->messages.empty() || state_->sender_count == 0
```

The condition variable does not store whether a message exists. It only provides the sleep/wake mechanism. A notification means “the protected state may have changed; wake and check it again.”

The predicate form also handles **spurious wakeups**, where a thread wakes even though no useful state change occurred. If the predicate remains false, `wait()` sleeps again.

Memory trick:

```text
mutex               protects shared state
predicate           describes when progress is possible
condition_variable  sleeps until state may have changed
notify              says "wake up and check again"
```

### One producer

```cpp
auto [sender, receiver] = make_channel<std::string>();

std::jthread producer{
    [sender = std::move(sender)] {
        sender.send("hello");
        sender.send("from worker");
    }
};

while (auto message = receiver.receive()) {
    std::cout << *message << '\n';
}
```

The sender is moved into the worker lambda. When that lambda is destroyed after the thread finishes, the final `Sender` destructor reduces `sender_count` to zero and wakes the receiver.

### Multiple producers with an unknown message count

```cpp
auto [sender_one, receiver] = make_channel<std::string>();

// Copying is the equivalent of Rust's tx.clone().
auto sender_two = sender_one;

std::jthread producer_one{
    [sender = std::move(sender_one)] {
        sender.send("one: hello");
        sender.send("one: done");
    }
};

std::jthread producer_two{
    [sender = std::move(sender_two)] {
        sender.send("two: hello");
        sender.send("two: done");
    }
};

// No message count is required. receive() returns nullopt only after every
// Sender is destroyed and all buffered messages have been drained.
while (auto message = receiver.receive()) {
    std::cout << *message << '\n';
}
```

Rust comparison:

```rust
let (tx, rx) = std::sync::mpsc::channel();
let tx2 = tx.clone();

std::thread::spawn(move || tx.send("one").unwrap());
std::thread::spawn(move || tx2.send("two").unwrap());

for message in rx {
    println!("{message}");
}
```

The essential close rule is:

```text
sender_count > 0               more messages may still arrive
sender_count == 0, queue full  drain remaining buffered messages
sender_count == 0, queue empty end-of-stream: return std::nullopt
```

## 10. One-result communication: futures

For one result rather than a stream of messages, use `std::future`.

### `std::async`

```cpp
#include <future>

std::future<int> result = std::async(
    std::launch::async,
    [] {
        return 6 * 7;
    }
);

std::cout << result.get() << '\n';
```

`get()` waits if necessary and returns the value. It may be called only once.

### `std::promise` and `std::future`

```cpp
std::promise<int> sender;
std::future<int> receiver{sender.get_future()};

std::thread worker{
    [sender = std::move(sender)]() mutable {
        sender.set_value(42);
    }
};

std::cout << receiver.get() << '\n';
worker.join();
```

Think of this pair as a one-shot channel:

```text
promise<T> -> sending side
future<T>  -> receiving side
```

## 11. Cooperative cancellation with `std::jthread`

```cpp
#include <chrono>
#include <stop_token>
#include <thread>

using namespace std::chrono_literals;

std::jthread worker{[](std::stop_token token) {
    while (!token.stop_requested()) {
        // Perform a small unit of work.
        std::this_thread::sleep_for(50ms);
    }
}};

std::this_thread::sleep_for(200ms);
worker.request_stop();
```

The stop request does not forcibly kill the thread. The worker must check its token and exit voluntarily.

## 12. Atomics

For a simple independent counter or flag, an atomic may avoid a mutex:

```cpp
#include <atomic>

std::atomic<std::uint64_t> counter{0};
counter.fetch_add(1, std::memory_order_relaxed);
```

Atomics are appropriate for simple state transitions and counters. A mutex is usually easier when several values must change together as one invariant. Do not default to atomics merely because they appear faster; memory ordering becomes subtle quickly.

## 13. Rust `Send` and `Sync` versus C++

Rust's compiler tracks:

- `Send`: ownership of a value may cross a thread boundary.
- `Sync`: shared references to a value may be used across threads.

C++ has no exact compiler-enforced equivalent. You must understand each type's concurrency guarantees.

For example, different threads may safely copy and destroy `shared_ptr` instances sharing one control block. That does not make simultaneous writes to the pointed-to `T` safe.

## 14. C++20/23 concurrency tool map

| Need | Standard tool |
|---|---|
| Start a thread | `std::thread` |
| Safer auto-joining thread | `std::jthread` |
| Wait for a thread | `join()` |
| Request cancellation | `std::stop_token` |
| Protect shared state | `std::mutex` |
| Simple scoped lock | `std::lock_guard` |
| Flexible lock | `std::unique_lock` |
| Sleep until state changes | `std::condition_variable` |
| One asynchronous result | `std::future` |
| Send one result manually | `std::promise` |
| Simple concurrent counter/flag | `std::atomic` |
| Share object lifetime | `std::shared_ptr` |
| Reusable message channel | Not provided in C++20 or C++23 |

## 15. Rules worth memorizing

1. Join every `std::thread` before its destructor runs.
2. Prefer moving owned data into a worker over capturing references unnecessarily.
3. A shared pointer protects lifetime, not the object from data races.
4. Every shared mutable non-atomic value needs synchronization.
5. Use RAII lock guards so exceptions and early returns still unlock mutexes.
6. Keep critical sections small.
7. Never hold a lock while sleeping unless that behavior is intentional.
8. Use condition variables instead of repeatedly polling shared state.
9. Use futures for one result and a queue/channel for a stream of results.
10. Prefer `std::jthread` when automatic joining and cooperative stopping fit the task.

## Mental model

```text
std::thread / jthread        who performs the work?
move capture                 who owns the worker's data?
shared_ptr                   how long does shared data live?
mutex                        who may mutate shared data now?
condition_variable           how does a waiting thread sleep and wake?
channel                      how are messages transferred?
future                       how is one result returned?
atomic                       how is one simple value updated indivisibly?
```

The central idea is the same as in Rust: concurrency is mostly about ownership, lifetime, communication, and synchronized access. Rust enforces more of those rules through its type system; C++ gives you the mechanisms but expects you to enforce the design correctly.
