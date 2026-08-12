# C++20/23 Async and Futures

This note explains C++ asynchronous work using `std::async`, `std::future`, `std::promise`, and `std::shared_future`, with comparisons to Rust threads and Rust `async`/`.await`.

## The most important distinction

Despite its name, C++ `std::async` is **not** equivalent to Rust's async runtime model.

```text
C++ std::async
    runs a regular blocking function
    often uses a separate OS thread
    returns a std::future containing its eventual result
    future.get() blocks the calling OS thread

Rust async
    creates a state machine implementing Future
    does nothing until polled
    normally needs an executor/runtime such as Tokio
    .await suspends the task instead of blocking the executor thread
```

The closest Rust comparison for ordinary `std::async(std::launch::async, ...)` is often:

```text
C++ std::async + future.get()
    ≈ Rust std::thread::spawn + handle.join()
```

It only looks syntactically similar to `.await`; its execution model is different.

## Build command

```bash
clang++ -std=c++20 -Wall -Wextra -Wpedantic -pthread -g main.cpp -o async_lab
./async_lab
```

## 1. Basic `std::async` and `std::future`

```cpp
#include <chrono>
#include <future>
#include <iostream>
#include <thread>

using namespace std::chrono_literals;

int calculate_square(int value) {
    std::this_thread::sleep_for(500ms);
    return value * value;
}

int main() {
    std::future<int> result = std::async(
        std::launch::async,
        calculate_square,
        7
    );

    // Main can perform other work while calculate_square runs.
    std::cout << "Main continues\n";

    // Wait if necessary, then take the result.
    int value{result.get()};
    std::cout << value << '\n';
}
```

Read this expression as:

```cpp
std::future<int> result = std::async(
    std::launch::async, // require asynchronous execution
    calculate_square,  // callable
    7                  // argument passed to callable
);
```

`std::future<int>` means:

> A handle to an `int` that may not exist yet.

`result.get()` means:

1. Wait until the operation finishes if necessary.
2. Return its result.
3. Consume the future's result.

## 2. `get()` is blocking

```cpp
int value{result.get()};
```

If the result is not ready, the current OS thread stops there and waits.

Rust `.await` normally behaves differently:

```rust
let value = task.await?;
```

The Rust task can yield control to its executor, allowing that executor thread to poll other ready tasks. That is why thousands of Rust async tasks can share a small number of OS threads.

```text
future.get()    blocks thread
.await          suspends task
```

## 3. Launch policies

### Require asynchronous execution

```cpp
std::async(std::launch::async, work);
```

The function runs asynchronously, normally on another thread.

### Deferred execution

```cpp
std::async(std::launch::deferred, work);
```

The function does not run when the future is created. It runs synchronously on whichever thread first calls `get()` or `wait()`.

```cpp
auto result = std::async(std::launch::deferred, [] {
    return 42;
});

// Work has not run yet.
int value{result.get()}; // Work runs here on this thread.
```

### Omitting the policy

```cpp
std::async(work);
```

The implementation may choose asynchronous or deferred execution. If concurrency matters, specify `std::launch::async` explicitly.

## 4. Waiting without consuming the result

`wait()` blocks until the result is ready but does not retrieve it:

```cpp
result.wait();
int value{result.get()};
```

`wait_for()` waits for a limited duration:

```cpp
using namespace std::chrono_literals;

if (result.wait_for(100ms) == std::future_status::ready) {
    std::cout << result.get() << '\n';
} else {
    std::cout << "Not ready yet\n";
}
```

Possible statuses include:

```text
std::future_status::ready       result is available
std::future_status::timeout     duration expired
std::future_status::deferred    operation uses deferred execution
```

Polling repeatedly with `wait_for()` is usually less elegant than structuring the program so it can do useful work before eventually calling `get()`.

## 5. Running several tasks concurrently

```cpp
#include <future>
#include <vector>

std::vector<std::future<int>> results;

for (int number : {2, 3, 4, 5}) {
    results.push_back(std::async(
        std::launch::async,
        calculate_square,
        number
    ));
}

for (std::future<int>& result : results) {
    std::cout << result.get() << '\n';
}
```

All tasks are launched before results are collected. Calling `get()` in vector order does not mean the operations finish in that order; it only means the program retrieves them in that order.

Be careful about launching huge numbers of `std::async` operations. It is not a standard thread pool API and may create many OS threads.

## 6. Moving ownership into async work

Arguments passed to `std::async` are normally stored by value. Use `std::move` for an object whose resources should be transferred into the task:

