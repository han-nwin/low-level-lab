#include <cstdint>
#include <iostream>
#include <vector>

using UserId = std::uint64_t;
using Scores = std::vector<int>;

// An alias improves readability but does not create a distinct type: UserId and
// uint64_t remain interchangeable. A struct would create a truly separate type.
struct StrongUserId {
    std::uint64_t value;
};

int main() {
    const UserId user{42};
    const Scores scores{90, 85, 100};
    const StrongUserId strong_user{7};

    int total{0};
    for (const int score : scores)
        total += score;
    std::cout << "user=" << user << ", strong user=" << strong_user.value
              << ", average=" << total / static_cast<int>(scores.size())
              << '\n';
}
