#include <cstdint>
#include <iostream>
#include <memory>
#include <span>

int main() {
    int value{42};
    int *pointer = &value; // & obtains the address; * declares a pointer

    std::cout << "address=" << pointer << ", value=" << *pointer << '\n';
    *pointer = 99; // dereference the pointer and modify the pointed-to object
    std::cout << "updated value=" << value << '\n';

    // nullptr explicitly means "points to no object". Check before
    // dereferencing.
    pointer = nullptr;
    if (pointer == nullptr)
        std::cout << "pointer is empty\n";

    // NOTE:
    // Prefer automatic storage and smart pointers. Manual new/delete is shown
    // only to recognize legacy code: int* p = new int{5}; delete p;

    // Increment a pointer
    char *p1 = new char[3]{'a', 'b', 'c'}; // create 10 memory blocks each
                                           // block has the size of a char
    int *p2 = new int[10]{1, 2, 3, 4, 5};  // create 10 memory blocks each block
                                           // has the size of an int
    double *p3 =
        new double[10]{1.01, 2.02, 3.03, 4.04, 5.05}; // create 10 memory
                                                      // size of a double
    std::uint64_t *p4 =
        new std::uint64_t[10]{1, 2, 3, 4, 5}; // create 10 memory blocks each
                                              // size of a uint32_t

    // Pointer will step sizeof(char) = 1 byte
    std::println("char size = {}", sizeof(char));
    char *p1_plus = p1 + 1;
    std::println("p1 = {}", static_cast<void *>(p1));
    std::println("p1 value = {}", *p1);
    std::println("p1_plus = {} ", static_cast<void *>(p1_plus));
    std::println("p1_plus value = {}", *p1_plus);

    //  Pointer will step sizeof(int) = 4 byte
    std::println("int size = {}", sizeof(int));
    int *p2_plus = p2 + 1;
    std::println("p2 = {}", static_cast<void *>(p2));
    std::println("p2 value = {}", *p2);
    std::println("p2_plus = {} ", static_cast<void *>(p2_plus));
    std::println("p2_plus value = {}", *p2_plus);

    // Pointer will step sizeof(double) = 8 byte
    std::println("double size = {}", sizeof(double));
    double *p3_plus = p3 + 1;
    std::println("p3 = {}", static_cast<void *>(p3));
    std::println("p3 value = {}", *p3);
    std::println("p3_plus = {} ", static_cast<void *>(p3_plus));
    std::println("p3_plus value = {}", *p3_plus);

    // Pointer will step sizeof(uint64_t) = 8 byte
    std::println("std::uint64_t size = {}", sizeof(std::uint64_t));
    std::uint64_t *p4_plus = p4 + 1;
    std::println("p4 = {}", static_cast<void *>(p4));
    std::println("p4 value = {}", *p4);
    std::println("p4_plus = {} ", static_cast<void *>(p4_plus));
    std::println("p4_plus value = {}", *p4_plus);

    // == Byte and string playground ==//
    std::string hello{"Hello"};
    std::span<const std::byte> bytes;
    bytes = std::as_bytes(std::span{hello.data(), hello.size()});

    for (std::byte b : bytes) {
        if (b == static_cast<std::byte>('\0')) {
            std::println("YOOO");
            break;
        }
        std::cout << std::hex << static_cast<int>(b) << ' ' << std::endl;
    }
}