```cpp
#include <string>

std::size_t process_text(std::string text) {
    return text.size();
}

std::string text{"C++ async"};

std::future<std::size_t> result = std::async(
    std::launch::async,
    process_text,
    std::move(text)
);
```

The original C++ `text` object still exists but is moved-from. It is valid, but its previous contents must not be assumed.

Rust comparison:

```rust
let text = String::from("Rust async");

let task = tokio::spawn(async move {
    text.len()
});

// text is no longer accessible here.
```

## 7. Passing by reference

Use `std::ref()` when the callable requires an lvalue reference:

```cpp
#include <functional>

void update(int& value) {
    value = 42;
}

int value{0};

auto result = std::async(
    std::launch::async,
    update,
    std::ref(value)
);

result.get();
```

The referenced object must remain alive until the operation finishes. Concurrent access must also be synchronized. Moving ownership into the task is often easier to reason about than borrowing by reference.

## 8. Exceptions travel through futures

If the async callable throws, the exception is stored in the shared result state and rethrown by `get()`:

```cpp
#include <stdexcept>

auto result = std::async(std::launch::async, []() -> int {
    throw std::runtime_error{"calculation failed"};
});

try {
    int value{result.get()};
} catch (const std::exception& error) {
    std::cout << error.what() << '\n';
}
```

Rust normally represents expected failure explicitly:

```rust
async fn calculate() -> Result<i32, Error> {
    // ...
}

let value = calculate().await?;
```

C++ exceptions are closer to an implicit error path stored inside the future.

## 9. `std::promise` and `std::future`

`std::async` creates and connects the result machinery automatically. A `std::promise` lets you manually produce a value that another part of the program receives through a future.

```cpp
#include <future>
#include <string>
#include <thread>
#include <utility>

std::promise<std::string> sender;
std::future<std::string> receiver{sender.get_future()};

std::jthread worker{
    [sender = std::move(sender)]() mutable {
        sender.set_value("hello from worker");
    }
};

std::string message{receiver.get()};
```

Mental model:

```text
promise<T>    producing/sending side
future<T>     consuming/receiving side
```

This resembles a one-shot Rust channel, not an MPSC stream:

```rust
let (sender, receiver) = tokio::sync::oneshot::channel();
sender.send(value)?;
let value = receiver.await?;
```

A promise can also deliver an exception:

```cpp
try {
    // work
} catch (...) {
    sender.set_exception(std::current_exception());
}
```

If a promise is destroyed without setting a value or exception, the receiver observes a `std::future_error` for a broken promise.

## 10. `future` versus `shared_future`

`std::future<T>` is move-only and normally has one consumer:

```cpp
int value{result.get()};
// result.get(); // Invalid: the value was already consumed.
```

Use `valid()` to check whether a future still refers to a shared state:

```cpp
if (result.valid()) {
    // wait/get may be used
}
```

`std::shared_future<T>` is copyable and allows several consumers:

```cpp
std::shared_future<int> shared = std::async(
    std::launch::async,
    [] { return 42; }
).share();

std::jthread first{[shared] {
    std::cout << shared.get() << '\n';
}};

std::jthread second{[shared] {
    std::cout << shared.get() << '\n';
}};
```

Useful intuition:

```text
future<T>          one handle / one consumer
shared_future<T>   copyable handles / multiple consumers
```

This resembles the ownership difference between `unique_ptr` and `shared_ptr`, but applied to a shared result state.

## 11. `std::async` versus `std::thread` and `std::jthread`

| Tool | Best mental model | Result | Cancellation | Lifetime |
|---|---|---|---|---|
| `std::thread` | Run this callable on a thread | Manual/shared output | Manual | Must join or detach |
| `std::jthread` | RAII-owned stoppable thread | Manual/shared output | `stop_token` | Requests stop and joins |
| `std::async` | Run computation and give me its result | `future<T>` | No built-in stop request | Associated with future |

Choose based on the job:

```text
Need a returned value from a finite calculation?   std::async
Need a long-running worker loop?                   std::jthread
Need exact manual thread lifetime behavior?        std::thread
```

`std::jthread` still represents an actual thread. It is not a lightweight async task.

## 12. An important future-destructor surprise

The future returned by `std::async(std::launch::async, ...)` can wait for its operation to finish when that future is destroyed. Therefore, immediately discarding the future can accidentally serialize calls:

```cpp
std::async(std::launch::async, work_one); // temporary future may wait here
std::async(std::launch::async, work_two); // may not start until work_one ends
```

Keep the futures alive when you intend work to overlap:

```cpp
auto first  = std::async(std::launch::async, work_one);
auto second = std::async(std::launch::async, work_two);

first.get();
second.get();
```

