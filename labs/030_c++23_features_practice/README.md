# Lab 030: C++23 features practice

Continue lab 029 by implementing six exercises in `main.cpp`. Each includes
requirements, examples, edge cases, and an explanation prompt. Leave the test
harness intact. Exercise 3 depends on exercise 2.

1. Collect a lazy sensor pipeline with `std::ranges::to`.
2. Parse configuration using `std::expected` and `std::unexpected`.
3. Compose error handling with `transform_error`, `and_then`, and `transform`.
4. Chain `std::optional` validation, transformation, and fallback.
5. Compress consecutive states with `std::views::chunk_by`.
6. Decode network bytes using `std::byteswap` and `std::expected`.

```sh
c++ -std=c++23 -Wall -Wextra -pedantic main.cpp -o practice
./practice
```

The starter compiles and reports TODO for every case. Exit status 1 is expected
until all exercises pass; status 0 means all tests passed. Tests check behavior;
review your code against each exercise's required features as well.

The starter was checked with Apple clang 21.0.0 and its bundled libc++. Both the
compiler and standard library need the requested C++23 features; `-std=c++23`
alone cannot add missing library support. Feature checks at the top of the file
identify missing facilities. `compile_flags.txt` supplies editor compiler flags.

Background from the C++ committee:

- [Range-to-container conversion (P1206R7)](https://www.open-std.org/jtc1/sc22/wg21/docs/papers/2022/p1206r7.pdf)
- [Expected chaining operations (P2505R5)](https://www.open-std.org/jtc1/sc22/wg21/docs/papers/2022/p2505r5.html)
- [Optional chaining operations (P0798R8)](https://www.open-std.org/jtc1/sc22/wg21/docs/papers/2021/p0798r8.html)
