#include <iostream>
#include <memory>
#include <string>

struct Resource {
    explicit Resource(std::string value) : name{std::move(value)} {}
    ~Resource() { std::cout << "destroy " << name << '\n'; }
    std::string name;
};

int main() {
    // unique_ptr expresses one owner and is the default smart-pointer choice.
    auto unique = std::make_unique<Resource>("unique resource");
    std::cout << unique->name << '\n';

    // shared_ptr uses reference counting for genuine shared ownership.
    auto owner = std::make_shared<Resource>("shared resource");
    std::weak_ptr<Resource> observer =
        owner; // observes without extending lifetime
    std::cout << "owners=" << owner.use_count() << '\n';
    if (const auto locked = observer.lock())
        std::cout << locked->name << '\n';

    // RAII releases both resources automatically when their owners leave scope.
}
