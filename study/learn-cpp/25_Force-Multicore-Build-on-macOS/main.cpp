#include <iostream>

// Compilers generally compile one translation unit per process. Build tools
// gain parallelism by compiling independent .cpp files at the same time.
//
// CMake (portable):
//   cmake --build build --parallel
//   cmake --build build --parallel 8
//
// Unix Makefiles on macOS:
//   make -j"$(sysctl -n hw.logicalcpu)"
//
// Ninja uses parallel jobs by default:
//   cmake -S . -B build -G Ninja
//   cmake --build build
//
// Xcode generator:
//   cmake --build build -- -parallelizeTargets -jobs 8
// More jobs are not always faster when memory, I/O, or a single large source
// file is the bottleneck. Start near the number of logical CPUs and measure.

int main() {
    std::cout << "Parallel builds schedule independent compile commands.\n";
}
