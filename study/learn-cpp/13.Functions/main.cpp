#include <iostream>
#include <string>

// A declaration tells callers the signature. The definition supplies the body.
int add(int left, int right);

// Overloads share a name but have different parameter lists.
double add(double left, double right) { return left + right; }

// A default argument is used when the caller omits that argument.
std::string greet(const std::string &name,
                  const std::string &prefix = "Hello") {
    return prefix + ", " + name;
}

int main() {
    std::cout << add(2, 3) << ", " << add(2.5, 3.5) << '\n';
    std::cout << greet("C++ learner") << '\n';
}

int add(int left, int right) { return left + right; }
