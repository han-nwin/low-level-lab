#include <iostream>

int main() {
    const int a{17};
    const int b{5};

    // Arithmetic: +, -, *, / and %. Integer division discards the fractional
    // part.
    std::cout << "a + b = " << a + b << '\n';
    std::cout << "a / b = " << a / b << ", remainder = " << a % b << '\n';

    int value{10};
    value += 3; // compound assignment; equivalent to value = value + 3
    ++value;    // increment before the value is used

    // Comparisons produce bool. Logical operators combine boolean expressions.
    const bool in_range = value >= 10 && value <= 20;
    std::cout << "value=" << value << ", in range=" << std::boolalpha
              << in_range << '\n';
}
