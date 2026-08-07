#include <algorithm>
#include <iostream>
#include <ranges>
#include <vector>

int main() {
    // NOTE: std::ranges was introduced in C++20. Range algorithms accept a
    // whole range, so begin/end iterators do not need to be passed separately.
    std::vector<int> numbers{9, 2, 7, 4, 1, 8, 3, 6, 5};

    std::ranges::sort(numbers);

    const auto found = std::ranges::find(numbers, 7);
    if (found != numbers.end()) {
        std::cout << "found " << *found << '\n';
    }

    // Views are lazy: they describe work but do not create a new container.
    // Values are filtered and transformed only as the view is iterated.
    auto even_squares =
        numbers | std::views::filter([](int value) { return value % 2 == 0; }) |
        std::views::transform([](int value) { return value * value; });

    std::cout << "even squares:";
    for (const int value : even_squares) {
        std::cout << ' ' << value;
    }
    std::cout << '\n';

    // Adaptors can be piped together. take(3) exposes at most the first 3
    // values.
    auto first_three = numbers | std::views::take(3);
    std::cout << "first three:";
    for (const int value : first_three) {
        std::cout << ' ' << value;
    }
    std::cout << '\n';

    // Projections let algorithms compare a selected part of each element.
    struct Student {
        const char *name;
        int score;
    };

    std::vector<Student> students{{"Lin", 82}, {"Maya", 95}, {"Sam", 76}};
    std::ranges::sort(students, std::ranges::greater{}, &Student::score);

    std::cout << "scores high to low:";
    for (const auto &student : students) {
        std::cout << ' ' << student.name << '=' << student.score;
    }
    std::cout << '\n';

    // Most views borrow from their source. Do not return or keep a view after
    // the container it refers to has been destroyed. A view also does not own
    // elements.
}
