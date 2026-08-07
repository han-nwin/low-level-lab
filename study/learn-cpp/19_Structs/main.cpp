#include <iostream>
#include <string>

// A struct groups related data into a user-defined type. Members are public by
// default, which makes structs a natural fit for simple value-like objects.
struct Point {
    double x{};
    double y{};

    // Structs can have member functions, constructors, operators, inheritance
    // and access-control sections just like classes.
    void move(double delta_x, double delta_y) {
        x += delta_x;
        y += delta_y;
    }
};

struct Student {
    std::string name;
    int score{};

    [[nodiscard]] bool passed() const {
        // const means this function does not modify the Student.
        return score >= 60;
    }
};

void award_bonus(Student &student, int points) {
    // Pass by reference when the function should modify the caller's object.
    student.score += points;
}

int main() {
    // Point is an aggregate, so its public members can be initialized in order.
    Point position{2.0, 3.0};
    position.move(1.5, -1.0);

    Student learner{"Han", 85};
    award_bonus(learner, 5);

    std::cout << "position=(" << position.x << ", " << position.y << ")\n";
    std::cout << learner.name << " scored " << learner.score
              << " and passed=" << std::boolalpha << learner.passed() << '\n';

    // In C++, struct and class have the same capabilities. Their defaults
    // differ:
    // - struct members and base classes are public by default
    // - class members and base classes are private by default
    // Conventionally, use struct for transparent data and class when invariants
    // require a private representation and controlled operations.
}
