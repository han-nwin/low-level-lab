#include <concepts>
#include <cstdint>
#include <print>

template <typename T>
concept Led = requires(T led) {
    { led.turn_on() };
    { led.turn_off() };
    { led.is_on() } -> std::same_as<bool>;
};

template <typename T>
concept Dimmable = requires(T led, std::uint8_t value) {
    { led.set_brightness(value) } -> std::same_as<void>;
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
    std::uint8_t brightness;
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

    void set_brightness(std::uint8_t value) { brightness = value; }
};

template <typename T>
    requires Led<T>
void blink_once(T &led) {
    led.turn_on();
    std::println("LED state: {}", led.is_on());

    led.turn_off();
    std::println("LED state: {}", led.is_on());
}

template <typename T>
    requires Led<T> && Dimmable<T>
void test_dimmable(T &led, std::uint8_t value) {
    led.set_brightness(value);
}

int main() {
    ConsoleLed console = ConsoleLed{.on = false};
    MemoryLed memory = MemoryLed{.on = false, .toggle_count = 0};

    blink_once(console);
    blink_once(memory);

    std::println("toggles: {}", memory.toggle_count);

    test_dimmable(memory, 100);
    std::println("brightness: {}", memory.brightness);
}
