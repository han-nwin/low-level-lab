#include <iostream>

struct Vector2 {
    double x{};
    double y{};

    Vector2 &operator+=(const Vector2 &other) {
        x += other.x;
        y += other.y;
        return *this;
    }
};

// Implement a binary operator using its compound-assignment counterpart.
Vector2 operator+(Vector2 left, const Vector2 &right) {
    left += right;
    return left;
}

std::ostream &operator<<(std::ostream &output, const Vector2 &vector) {
    return output << '(' << vector.x << ", " << vector.y << ')';
}

int main() {
    const Vector2 position{2.0, 3.0};
    const Vector2 movement{1.5, -1.0};
    std::cout << "result=" << position + movement << '\n';
    // Overloads should preserve the operator's familiar meaning; they cannot
    // add new operators or change precedence, associativity, or number of
    // operands.
}
