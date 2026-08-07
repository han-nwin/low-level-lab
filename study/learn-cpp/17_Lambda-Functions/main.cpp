#include <algorithm>
#include <iostream>
#include <vector>

int main() {
    int minimum{5};
    std::vector<int> numbers{8, 1, 6, 3, 9};

    // [minimum] captures a copy. (int value) is the parameter list. The return
    // type is inferred. Use [&name] to capture an object by reference instead.
    const auto is_large = [minimum](int value) { return value >= minimum; };
    const auto count = std::count_if(numbers.begin(), numbers.end(), is_large);

    std::sort(numbers.begin(), numbers.end(),
              [](int left, int right) { return left > right; });

    std::cout << count << " values are at least " << minimum << ":";
    for (const int number : numbers)
        std::cout << ' ' << number;
    std::cout << '\n';
}
