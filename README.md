# Low-Level Lab

A long-term repository documenting my journey into low-level software engineering.

This repo contains:

- 📚 Solutions to book exercises
- 🔍 Notes from reading production source code (Linux, etc.)
- 🛠️ Experiments and implementations of low-level concepts
- 📈 Performance benchmarks and profiling

## References
Linux Source Code: <https://github.com/torvalds/linux>

## Structure 
``` text
study/ - any book or source study
labs/ - any implementation for practice
```
## Goals

- Understand how software interacts with hardware.
- Learn from production code instead of toy examples.
- Build intuition for systems programming and performance.
- Continuously grow this repository as I explore kernels, compilers, runtimes, and computer architecture.

## Knowledge Progress

Checked means I have implemented or demonstrated the concept in this repository,
not merely read about it.

### Embedded systems

- [x] Binary representation, bitwise operations, masks, shifts, and register fields
- [x] Bare-metal Rust program with `no_std`, `no_main`, a linker script, and target metadata
- [x] Volatile memory-mapped register reads and writes
- [x] GPIO output and input using raw registers
- [x] Reset controller and peripheral reset sequencing
- [x] Clock tree, XOSC, PLL configuration, and clock measurement
- [x] Polling SPI controller and communicating with an external peripheral
- [x] Polling I2C controller and communicating with an external peripheral
- [ ] UART transmit, receive, baud-rate configuration, status flags, and error handling (`016` — current)
- [ ] Hardware timers, alarms, deadlines, and monotonic time (`017`)
- [ ] Interrupt vector table, NVIC, ISR design, and interrupt acknowledgement (`018`)
- [ ] Safe shared state between main code and an ISR
- [ ] ADC sampling, conversion timing, calibration, and noise (`019`)
- [ ] PWM frequency, duty cycle, slices/channels, and waveform verification (`020`)
- [ ] DMA configuration, buffer ownership, peripheral transfers, and completion interrupts (`021`)
- [ ] Convert a polling peripheral driver into an interrupt-driven state machine
- [ ] Build a safe driver API around a small, isolated MMIO/`unsafe` layer
- [ ] Model peripheral states and exclusive ownership with Rust types
- [ ] Implement timeouts, peripheral errors, and recovery rather than infinite polling
- [ ] USB device fundamentals and endpoints (`022`)
- [ ] Programmable I/O/state machines (`023`)
- [ ] Combine timers, interrupts, DMA, and a peripheral in one embedded system
- [ ] Understand how an embedded async executor connects futures, wakers, timers, and interrupts
- [ ] Implement the same embedded application with polling, interrupts, and async and compare them

### C++

- [x] Core syntax, control flow, functions, references, pointers, and const correctness
- [x] Classes, constructors/destructors, operator overloading, and object lifetime basics
- [x] RAII and `unique_ptr`/`shared_ptr` ownership
- [x] Move semantics and move-only objects
- [x] Templates, concepts, lambdas, and type deduction
- [x] Threads, mutexes, semaphores, and condition variables
- [x] Futures, promises, `std::async`, and cooperative cancellation concepts
- [x] Basic C++20 coroutine mechanics and Boost.Asio awaitables
- [x] Implement a Rust-style MPSC channel with C++ synchronization primitives
- [ ] Organize a substantial program into interfaces, implementation files, libraries, and tests
- [ ] Use target-based CMake for multiple libraries, executables, tests, and build configurations
- [ ] Understand translation units, linkage, symbols, templates across files, ODR, and ABI boundaries
- [ ] Deepen Rule of Zero/Five, exception safety, `noexcept`, and resource ownership
- [ ] Understand object layout, padding, alignment, vtables, and cache-line effects
- [ ] Use `span`, `string_view`, `variant`, `optional`, ranges, and algorithms effectively
- [ ] Understand allocators, placement `new`, memory pools, and `std::pmr`
- [ ] Understand the C++ memory model and acquire/release atomics
- [ ] Implement compare-exchange loops and reason about their memory ordering
- [ ] Add bounded capacity, backpressure, shutdown, and move-only messages to the C++ channel
- [ ] Build a thread pool with cancellation, error propagation, and clean shutdown
- [ ] Build a coroutine task type or small executor to understand coroutine scheduling
- [ ] Use AddressSanitizer, UndefinedBehaviorSanitizer, and ThreadSanitizer to diagnose real bugs
- [ ] Use C++ for bare-metal MMIO and compare its safety and zero-cost abstractions with Rust
- [ ] Reimplement a substantial Rust systems lab in C++ and compare both designs

