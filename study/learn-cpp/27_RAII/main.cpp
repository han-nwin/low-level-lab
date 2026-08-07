#include <fstream>
#include <iostream>
#include <memory>
#include <stdexcept>
#include <string>
#include <utility>

// RAII means Resource Acquisition Is Initialization:
// - acquire a resource in an object's constructor
// - release it in the object's destructor
// - let scope determine the resource's lifetime
//
// Resources include memory, files, mutex locks, sockets and database
// connections. Destructors run when leaving scope, including when an exception
// is thrown.

class ScopeMessage {
  public:
    explicit ScopeMessage(std::string name) : name_{std::move(name)} {
        std::cout << "acquire " << name_ << '\n';
    }

    // ~ClassName() is the destructor. The ~ here means destruction/cleanup; it
    // is not the bitwise-NOT operator. A destructor has no return type or
    // parameters and runs automatically when the object's lifetime ends.
    ~ScopeMessage() {
        // Destructors should release resources and normally must not throw.
        std::cout << "release " << name_ << '\n';
    }

    ScopeMessage(const ScopeMessage &) = delete;
    ScopeMessage &operator=(const ScopeMessage &) = delete;

  private:
    std::string name_;
};

void automatic_cleanup() {
    ScopeMessage outer{"outer resource"};

    {
        ScopeMessage inner{"inner resource"};
        std::cout << "using both resources\n";
    } // inner.~ScopeMessage() runs automatically here

    std::cout << "inner resource is already released\n";
} // outer.~ScopeMessage() runs automatically here

void cleanup_during_exception() {
    auto number = std::make_unique<int>(42);
    ScopeMessage resource{"exception-safe resource"};

    if (*number == 42) {
        // Both the object and unique_ptr still clean up during stack unwinding.
        throw std::runtime_error{"demonstrating stack unwinding"};
    }
}

int main() {
    automatic_cleanup();

    try {
        cleanup_during_exception();
    } catch (const std::exception &error) {
        std::cout << "caught: " << error.what() << '\n';
    }

    // Standard-library RAII types:
    // - std::vector/std::string own dynamic memory
    // - std::unique_ptr owns one heap object
    // - std::shared_ptr represents shared ownership
    // - std::fstream closes its file automatically
    // - std::lock_guard releases a mutex automatically
    std::ofstream file{"/tmp/learn-cpp-raii.txt"};
    if (file)
        file << "The file closes automatically when file leaves scope.\n";

    // Prefer the Rule of Zero: compose existing RAII types so your class needs
    // no custom destructor, copy constructor, or assignment operators.
}
