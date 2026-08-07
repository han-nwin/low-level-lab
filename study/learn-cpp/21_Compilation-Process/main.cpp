#include <iostream>

// Try each stage separately (the text after each command explains its output):
//   clang++ -std=c++20 -E main.cpp -o main.ii
//     Preprocess: expand directives such as #include and #define.
//   clang++ -std=c++20 -S main.ii -o main.s
//     Compile: translate preprocessed C++ into assembly.
//   clang++ -c main.s -o main.o
//     Assemble: translate assembly into an object file.
//   clang++ main.o -o compilation
//     Link: combine object files and libraries into an executable.
//
// A normal command performs all stages:
//   clang++ -std=c++20 main.cpp -o compilation

#define COURSE_NAME "learn-cpp" // replaced during preprocessing

int square(int value) { return value * value; }

int main() {
    // The declaration for cout comes from the expanded iostream header.
    // square() becomes machine instructions and the linker connects
    // standard-library code.
    std::cout << COURSE_NAME << ": square(6)=" << square(6) << '\n';
}
