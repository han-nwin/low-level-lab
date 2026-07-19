#include <iostream>

// A minimal CMakeLists.txt beside a source file would contain:
//
//   cmake_minimum_required(VERSION 3.20)
//   project(cmake_lesson LANGUAGES CXX)
//   add_executable(cmake_lesson main.cpp)
//   target_compile_features(cmake_lesson PRIVATE cxx_std_17)
//   target_compile_options(cmake_lesson PRIVATE -Wall -Wextra -Wpedantic)
//
// Configure and build out-of-source:
//   cmake -S . -B build
//   cmake --build build
//   ./build/cmake_lesson
// Targets describe requirements and dependencies; avoid global flags when a
// target_* command can express the same information.

int main() {
    std::cout
        << "CMake generates build files; the compiler still builds C++.\n";
}
