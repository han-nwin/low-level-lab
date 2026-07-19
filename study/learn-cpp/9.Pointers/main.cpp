#include <iostream>

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

    // Prefer automatic storage and smart pointers. Manual new/delete is shown
    // only to recognize legacy code: int* p = new int{5}; delete p;
}
