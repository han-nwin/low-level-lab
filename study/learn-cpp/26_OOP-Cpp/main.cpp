#include <iostream>
#include <memory>
#include <vector>

class Shape {
  public:
    virtual ~Shape() = default; // safe deletion through base pointer
    [[nodiscard]] virtual double area() const = 0; // abstract interface
};

class Rectangle final : public Shape {
  public:
    Rectangle(double width, double height) : width_{width}, height_{height} {}
    [[nodiscard]] double area() const override { return width_ * height_; }

  private:
    double width_;
    double height_;
};

int main() {
    // Encapsulation hides representation, inheritance implements the interface,
    // and runtime polymorphism selects Rectangle::area through a Shape pointer.
    std::vector<std::unique_ptr<Shape>> shapes;
    shapes.push_back(std::make_unique<Rectangle>(3.0, 4.0));
    shapes.push_back(std::make_unique<Rectangle>(2.0, 5.0));

    for (const auto &shape : shapes)
        std::cout << "area=" << shape->area() << '\n';
    // Prefer composition unless an honest "is-a" relationship needs
    // polymorphism.
}
