// ============================================================
// C++20 CONCEPTS — CHEATSHEET
// Rust mental model included
// ============================================================

#include <concepts>

// ------------------------------------------------------------
// 1. DEFINE A CONCEPT
//
// concept = named compile-time condition
//
// Rough Rust analogy:
//     trait Sensor { ... }
// ------------------------------------------------------------

template <typename T>
concept Sensor = requires(T sensor) { sensor.read(); };

// Think:
//
// Sensor<MyType>
//
// → true  if MyType has .read()
// → false otherwise

// ------------------------------------------------------------
// 2. REQUIRE A SPECIFIC RETURN TYPE
// ------------------------------------------------------------

template <typename T>
concept Sensor2 = requires(T sensor) {
    { sensor.read() } -> std::same_as<int>;
};

// Means:
//
// 1. sensor.read() must compile
// 2. it must return exactly int

// ------------------------------------------------------------
// 3. MULTIPLE REQUIREMENTS
// ------------------------------------------------------------

template <typename T>
concept Device = requires(T device) {
    { device.read() } -> std::same_as<int>;
    { device.write() } -> std::same_as<void>;
    { device.reset() } -> std::same_as<void>;
};

// ------------------------------------------------------------
// 4. USE A CONCEPT ON A TEMPLATE
//
// C++:
//     requires Sensor<T>
//
// Rust:
//     where T: Sensor
// ------------------------------------------------------------

template <typename T>
    requires Sensor<T>
void use_sensor(T &sensor) {
    sensor.read();
}

// Rust equivalent:
//
// fn use_sensor<T>(sensor: &mut T)
// where
//     T: Sensor,
// {
//     sensor.read();
// }

// ------------------------------------------------------------
// 5. SHORTER SYNTAX
// ------------------------------------------------------------

template <Sensor T> void use_sensor_short(T &sensor) { sensor.read(); }

// These:
//
// template<typename T>
// requires Sensor<T>
//
// and:
//
// template<Sensor T>
//
// basically express the same constraint.

// ------------------------------------------------------------
// 6. MULTIPLE CONCEPTS
// ------------------------------------------------------------

template <typename T>
concept Readable = requires(T x) { x.read(); };

template <typename T>
concept Writable = requires(T x) { x.write(); };

template <typename T>
    requires Readable<T> && Writable<T>
void use_device(T &device) {
    device.read();
    device.write();
}

// Rust:
//
// where T: Readable + Writable

// ------------------------------------------------------------
// 7. BUILD A CONCEPT FROM OTHER CONCEPTS
// ------------------------------------------------------------

template <typename T>
concept ReadWrite = Readable<T> && Writable<T>;

template <typename T>
    requires ReadWrite<T>
void foo(T &device) {
    // ...
}

// ------------------------------------------------------------
// 8. CONCEPTS CAN HAVE MULTIPLE TYPE PARAMETERS
// ------------------------------------------------------------

template <typename Bus, typename Data>
concept WritableTo = requires(Bus bus, Data data) { bus.write(data); };

// Ask:
//
// WritableTo<Spi, uint8_t>
//
// "Can Spi.write(uint8_t) compile?"

// ------------------------------------------------------------
// 9. USE EXISTING STANDARD CONCEPTS
// ------------------------------------------------------------

template <typename T>
    requires std::integral<T>
bool is_even(T value) {
    return value % 2 == 0;
}

// Common standard concepts:
//
// std::integral<T>
// std::floating_point<T>
// std::same_as<T, U>
// std::convertible_to<T, U>

// ------------------------------------------------------------
// 10. EXACT TYPE vs CONVERTIBLE TYPE
// ------------------------------------------------------------

template <typename T>
concept A = requires(T x) {
    { x.size() } -> std::same_as<std::size_t>;
};

// EXACTLY size_t

template <typename T>
concept B = requires(T x) {
    { x.size() } -> std::convertible_to<std::size_t>;
};

// Result can be converted to size_t

// ------------------------------------------------------------
// 11. ABBREVIATED SYNTAX
// ------------------------------------------------------------

void print_sensor(Sensor auto &sensor) { sensor.read(); }

// Roughly:
//
// template<typename T>
// requires Sensor<T>
// void print_sensor(T& sensor)

// ============================================================
// THE 3 THINGS TO REMEMBER
// ============================================================

// #1 — DEFINE what a valid type looks like:

template <typename T>
concept Sensor3 = requires(T x) { x.read(); };

// #2 — CONSTRAIN a generic function:

template <typename T>
    requires Sensor3<T>
void run(T &x) {
    x.read();
}

// #3 — A struct satisfies the concept automatically:

struct Rc522 {
    void read() {
        // ...
    }
};

// No:
//     impl Sensor for Rc522
//
// is needed in C++.
//
// If Rc522 has the required operations:
//
//     Sensor3<Rc522> == true

// ============================================================
// RUST ↔ C++ MENTAL MAP
// ============================================================
//
// RUST                          C++
// ------------------------------------------------------------
//
// trait Sensor                  concept Sensor
//
// impl Sensor for Rc522         Rc522 has matching methods
//                               (implicit satisfaction)
//
// T: Sensor                     requires Sensor<T>
//
// where T: Sensor              requires Sensor<T>
//
// Readable + Writable          Readable<T> && Writable<T>
//
// fn foo<T: Sensor>()          template<Sensor T>
//

// ============================================================
// MOST IMPORTANT DISTINCTION
// ============================================================

// DEFINE the rule:
//
// template<typename T>
// concept Sensor = requires(T x) {
//     x.read();
// };

// USE / APPLY the rule:
//
// template<typename T>
// requires Sensor<T>
// void foo(T& x);

// requires(T x) { ... }
//      ^
//      |
//      "Would these operations compile?"
//
//
// requires Sensor<T>
//      ^
//      |
//      "Only allow T if Sensor<T> is true."

// ============================================================
// ONE-LINE MEMORY TRICK
// ============================================================
//
// concept            = WHAT must T support?
//
// requires(T x) {...}= CAN T do these operations?
//
// requires Foo<T>    = ONLY allow T if Foo<T> is true
//
// C++ requires Foo<T> ≈ Rust where T: Foo
// ============================================================
