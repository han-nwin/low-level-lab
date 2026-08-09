#include <bitset>
#include <cstdint>
#include <print>
#include <string_view>

using Byte = std::uint8_t;

void print_bits(std::string_view label, Byte value) {
    std::println("{:<20} {} ({:#04x})", label,
                 std::bitset<8>{value}.to_string(),
                 static_cast<unsigned int>(value));
}

int main() {
    const Byte left{0b0000'1100};
    const Byte right{0b0000'1010};

    std::println("Basic bitwise operators");
    print_bits("left", left);
    print_bits("right", right);

    // &, | and ^ operate on corresponding bits. ~ flips every bit. The cast
    // back to Byte is needed because small integer types are promoted to int
    // before arithmetic and bitwise operations.
    print_bits("left & right", static_cast<Byte>(left & right));
    print_bits("left | right", static_cast<Byte>(left | right));
    print_bits("left ^ right", static_cast<Byte>(left ^ right));
    print_bits("~left", static_cast<Byte>(~left));

    // Shifts move bits. A left shift by one often multiplies an unsigned value
    // by two, while a right shift by one divides it by two. Bits shifted out of
    // the byte are lost.
    std::println("\nShifts");
    print_bits("left << 1", static_cast<Byte>(left << 1));
    print_bits("left >> 1", static_cast<Byte>(left >> 1));

    // A mask gives individual bits a meaning. Here the lowest three bits store
    // read, write, and execute permissions.
    constexpr Byte read{1U << 2U};
    constexpr Byte write{1U << 1U};
    constexpr Byte execute{1U << 0U};
    Byte permissions{static_cast<Byte>(read | write)};

    std::println("\nBit masks");
    print_bits("initial permissions", permissions);
    std::println("can write: {}", (permissions & write) != 0U);
    std::println("can execute: {}", (permissions & execute) != 0U);

    permissions |= execute; // Set a bit.
    print_bits("set execute", permissions);

    // ~ means "not". It flips every bit. The cast back to Byte is needed
    permissions &= static_cast<Byte>(~write); // Clear a bit.
    print_bits("clear write", permissions);

    permissions ^= read; // Toggle a bit.
    print_bits("toggle read", permissions);

    // Masks and shifts can also pack multiple small values into one byte. The
    // upper and lower four-bit groups are commonly called nibbles.
    constexpr Byte high_nibble{0b1010};
    constexpr Byte low_nibble{0b0101};
    constexpr Byte nibble_mask{0b0000'1111};
    const Byte packed{static_cast<Byte>((high_nibble << 4U) | low_nibble)};
    const Byte unpacked_high{static_cast<Byte>((packed >> 4U) & nibble_mask)};
    const Byte unpacked_low{static_cast<Byte>(packed & nibble_mask)};

    std::println("\nPacking values");
    print_bits("packed", packed);
    std::println("unpacked high: {:#x}, low: {:#x}", unpacked_high,
                 unpacked_low);
}