Also do not mark a meaningful async result `[[maybe_unused]]` merely to silence the issue; explicitly retain and wait for it so the lifetime policy is obvious.

## 13. C++20 coroutines are a separate feature

C++20 added coroutine language keywords:

```cpp
co_await
co_yield
co_return
```

But the standard library did not provide a complete general-purpose executor/runtime like Tokio. Coroutine keywords are low-level machinery used by libraries to build task types, schedulers, generators, and asynchronous I/O systems.

```text
C++20 coroutine keywords = language mechanism
Tokio                    = runtime + reactor + scheduler + APIs
```

Writing `co_await` alone does not automatically provide networking, a scheduler, or background threads.

Standard `std::future` is also not directly awaitable in ordinary C++20 code, and C++20/23 does not provide standard `future.then(...)` chaining.

## 14. CPU-bound versus I/O-bound work

`std::async` can be reasonable for a small number of independent CPU-bound blocking calculations:

```text
image chunk processing
independent calculations
parallel parsing
compression jobs
```

For large quantities of jobs, a bounded thread pool is usually better than potentially creating one thread per operation.

For high-volume asynchronous I/O, Rust/Tokio-style runtimes or C++ async-I/O libraries are more appropriate because blocking one OS thread per connection scales poorly.

## 15. C++ and Rust comparison table

| C++20/23 | Closest Rust idea | Important difference |
|---|---|---|
| `std::async(std::launch::async, f)` | `thread::spawn(f)` | C++ returns a future rather than a join handle |
| `std::future<T>` | eventual result / `JoinHandle<T>` | C++ future is not Rust's `Future` trait |
| `future.get()` | `join()` more than `.await` | Blocks the OS thread |
| `std::promise<T>` | oneshot sender | Manually completes one result |
| `std::shared_future<T>` | shared completed result | Multiple consumers may call `get()` |
| `std::jthread` | scoped RAII worker concept | Requests stop and joins on destruction |
| C++ coroutine | Rust compiler-generated future machinery | C++ lacks a standard Tokio-like runtime |
| Exception in future | `Result<T, E>` / panic handling | C++ error path is exception-based |

## 16. Complete `main.cpp` example

```cpp
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

int square(int value) {
    std::this_thread::sleep_for(200ms);
    return value * value;
}

int main() {
    // One asynchronous result.
    std::future<int> answer = std::async(
        std::launch::async,
        square,
        7
    );

    std::cout << "Main performs other work\n";
    std::cout << "7 squared = " << answer.get() << "\n\n";

    // Several asynchronous results.
    std::vector<std::future<int>> results;

    for (int value : {2, 3, 4, 5}) {
        results.push_back(std::async(
            std::launch::async,
            square,
            value
        ));
    }

    for (std::future<int>& result : results) {
        std::cout << result.get() << '\n';
    }

    // One-shot manually produced result.
    std::promise<std::string> sender;
    std::future<std::string> receiver{sender.get_future()};

    std::jthread producer{
        [sender = std::move(sender)]() mutable {
            std::this_thread::sleep_for(100ms);
            sender.set_value("message from producer");
        }
    };

    std::cout << receiver.get() << '\n';

    // Exception propagation.
    auto failure = std::async(std::launch::async, []() -> int {
        throw std::runtime_error{"worker failed"};
    });

    try {
        std::cout << failure.get() << '\n';
    } catch (const std::exception& error) {
        std::cout << "Caught: " << error.what() << '\n';
    }
}
```

## 17. Rules worth memorizing

1. Use `std::launch::async` when you require concurrent execution.
2. `future.get()` blocks the calling thread and consumes a normal future's result.
3. Rust `.await` suspends a task; it does not normally block the runtime thread.
4. Retain futures returned by `std::async`; discarding them may cause unexpected waiting.
5. Use `std::promise` when some manually managed producer must complete one result.
6. Use `shared_future` only when multiple consumers need the same eventual result.
7. Move owned data into background work when possible; references require lifetime and synchronization discipline.
8. `std::async` is not a thread pool and is not a Tokio equivalent.
9. Use `std::jthread` for long-running, cooperatively stoppable workers.
10. C++20 coroutines are building blocks, not a complete standard async runtime.

## Final mental model

```text
std::async              start or defer a result-producing function
std::future<T>          handle to one eventual T
future.get()            block, retrieve, and consume the result
std::promise<T>         manually provide one future's result
std::shared_future<T>   share one eventual result with many consumers
std::jthread            RAII-managed OS thread with cooperative stop support
C++ coroutine           mechanism from which async libraries build task systems
Rust .await             suspend a Future-based task without blocking its executor thread
```
