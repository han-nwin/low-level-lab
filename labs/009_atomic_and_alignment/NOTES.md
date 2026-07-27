
## Lab: Atomic Counters, False Sharing, and Cache Alignment

Build a multithreaded Rust benchmark that proves three things:

1. Normal integers cannot be safely shared between threads.
2. Atomics make shared updates correct.
3. Two correct atomic counters can still be slow when they share a cache line.

# Goal

Create a program where two threads repeatedly increment separate counters inside one memory space (aka inside a struct).

```
struct Counter {
  counter_a: usize,
  counter_b: usize
}
```

You will compare:

```text
Version 1: Mutex-protected counters
Version 2: Adjacent atomic counters
Version 3: Cache-aligned atomic counters
```

Expected lesson:

```text
Correctness:
Mutex        ✅
Atomics      ✅
Aligned      ✅

Performance:
Mutex        usually slowest
Atomics      faster, but may suffer false sharing
Aligned      often fastest when threads update separate counters
```

Performance results depend on your CPU, so the important part is measuring and explaining the difference.

# CLI design

```bash
cargo run -- mutex --iterations 100000000
cargo run -- atomic --iterations 100000000
cargo run -- aligned --iterations 100000000
```

Example output:

```text
mode:           atomic
iterations:     100000000 per thread
counter-a:      100000000
counter-b:      100000000
elapsed:        1.42 seconds
operations/sec: 140845070
```

# Project structure

```text
009_atomic_alignment/
├── Cargo.toml
└── src/
    ├── main.rs
    ├── mutex_version.rs
    ├── atomic_version.rs
    └── aligned_version.rs
```

