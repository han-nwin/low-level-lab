# Chapter 17: Async Rust — Short Notes

Based on [The Rust Programming Language, Chapter 17](https://doc.rust-lang.org/book/ch17-00-async-await.html) and the examples in this directory.

## Mental model

- **Concurrency** means making progress on several jobs by switching between them. **Parallelism** means doing jobs at the same instant, usually on multiple CPU cores.
- A **future** is a lazy value representing work that may finish later. Creating or calling one does nothing until it is polled, normally through `.await`, `join`, `select`, or a spawned task.
- `async fn foo() -> T` is roughly `fn foo() -> impl Future<Output = T>`. The compiler turns the function into a state machine that stores what must survive across `.await` points.
- `.await` may pause the current future and return control to the runtime if the awaited future is pending. Code between await points is ordinary synchronous code.
- Rust supplies the `Future` abstraction but not a built-in async runtime. A runtime/executor such as Tokio polls futures and schedules tasks. `trpl::block_on` starts a Tokio-backed runtime for the book examples.
- **Future → task → runtime/executor → OS threads**: a future is a unit of async work; a task drives one or more futures; the runtime schedules tasks, often across a thread pool.

## Async as a state machine

The compiler turns each async function or block into a hidden state-machine type. Roughly, every `.await` creates a state containing the local variables needed when execution resumes:

```text
Start → waiting for request → waiting for body → Complete(result)
```

Polling the future runs it from its current state until it either:

- finishes and returns `Poll::Ready(result)`, or
- reaches an operation that is not ready, saves its state, and returns `Poll::Pending`.

The awaited operation later wakes the task, and the executor polls the future again. Execution resumes from the saved state rather than restarting the function. Because the future may store references between its own fields across states, moving it after polling can be unsafe; this is why low-level future APIs use `Pin`.

## Running futures together

| Tool | Meaning |
| --- | --- |
| `future.await` | Wait for one future before continuing; sequential when repeated one after another. |
| `join(a, b).await` | Drive both concurrently and wait for **both** outputs. The book's `trpl::join` is fair. |
| `join!(a, b, ...)` | Drive a compile-time-known number of futures and wait for all. |
| `join_all(futures).await` | Drive a runtime-sized collection and wait for all. |
| `select(a, b).await` | Race two futures; return the first output and cancel/drop the loser. `trpl::select` polls the left one first. |
| `spawn_task(future)` | Start an independently scheduled task; await its handle if it must finish before shutdown. |

Putting two operations in one `async` block does **not** make them concurrent: that block runs in source order. Put them in separate futures and drive them with `join`, `select`, or tasks.

## Ownership and channels

- `async move` moves captured values into a future. This is useful for spawned work and for ensuring resources are dropped when the future finishes.
- An async receiver commonly uses `while let Some(value) = rx.recv().await`.
- A channel returns `None` only after **every sender is dropped**. A forgotten sender keeps the receiver waiting forever.
- Clone a sender for multiple producers, move each clone into its producer future, and run producers plus receiver together.

## Cooperation and blocking

- Async scheduling is cooperative. A long section with no pending `.await` can starve other futures.
- `yield_now().await` voluntarily gives the executor another scheduling opportunity.
- Never assume adding `.await` makes blocking work nonblocking. `std::thread::sleep`, heavy CPU loops, and blocking file/network APIs still occupy the executor thread.
- Prefer runtime-provided async I/O/timers. Put CPU-heavy or unavoidable blocking work on dedicated threads or a runtime facility such as `spawn_blocking`.
- Measure before adding many yields: yielding has overhead, and CPU-bound work is usually better handled with threads.

## `Future`, waking, and pinning

The essential trait is:

```rust
trait Future {
    type Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>)
        -> Poll<Self::Output>;
}
```

- `Poll::Ready(value)` means the result is finished. `Poll::Pending` means “not ready; poll me again after I wake the task.” Most completed futures must not be polled again.
- `Context` contains a `Waker`. A pending future arranges for the waker to be called when progress becomes possible; the executor then knows to poll it again.
- The custom `Delay` here calls `wake_by_ref()` on every pending poll, so it busy-polls until time expires. A real timer registers the waker and wakes only when the deadline is reached.
- Compiler-generated futures may hold references to their own stored state. Moving such a future after polling could invalidate those references.
- `Pin<P>` prevents moving the pointee when that guarantee matters. `Unpin` is the marker saying a type remains safe to move even through a pinned pointer; most normal types are `Unpin`, but async-block futures often are not.
- Different async blocks have different anonymous types even when all output `()`. To store them together, erase their types with `dyn Future` and pin them, for example `Vec<Pin<&mut dyn Future<Output = ()>>>` or heap-owned `Pin<Box<dyn Future<Output = ()>>>`.

Most application code only uses `.await`; direct polling and pinning matter mainly in async libraries, executors, and heterogeneous future collections.

## Streams

- A `Future` produces one eventual output. A `Stream` produces zero or more asynchronous items over time—an async counterpart to `Iterator`.
- Low-level shape: `poll_next(...) -> Poll<Option<Item>>`. `Pending` means no item is ready yet; `Ready(Some(item))` yields an item; `Ready(None)` ends the stream.
- Import `StreamExt` to get convenient methods such as `stream.next().await`.
- Streams model channels, network events, incremental file chunks, and other values arriving over time.

## Choosing threads or async

- Prefer **async** for many I/O-bound operations that spend most of their time waiting.
- Prefer **threads/parallel workers** for CPU-bound work that can run simultaneously across cores.
- They complement each other: threads can perform blocking computation and send results through an async channel; multithreaded runtimes also move tasks among worker threads.

## Examples in this directory

- `hello-async`: fetch two page titles and use `select` to keep the first result.
- `concurrency_with_async`: spawned tasks, `join`, async channels, multiple producers, and `join!`.
- `yielding_control_to_runtime`: starvation, explicit yield points, cancellation with `select`, and a composed timeout future.
- `any_num_of_future`: demonstrates that blocking work before an await can dominate a race despite the directory name.
- `stream-future-in-sequence` / `stream-trait`: high-level stream iteration and the low-level `Stream`/`StreamExt` shapes.
- `trait-for-async`: sketches `Future`, `Poll`, and why an executor—not a busy manual loop—must coordinate polling.
- `pin-type-unpin-trait`: pins unlike future types, erases them behind `dyn Future`, and awaits the collection with `join_all`.
- `implement_a_future`: implements a minimal timer-like future and shows `Context`, `Waker`, and `Poll`.
- `fut_tasks_threads`: combines producer threads with an async receiver and experiments with Tokio broadcast channels.

Further reading: [Tokio async tutorial](https://tokio.rs/tokio/tutorial/async).
