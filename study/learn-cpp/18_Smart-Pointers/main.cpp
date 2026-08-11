#include <iostream>
#include <memory>
#include <string>

struct Resource {
    explicit Resource(std::string value) : name{std::move(value)} {}
    ~Resource() { std::cout << "destroy " << name << '\n'; }
    std::string name;
};

int main() {
    // NOTE: unique_ptr expresses one owner and is the default smart-pointer
    // choice.
    auto unique = std::make_unique<Resource>("unique resource");
    std::println("{}", unique->name);
    std::println("====");

    // NOTE: shared_ptr uses reference counting for genuine shared ownership.
    auto owner = std::make_shared<Resource>("shared resource");
    std::println("{}", owner->name);
    // observes without extending lifetime
    std::weak_ptr<Resource> observer_1 = owner;
    std::weak_ptr<Resource> observer_2 = owner;
    std::array<std::weak_ptr<Resource>, 2> observers{observer_1, observer_2};
    std::println("owners={}", owner.use_count());

    for (auto &ob : observers) {
        if (const auto locked = ob.lock()) {
            std::println("inside loop at lock: owners={}", owner.use_count());
            std::println("{}", locked->name);
        } // lock is destroyed here
        std::println("inside loop after done lock: owners={}",
                     owner.use_count());
    }
    // RAII releases both resources automatically when their owners leave scope.
}
