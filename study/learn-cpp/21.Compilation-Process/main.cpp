#include <iostream>

// Try each stage separately:
//   clang++ -E main.cpp -o main.ii          # preprocess: expand
//   #include/#define clang++ -std=c++17 -S main.ii -o main.s # compile: C++ ->
//   assembly clang++ -c main.s -o main.o             # assemble: assembly ->
//   object file clang++ main.o -o compilation           # link:
//   objects/libraries -> executable
//
// A normal command performs all stages:
//   clang++ -std=c++17 main.cpp -o compilation

#define COURSE_NAME "learn-cpp" // replaced during preprocessing

int square(int value) { return value * value; }

int main() {
    // The declaration for cout comes from the expanded iostream header.
    // square() becomes machine instructions and the linker connects
    // standard-library code.
    std::cout << COURSE_NAME << ": square(6)=" << square(6) << '\n';
}
