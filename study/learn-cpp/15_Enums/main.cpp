#include <iostream>
#include <string_view>

// enum class keeps names scoped and does not implicitly convert to int.
enum class TrafficLight { red, yellow, green };

std::string_view describe(TrafficLight light) {
    switch (light) {
    case TrafficLight::red:
        return "stop";
    case TrafficLight::yellow:
        return "prepare";
    case TrafficLight::green:
        return "go";
    }
    return "unknown"; // defensive fallback for an invalid cast
}

int main() {
    const TrafficLight light{TrafficLight::green};
    std::cout << describe(light) << '\n';
    std::cout << "underlying value=" << static_cast<int>(light) << '\n';
}
