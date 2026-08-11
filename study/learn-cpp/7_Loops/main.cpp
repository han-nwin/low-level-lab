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
    for (const int &x : values) {
        sum += x;
    }
    std::println("sum: {}", sum);

    std::vector<float> sets{1.4f, 2.2f, 3.9f, 4.12f};
    float sum_f(0.0f);
    for (float &x : sets) {
        sum_f += x;
    }
    std::println("sum_f = {:.2f}", sum_f);
}
