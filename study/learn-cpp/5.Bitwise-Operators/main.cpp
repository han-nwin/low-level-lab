#include <bitset>
#include <cstdint>
#include <iostream>

int main() {
    const std::uint8_t left{0b1100};
    const std::uint8_t right{0b1010};

    // &, | and ^ operate on corresponding bits; ~value flips every bit in
    // value. This unary ~ operator is unrelated to ~ClassName(), the destructor
    // syntax.
    std::cout << "AND " << std::bitset<8>(left & right) << '\n';
    std::cout << "OR  " << std::bitset<8>(left | right) << '\n';
    std::cout << "XOR " << std::bitset<8>(left ^ right) << '\n';
    std::cout << "NOT " << std::bitset<8>(~left) << '\n';

    // Shifts move bits. For unsigned values, left shift by one often doubles
    // and right shift by one often halves (provided no significant bits are
    // lost).
    std::cout << "left shift  " << std::bitset<8>(left << 1) << '\n';
    std::cout << "right shift " << std::bitset<8>(left >> 1) << '\n';
}
