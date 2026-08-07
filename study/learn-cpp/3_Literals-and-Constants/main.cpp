#include <cstdint>
#include <iostream>

int main() {
    // Literals are values written directly in source code.
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

    static_assert(decimal == binary && binary == hexadecimal);
    std::cout << text << newline << "population=" << population << ", pi=" << pi
              << ", scientific=" << scientific << '\n';
}
