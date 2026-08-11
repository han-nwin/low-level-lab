#include <cstdint>
#include <iostream>

int get_value() { return 99; }

int main() {
    // Literals are values written directly in source code.
    // const means the value cannot be changed after initialization
    const int decimal = 42;
    const int binary = 0b101010;
    const int hexadecimal = 0x2A;
    const double scientific = 6.022e23;
    const char newline = '\n';
    const char *text = "string literal";

    // Digit separators improve readability and do not change the value.
    constexpr std::int64_t population{8'000'000'000LL};
    // constexpr requires a value that can be evaluated at compile time.
    constexpr double pi{3.141592653589793};

    const int val1 = get_value();
    // This can't be determined at compile time -> not allowed
    // constexpr int val2 = get_value();

    static_assert(decimal == binary && binary == hexadecimal);
    std::cout << text << newline << "population=" << population << ", pi=" << pi
              << ", scientific=" << scientific << '\n';
    std::println("val1 = {}", val1);
}
