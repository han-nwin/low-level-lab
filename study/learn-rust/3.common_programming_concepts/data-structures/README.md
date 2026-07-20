# Common Rust data structures

Run all examples:

```sh
cargo run
```

Run the small tests:

```sh
cargo test
```

## Quick selection guide

| Need | Start with | Why |
|---|---|---|
| Fixed-size values of one type | `[T; N]` | Size is known at compile time |
| Borrowed view of sequential data | `&[T]` | Accepts arrays, vectors, and other slices without taking ownership |
| Small fixed group of different types | `(T, U, ...)` | Simple positional grouping; use a `struct` when names help |
| Growable ordered sequence | `Vec<T>` | Fast indexing and efficient operations at the end |
| Stack (last in, first out) | `Vec<T>` | Use `push` and `pop` |
| Queue or deque | `VecDeque<T>` | Efficient operations at both ends |
| Owned, growable UTF-8 text | `String` | Owns its text; borrow it as `&str` |
| Key-value lookup, order unimportant | `HashMap<K, V>` | Fast average lookup by key |
| Sorted key-value lookup or key ranges | `BTreeMap<K, V>` | Keeps keys ordered |
| Unique values, order unimportant | `HashSet<T>` | Fast average membership checks |
| Unique sorted values or ranges | `BTreeSet<T>` | Keeps values ordered |
| Always process highest/lowest priority | `BinaryHeap<T>` | Efficient priority queue |
| Linked nodes | `LinkedList<T>` | Specialized; usually prefer `Vec` or `VecDeque` |

## Ownership tips

- Passing `&[T]`, `&str`, or `&HashMap<K, V>` lets a function read a collection without taking it.
- Passing `&mut Vec<T>` (or another mutable reference) lets a function change it without taking it.
- Iterating with `for item in &collection` borrows items.
- Iterating with `for item in &mut collection` mutably borrows items.
- Iterating with `for item in collection` consumes the collection unless the collection is `Copy`.

## Performance at a glance

`n` is the number of elements. These are the usual/common costs; details can vary.

| Type / operation | Typical cost |
|---|---|
| `Vec` index | `O(1)` |
| `Vec` push/pop at back | amortized `O(1)` |
| `Vec` insert/remove at front or middle | `O(n)` |
| `VecDeque` push/pop at either end | amortized `O(1)` |
| `HashMap` / `HashSet` lookup | average `O(1)` |
| `BTreeMap` / `BTreeSet` lookup | `O(log n)` |
| `BinaryHeap` peek | `O(1)` |
| `BinaryHeap` push/pop | `O(log n)` |

The examples in [`src/main.rs`](src/main.rs) show each type in use. All of these are in Rust's standard library and require no external dependencies.
