#include <iostream>

// C and C++ commonly use separate compilation. A header exposes declarations;
// each .cpp plus its included headers forms a translation unit and produces one
// object file. The linker resolves symbols between object files and libraries.
//
// Typical diagnostics:
// - syntax/type error: compilation failed in a translation unit
// - undefined reference/symbol: declaration was seen, but no linked definition
// - multiple definition: the One Definition Rule was violated
//
// Optimization can inline, fold constants and remove dead work while preserving
// observable behavior. Debug builds favor diagnostics; release builds favor
// speed.

int meaning_of_life(); // declaration

int main() { std::cout << "linked answer=" << meaning_of_life() << '\n'; }

int meaning_of_life() { return 42; } // definition
