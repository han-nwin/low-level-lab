#include <cctype>
#include <cstring>
#include <iostream>

int main() {
    // A C string is a char array terminated by '\0'. Its capacity and current
    // string length are different concepts.
    char greeting[16]{"Hello"};
    std::strcat(greeting, " C++"); // safe here only because the array has room

    for (char *cursor = greeting; *cursor != '\0'; ++cursor) {
        // cctype functions require unsigned char values (or EOF).
        const auto ch = static_cast<unsigned char>(*cursor);
        *cursor = static_cast<char>(std::toupper(ch));
    }

    std::cout << greeting << ", length=" << std::strlen(greeting)
              << ", capacity=" << sizeof(greeting) << '\n';
    // Prefer std::string for general text; raw C-string functions cannot know
    // the destination capacity and are easy to misuse.
}
