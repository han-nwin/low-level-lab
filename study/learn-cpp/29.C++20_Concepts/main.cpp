#include <concepts>
#include <iostream>
#include <string>
#include <type_traits>

// Concepts, introduced in C++20, name requirements that a template argument
// must satisfy. They document template intent and usually produce clearer
// errors than an unconstrained template failing deep inside its implementation.

template <typename T>
concept Number = std::integral<T> || std::floating_point<T>;

// A requires-expression checks whether the listed operations are valid. This
// concept accepts a type that has size() returning something convertible to
// size_t.
template <typename T>
concept Sized = requires(const T &value) {
    { value.size() } -> std::convertible_to<std::size_t>;
};

// Style 1: place the concept directly in the template parameter list.
template <Number T> T add(T left, T right) { return left + right; }

// Style 2: add a requires-clause after an ordinary template parameter list.
template <typename T>
    requires std::integral<T>
bool is_even(T value) {
    return value % 2 == 0;
}

// Style 3: an abbreviated function template uses a concept with auto.
void print_size(const Sized auto &value) {
    std::cout << "size=" << value.size() << '\n';
}

template <typename T>
concept PrintableNumber =
    Number<T> && requires(std::ostream &output, T value) { output << value; };

template <PrintableNumber T> void print_number(T value) {
    std::cout << "number=" << value << '\n';
}

int main() {
    std::cout << "integer sum=" << add(2, 3) << '\n';
    std::cout << "floating sum=" << add(2.5, 1.25) << '\n';
    std::cout << "42 is even=" << std::boolalpha << is_even(42) << '\n';

    const std::string language{"C++20"};
    print_size(language);
    print_number(99);

    // These calls fail at the constraint boundary with a useful diagnostic:
    // add(std::string{"a"}, std::string{"b"}); // string does not satisfy
    // Number is_even(2.5);                            // double is not
    // std::integral

    static_assert(Number<int>);
    static_assert(Number<double>);
    static_assert(!Number<std::string>);
    static_assert(Sized<std::string>);
}
