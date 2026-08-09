#include <concepts>
#include <cstdint>
#include <print>

template <typename T>
concept Led = requires(T led) {
    { led.turn_on() };
    { led.turn_off() };
    { led.is_on() } -> std::same_as<bool>;
};

struct ConsoleLed {
    bool on;

    void turn_on() {
        if (!on) {
            on = true;
        }
    }

    void turn_off() {
        if (on) {
            on = false;
        }
    }

    bool is_on() { return on; }
};

struct MemoryLed {
    bool on;
    std::uint32_t toggle_count;

    void turn_on() {
        if (!on) {
            on = true;
            toggle_count++;
        }
    }

    void turn_off() {
        if (on) {
            on = false;
            toggle_count++;
        }
    }

    bool is_on() { return on; }
};

template <typename T>
    requires Led<T>
void blink_once(T &led) {
    led.turn_on();
    std::println("LED state: {}", led.is_on());

    led.turn_off();
    std::println("LED state: {}", led.is_on());
}

int main() {
    ConsoleLed console = ConsoleLed{.on = false};
    MemoryLed memory = MemoryLed{.on = false, .toggle_count = 0};

    blink_once(console);
    blink_once(memory);

    std::println("toggles: {}", memory.toggle_count);
}
