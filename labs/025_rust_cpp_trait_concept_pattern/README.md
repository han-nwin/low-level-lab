Do the same tiny driver API twice: once in Rust, once in C++. The goal is to practice **structs, trait/concept constraints, generic functions, and multiple implementations** without hardware complexity.

## Lab: Generic LED Controller

You have two fake LEDs:

```text
ConsoleLed
MemoryLed
```

Both should support:

```text
turn_on()
turn_off()
is_on()
```

Then write one generic function:

```text
blink_once(...)
```

that works with **any LED implementation**.

### Part 1 — Rust requirements

Define:

```rust
trait Led {
    fn turn_on(&mut self);
    fn turn_off(&mut self);
    fn is_on(&self) -> bool;
}
```

Create:

```rust
struct ConsoleLed {
    on: bool,
}
```

and:

```rust
struct MemoryLed {
    on: bool,
    toggle_count: u32,
}
```

Implement `Led` for both.

`MemoryLed` should increment `toggle_count` every time its state changes.

Then write:

```rust
fn blink_once<L: Led>(led: &mut L) {
    led.turn_on();
    println!("LED state: {}", led.is_on());

    led.turn_off();
    println!("LED state: {}", led.is_on());
}
```

Your `main()` should roughly do:

```rust
fn main() {
    let mut console = ConsoleLed { on: false };

    let mut memory = MemoryLed {
        on: false,
        toggle_count: 0,
    };

    blink_once(&mut console);
    blink_once(&mut memory);

    println!("toggles: {}", memory.toggle_count);
}
```

Expected idea:

```text
LED state: true
LED state: false

LED state: true
LED state: false

toggles: 2
```

---

## Part 2 — C++ requirements

Do **not** use inheritance or `virtual`.

Define a concept:

```cpp
template<typename T>
concept Led = requires(T led) {
    // you fill this in
};
```

It must verify that:

```cpp
led.turn_on()
```

returns `void`,

```cpp
led.turn_off()
```

returns `void`,

and:

```cpp
led.is_on()
```

returns `bool`.

Then create:

```cpp
struct ConsoleLed {
    bool on;
};
```

and:

```cpp
struct MemoryLed {
    bool on;
    unsigned toggle_count;
};
```

Give both the required methods.

Then write the generic function using the **long form first**:

```cpp
template<typename T>
requires Led<T>
void blink_once(T& led)
{
    led.turn_on();
    std::println("LED state: {}", led.is_on());

    led.turn_off();
    std::println("LED state: {}", led.is_on());
}
```

After it works, rewrite it using the shorter syntax:

```cpp
template<Led T>
void blink_once(T& led)
{
    // ...
}
```

### Your C++ concept should eventually look roughly like

Don't copy this immediately—try it first:

```cpp
template<typename T>
concept Led = requires(T led) {
    { led.turn_on() } -> std::same_as<void>;
    { led.turn_off() } -> std::same_as<void>;
    { led.is_on() } -> std::same_as<bool>;
};
```

You'll need:

```cpp
#include <concepts>
```

and if you're using C++23:

```cpp
#include <print>
```

---

## Bonus challenge

After both versions work, add a second capability:

```text
Dimmable
```

with:

```text
set_brightness(0..100)
```

Only `MemoryLed` should support it.

Rust:

```rust
trait Dimmable {
    fn set_brightness(&mut self, value: u8);
}
```

C++:

```cpp
template<typename T>
concept Dimmable = requires(T led, unsigned value) {
    // ...
};
```

Then write a function requiring **both capabilities**.

Rust:

```rust
fn test_dimmable<L>(led: &mut L)
where
    L: Led + Dimmable,
{
    // ...
}
```

C++:

```cpp
template<typename T>
requires Led<T> && Dimmable<T>
void test_dimmable(T& led)
{
    // ...
}
```

That bonus is the important part because it'll make the relationship click:

```text
Rust
Led + Dimmable

C++
Led<T> && Dimmable<T>
```

