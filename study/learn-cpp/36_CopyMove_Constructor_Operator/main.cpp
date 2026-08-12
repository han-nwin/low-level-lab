#include <iostream>
#include <memory>
#include <string>
#include <type_traits>
#include <utility>
#include <vector>

// ============================================================
// 1. A class that manually implements the "Rule of Five"
// ============================================================

class Resource {
  private:
    std::string name_;
    std::unique_ptr<int> value_;

  public:
    // Normal constructor
    Resource(std::string name, int value)
        : name_{std::move(name)}, value_{std::make_unique<int>(value)} {
        std::cout << "normal constructor: " << name_ << '\n';
    }

    // --------------------------------------------------------
    // COPY CONSTRUCTOR
    //
    // Creates a NEW object by copying an existing object.
    //
    // Resource b{a};
    // --------------------------------------------------------
    Resource(const Resource &other)
        : name_{other.name_ + " (copy)"},
          value_{other.value_ ? std::make_unique<int>(*other.value_)
                              : nullptr} {
        std::cout << "copy constructor: " << other.name_ << " -> " << name_
                  << '\n';
    }

    // --------------------------------------------------------
    // MOVE CONSTRUCTOR
    //
    // Creates a NEW object by taking resources from another.
    //
    // Resource b{std::move(a)};
    //
    // noexcept promises that this operation will not throw.
    // Containers such as std::vector prefer a noexcept move
    // constructor when relocating their elements.
    // --------------------------------------------------------
    Resource(Resource &&other) noexcept
        : name_{std::move(other.name_)}, value_{std::move(other.value_)} {
        std::cout << "move constructor: resource moved into " << name_ << '\n';

        other.name_ = "<moved-from>";
    }

    // --------------------------------------------------------
    // COPY ASSIGNMENT
    //
    // Replaces an EXISTING object's value with a copy.
    //
    // b = a;
    // --------------------------------------------------------
    Resource &operator=(const Resource &other) {
        std::cout << "copy assignment: " << other.name_ << " -> " << name_
                  << '\n';

        if (this == &other) {
            return *this; // Protect against: a = a;
        }

        name_ = other.name_ + " (copy-assigned)";
        value_ = other.value_ ? std::make_unique<int>(*other.value_) : nullptr;

        // Assignment operators return *this so chaining works:
        // a = b = c;
        return *this;
    }

    // --------------------------------------------------------
    // MOVE ASSIGNMENT
    //
    // Replaces an EXISTING object's value by taking resources.
    //
    // b = std::move(a);
    // --------------------------------------------------------
    Resource &operator=(Resource &&other) noexcept {
        std::cout << "move assignment: " << other.name_ << " -> " << name_
                  << '\n';

        if (this == &other) {
            return *this;
        }

        // The old value_ owned by *this is automatically destroyed
        // when unique_ptr's move assignment replaces it.
        name_ = std::move(other.name_);
        value_ = std::move(other.value_);

        other.name_ = "<moved-from>";

        return *this;
    }

    // Destructor: releases the object's resources.
    //
    // We could write:
    //
    // ~Resource() = default;
    //
    // But this custom version lets us observe destruction.
    ~Resource() { std::cout << "destructor: " << name_ << '\n'; }

    void print() const {
        std::cout << name_ << " = ";

        if (value_) {
            std::cout << *value_;
        } else {
            std::cout << "<no resource>";
        }

        std::cout << '\n';
    }
};

// ============================================================
// 2. = default
//
// Ask the compiler to generate the normal implementation.
// This is somewhat similar to deriving traits in Rust.
// ============================================================

struct Point {
    int x{};
    int y{};

    Point() = default;

    Point(const Point &) = default;                // Default copy constructor
    Point(Point &&) noexcept = default;            // Default move constructor
    Point &operator=(const Point &) = default;     // Default copy assignment
    Point &operator=(Point &&) noexcept = default; // Default move assignment

    ~Point() = default; // Default destructor
};

// ============================================================
// 3. = delete
//
// Explicitly forbid an operation.
// ============================================================

class UniqueDevice {
  public:
    UniqueDevice() = default;

    // This object cannot be copied.
    UniqueDevice(const UniqueDevice &) = delete;
    UniqueDevice &operator=(const UniqueDevice &) = delete;

    // But it can be moved.
    UniqueDevice(UniqueDevice &&) noexcept = default;
    UniqueDevice &operator=(UniqueDevice &&) noexcept = default;

    ~UniqueDevice() = default;
};

int main() {
    std::cout << "\n=== 1. Copy constructor ===\n";

    Resource original{"original", 10};

    // copy is being CREATED, so this calls the copy constructor.
    Resource copy{original};

    original.print();
    copy.print();

    std::cout << "\n=== 2. Move constructor ===\n";

    // moved is being CREATED, so this calls the move constructor.
    //
    // std::move does not move anything by itself.
    // It allows the move constructor to be selected.
    Resource moved{std::move(original)};

    moved.print();
    original.print(); // Still valid, but moved-from

    std::cout << "\n=== 3. Copy assignment ===\n";

    Resource copy_target{"copy target", 20};

    // copy_target already exists, so this calls copy assignment.
    copy_target = copy;

    copy.print();
    copy_target.print();

    std::cout << "\n=== 4. Move assignment ===\n";

    Resource move_target{"move target", 30};

    // move_target already exists, so this calls move assignment.
    move_target = std::move(copy);

    move_target.print();
    copy.print(); // Still valid, but moved-from

    std::cout << "\n=== 5. The confusing = syntax ===\n";

    // Although this uses '=', a new object is being created.
    // Therefore, this calls the COPY CONSTRUCTOR.
    Resource another_copy = move_target;

    // Here another_copy already exists.
    // Therefore, this calls COPY ASSIGNMENT.
    another_copy = move_target;

    std::cout << "\n=== 6. Compiler-generated operations ===\n";

    Point p1;
    p1.x = 10;
    p1.y = 20;

    Point p2{p1};            // Defaulted copy constructor
    Point p3{std::move(p2)}; // Defaulted move constructor

    Point p4;
    p4 = p1; // Defaulted copy assignment

    Point p5;
    p5 = std::move(p3); // Defaulted move assignment

    std::cout << "p4 = {" << p4.x << ", " << p4.y << "}\n";
    std::cout << "p5 = {" << p5.x << ", " << p5.y << "}\n";

    std::cout << "\n=== 7. Deleted operations ===\n";

    static_assert(!std::is_copy_constructible_v<UniqueDevice>);
    static_assert(!std::is_copy_assignable_v<UniqueDevice>);
    static_assert(std::is_move_constructible_v<UniqueDevice>);
    static_assert(std::is_move_assignable_v<UniqueDevice>);

    UniqueDevice device1;

    // ERROR: copy constructor was deleted.
    // UniqueDevice device2{device1};

    // OK: move constructor is available.
    UniqueDevice device2{std::move(device1)};

    // ERROR: copy assignment was deleted.
    // device1 = device2;

    // OK: move assignment is available.
    device1 = std::move(device2);

    std::cout << "UniqueDevice can move but cannot copy\n";

    std::cout << "\n=== 8. Why move is commonly noexcept ===\n";

    std::vector<Resource> resources;
    resources.reserve(1);

    resources.emplace_back("vector item 1", 100);

    // The vector currently has room for only one element.
    // Adding another element causes reallocation.
    //
    // Because Resource's move constructor is noexcept,
    // vector can safely MOVE the existing Resource into
    // its new storage.
    resources.emplace_back("vector item 2", 200);

    std::cout << "\n=== End of main ===\n";
}
