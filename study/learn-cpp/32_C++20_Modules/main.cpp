#include <iostream>

// import consumes the module's compiled interface instead of textually copying
// a header with #include. The module must be compiled before this source file.
import learn.math;

int main() {
    std::cout << "2 + 3 = " << learn::math::add(2, 3) << '\n';
    std::cout << "6 squared = " << learn::math::square(6) << '\n';
}

// Build with Clang from this chapter directory:
//
//   mkdir -p build
//   clang++ -std=c++20 math.cppm --precompile -o build/learn.math.pcm
//   clang++ -std=c++20 build/learn.math.pcm -c -o build/math.o
//   clang++ -std=c++20 main.cpp \
//     -fmodule-file=learn.math=build/learn.math.pcm \
//     build/math.o -o build/modules
//   ./build/modules
//
// learn.math.pcm is Clang's BMI (Built Module Interface). math.o contains
// machine code needed by the linker. BMI formats and module build commands are
// currently compiler-specific, so production projects should let CMake or
// another build system track module dependencies and choose the correct
// compilation order.