### Rust

- [x] Ownership, borrowing, lifetimes, traits, generics, and smart pointers
- [x] Threads, mutexes, channels, `Send`, and `Sync` fundamentals
- [x] Async functions, futures, tasks, executors, streams, pinning, and wakers
- [x] Tokio networking and combining async I/O with parallel CPU work
- [x] Basic unsafe Rust and volatile hardware access
- [ ] Design reusable `no_std` libraries instead of single-file firmware programs
- [ ] Isolate unsafe code and document each safety invariant
- [ ] Deepen pointer provenance, aliasing, interior mutability, and unsafe abstractions
- [ ] Test unsafe code with Miri where applicable
- [ ] Implement a minimal executor to connect `Future::poll`, `Waker`, and task scheduling
- [ ] Use embedded async after understanding its timer and interrupt integration
- [ ] Build a Rust/C/C++ FFI boundary with explicit ownership and error rules

### Concurrency and async

- [x] Distinguish concurrency, parallelism, blocking, and asynchronous I/O
- [x] Compare mutexes, atomics, and cache-line alignment
- [x] Use channels and condition variables for producer/consumer communication
- [ ] Explain happens-before relationships in concrete programs
- [ ] Understand relaxed, acquire, release, acquire-release, and sequentially consistent ordering
- [ ] Handle spurious wakeups, lost-wakeup risks, and shutdown races
- [ ] Build a bounded blocking queue in both Rust and C++
- [ ] Add cancellation, deadlines, backpressure, and graceful shutdown to async/concurrent systems
- [ ] Diagnose a deliberate data race and deadlock with tooling
- [ ] Understand work stealing, task scheduling, fairness, starvation, and priority inversion
- [ ] Understand lock-free progress guarantees and the ABA/reclamation problems
- [ ] Implement a lock-free structure only after mastering memory ordering and measurement

### Systems programming and OS

- [x] Basic inspection with `file`, `size`, `strings`, `nm`, `readelf`, and `objdump`
- [x] Basic process, socket, syscall-tracing, debugging, and profiling tools
- [x] Build a TCP server and parse HTTP without a web framework
- [ ] Processes, `fork`, `exec`, exit status, pipes, redirection, and signals
- [ ] File descriptors, blocking modes, readiness, and partial I/O
- [ ] Build a nonblocking event-loop server using `poll`, `epoll`, or `kqueue`
- [ ] Virtual address spaces, pages, page faults, and `mmap`
- [ ] Implement a small allocator and measure fragmentation and allocation cost
- [ ] ELF sections, symbols, relocations, static/dynamic linking, and program loading
- [ ] Scheduling, context switching, system calls, and user/kernel transitions
- [ ] Filesystem concepts: inodes, directories, caching, journaling, and memory-mapped files
- [ ] Kernel synchronization, interrupt context, and device-driver architecture
- [ ] Build a small shell, userspace filesystem, kernel module, or teaching-OS component

### Low latency and performance

- [x] Demonstrate false sharing and cache-line alignment effects
- [ ] Design trustworthy benchmarks with optimized builds, warm-up, repetition, and correct operation counts
- [ ] Measure latency distributions including median, p95, p99, and worst case
- [ ] Separate throughput, average latency, tail latency, and jitter
- [ ] Measure allocations, cache misses, branch misses, page faults, and context switches
- [ ] Understand CPU affinity, frequency scaling, preemption, and measurement noise
- [ ] Inspect generated assembly and connect source-level choices to machine behavior
- [ ] Profile before optimizing and preserve evidence for each performance claim
- [ ] Build a fixed-capacity/allocation-free hot path
- [ ] Explore data-oriented layout, batching, branch prediction, SIMD, and cache locality
- [ ] Measure an end-to-end concurrent pipeline and reduce its tail latency

### Cross-track projects

- [ ] Interrupt-driven embedded sensor/communication system in Rust
- [ ] Equivalent embedded component in modern C++
- [ ] Polling vs interrupt vs embedded-async comparison
- [ ] Bounded concurrent pipeline implemented in both Rust and C++
- [ ] Nonblocking network service with profiling and graceful shutdown
- [ ] Small OS or kernel component that applies memory, concurrency, and hardware concepts


git_branch() {
    git branch --show-current 2>/dev/null
}

PS1='\[\e[1;36m\]\u\[\e[0m\] \[\e[1;34m\]\w\[\e[0m\] \[\e[1;33m\]$(git_branch)\[\e[0m\]\n❯ '
