#include <array>
#include <iostream>
#include <print>

int main() {
    // A built-in array has a fixed size and does not know its own length
    // directly.
    int raw[3]{10, 20, 30};
    raw[1] = 25;
    for (int &n : raw) {
        std::cout << n << std::endl;
    }

    // std::array is fixed-size too, but supplies size(), iterators and bounds
    // checks.
    std::array<int, 4> numbers{1, 2, 3, 4};
    numbers.at(2) = 30; // at() throws if the index is outside the array
    numbers[6] = 30;    // NOTE: this won't throw -> use at() is safer

    int total{0};
    for (const int &number : numbers)
        total += number;

    // Indexing starts at zero. [] does not perform bounds checking.
    std::cout << "raw[1]=" << raw[1] << ", first=" << numbers.front()
              << ", size=" << numbers.size() << ", total=" << total << '\n';
}
