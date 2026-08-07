#include <iostream>
#include <string>
#include <type_traits>

int main() {
    // A declaration gives a name a type. An initializer supplies its first
    // value.
    int age{30};                 // signed integer
    double temperature{72.5};    // floating-point number
    char grade{'A'};             // one character
    bool is_learning{true};      // true or false
    std::string language{"C++"}; // owning, growable text

    // auto asks the compiler to infer the type from the initializer.
    auto lesson_count = 27; // inferred as int
    static_assert(std::is_same_v<decltype(lesson_count), int>);

    // Variables can change unless declared const.
    age += 1;

    std::cout << language << ": age=" << age << ", temperature=" << temperature
              << ", grade=" << grade << ", learning=" << std::boolalpha
              << is_learning << ", lessons=" << lesson_count << '\n';
}
