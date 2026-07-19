#include <iostream>

// C++ is a compiled, statically typed language. Source code is translated into
// machine code before it runs. The usual path is:
// source (.cpp) -> preprocessing -> compilation -> assembly -> linking ->
// program.
//
// Build this file:
//   clang++ -std=c++17 -Wall -Wextra -pedantic main.cpp -o introduction
// Run it:
//   ./introduction

int main() {
    // main is the program's entry point. Returning zero means success.
    std::cout << "Hello, C++!\n";
    std::cout << "A statement normally ends with a semicolon.\n";
    return 0;
}
