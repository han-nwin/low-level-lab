#include <iostream>

int main() {
    int first{10};
    int second{20};

    const int *pointer_to_const =
        &first; // pointed value is read-only through this pointer
    pointer_to_const = &second; // pointer itself can change

    int *const const_pointer = &first; // pointer cannot change; value can
    *const_pointer = 11;

    const int *const both_const =
        &second;                        // neither can change through this name
    const int &const_reference = first; // read-only reference

    std::cout << *pointer_to_const << ' ' << *const_pointer << ' '
              << *both_const << ' ' << const_reference << '\n';
    // Read declarations from the identifier outward: const_pointer is a const
    // pointer to int; pointer_to_const is a pointer to const int.
}
