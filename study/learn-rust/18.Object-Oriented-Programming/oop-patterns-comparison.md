# Classic OOP State Pattern vs. Rust Typestate Pattern

The key difference is where the post's state is enforced:

- `implement-oop`: state is checked and changed at runtime.
- `oop-the-rust-way`: state is represented by Rust types and enforced at compile time.

## `implement-oop`: the classic State pattern

There is one public `Post` type containing a dynamically selected state:

```rust
pub struct Post {
    state: Option<Box<dyn State>>,
    content: String,
}
```

The current state is an object implementing `State`: `Draft`, `PendingReview`, or `Published`. Calls such as `approve()` are dynamically dispatched to the current state object.

```text
Post
 |- content
 `- Box<dyn State>
      |- Draft
      |- PendingReview
      `- Published
```

Because every state implements the same `State` trait, every operation exists in every state. The implementation must therefore decide at runtime what an invalid operation does.

For example:

- Calling `request_review()` while published silently stays published.
- Calling `reject()` while in draft silently stays draft.
- Calling `content()` before publication returns an empty string.
- Calling `approve()` directly on a draft currently publishes it.

The transition methods take `self: Box<Self>` because changing state consumes the old state object and returns a new one:

```rust
fn approve(self: Box<Self>) -> Box<dyn State>;
```

`Option` allows `Post::state.take()` to temporarily move the state out of a mutably borrowed `Post`, consume it, and put the new state back.

### Advantages

- There is one stable `Post` type.
- States and transitions can be selected dynamically.
- State implementations fit behind the same `Box<dyn State>` interface.
- It is useful when state is known only at runtime or objects must be stored uniformly.

### Costs

- It uses heap allocation through `Box`.
- It uses dynamic dispatch through `dyn State`.
- Invalid operations remain available and require runtime handling.
- `Option` and `unwrap()` add implementation complexity.
- A user cannot determine a post's current state from its type.

## `oop-the-rust-way`: the typestate pattern

The Rust version uses a separate type for each state:

```rust
DraftPost
PendingReviewPost
Post // published
```

This is called the **typestate pattern**. An object's type encodes its current state.

Each type exposes only the operations that are valid for that state:

```text
DraftPost
 |- add_text()
 `- request_review() -> PendingReviewPost
                         `- approve() -> Post
                                          `- content()
```

Only `DraftPost` has `add_text()`, and only the published `Post` has `content()`. Therefore, this code cannot compile:

```rust,compile_fail
let post = Post::new(); // returns DraftPost
println!("{}", post.content()); // DraftPost has no content() method
```

There is no need to check at runtime whether the post is published. A `DraftPost` simply has no `content()` method.

State transitions consume `self`:

```rust
pub fn request_review(self) -> PendingReviewPost
pub fn approve(self) -> Post
```

Ownership ensures that the old state becomes unusable. After:

```rust
let post = post.request_review();
```

the original `DraftPost` has been moved, and `post` now holds a `PendingReviewPost`. Reusing the variable name is **shadowing**, not changing one object's runtime type.

### Advantages

- Invalid state transitions become compile-time errors.
- Public methods document which operations each state supports.
- It needs no trait object, heap allocation, dynamic dispatch, or `unwrap()`.
- State transitions are explicit in function return types.
- The workflow is often easier to reason about.

### Costs

- The concrete type changes after every transition.
- Different states cannot easily be placed in one homogeneous collection.
- Runtime-driven workflows may require an enum or trait object instead.
- Many states can make the API verbose.
- Adding transitions can affect several public types.

## Important behavioral difference

The two examples do not implement exactly the same workflow.

`implement-oop` requires **two approvals**:

```rust
if self.approve_count >= 2 {
    Box::new(Published {})
}
```

The typestate version requires only **one approval**:

```rust
pub fn approve(self) -> Post
```

It also does not implement rejection. The same two-approval workflow could be represented with additional typestate types:

```text
DraftPost
 `- request_review()
      v
PendingFirstApproval
 |- approve() -> PendingSecondApproval
 `- reject()  -> DraftPost
                  ^
PendingSecondApproval
 |- approve() -> Post
 `- reject()  -> DraftPost
```

Each workflow stage becomes a type, and each valid transition becomes a method that consumes the current value and returns the next type.

## Constructor naming

One naming oddity is that `Post::new()` returns `DraftPost` rather than `Self`, which is why the example suppresses Clippy's `new_ret_no_self` lint. `DraftPost::new()` or `Post::draft()` would communicate the return type more conventionally.

## Summary

The classic State pattern models **state as data hidden inside an object**. The Rust typestate pattern models **state as the object's type**.

Typestate uses Rust's ownership system and type checker to make illegal operations and transitions difficult—or impossible—to express. The dynamic State pattern is more flexible when state must be handled uniformly or chosen at runtime, but it moves more correctness checks into runtime logic.
