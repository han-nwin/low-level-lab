#include <iostream>
#include <string>

int main() {
    // Widening conversions are usually safe and may happen implicitly.
    const int whole{7};
    const double widened = whole;

    // Make narrowing intent explicit. The fractional part is discarded.
    const double price{19.95};
    const int dollars = static_cast<int>(price);

    // Text conversion uses standard helpers. stoi can throw on invalid input.
    const std::string digits{"123"};
    const int parsed = std::stoi(digits);
    const std::string formatted = std::to_string(parsed + 1);

    std::cout << "widened=" << widened << ", dollars=" << dollars
              << ", formatted=" << formatted << '\n';
}
