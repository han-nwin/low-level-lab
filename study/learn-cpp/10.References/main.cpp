#include <iostream>

void increment(int &value) { ++value; }              // may modify the caller
void print(const int &value) { std::cout << value; } // read-only, avoids a copy

int main() {
    int number{10};
    int &alias = number; // a reference is another name for an existing object

    alias = 20;
    increment(number);

    // References must be initialized and normally cannot be reseated. Unlike a
    // pointer, ordinary reference use needs no explicit dereference operator.
    std::cout << "number=";
    print(number);
    std::cout << ", alias=" << alias << '\n';
}
