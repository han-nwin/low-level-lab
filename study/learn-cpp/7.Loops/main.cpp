#include <iostream>
#include <vector>

int main() {
    for (int i = 0; i < 3; ++i) {
        std::cout << "for: " << i << '\n';
    }

    int countdown{3};
    while (countdown > 0) {
        std::cout << "while: " << countdown-- << '\n';
    }

    // A range-based for loop visits each element. Use const auto& to avoid
    // copies.
    const std::vector<int> values{2, 4, 6, 8};
    int sum{0};
    for (const auto &value : values) {
        if (value == 6)
            continue;
        sum += value;
    }
    std::cout << "sum (excluding 6)=" << sum << '\n';
}
