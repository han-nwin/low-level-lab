#include <iostream>
#include <string>
#include <string_view>

void inspect(std::string_view text) {
    // string_view is a non-owning view. It must not outlive the viewed
    // characters.
    std::cout << '"' << text << "\" has " << text.size() << " characters\n";
}

int main() {
    std::string message{"Learn"};
    message += " C++";
    message.append(" step by step");

    const auto position = message.find("C++");
    if (position != std::string::npos)
        message.replace(position, 3, "modern C++");

    inspect(message);
    std::cout << "substring=" << message.substr(0, 5)
              << ", capacity=" << message.capacity() << '\n';
}
