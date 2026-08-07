#include <iostream>
#include <string>

int main() {
    const int score{87};
    std::string grade;

    if (score >= 90) {
        grade = "A";
    } else if (score >= 80) {
        grade = "B";
    } else {
        grade = "needs more practice";
    }

    // The conditional operator is an expression: condition ? true_value :
    // false_value.
    const std::string result = score >= 60 ? "pass" : "fail";

    switch (score / 10) {
    case 10:
    case 9:
        std::cout << "excellent; ";
        break;
    case 8:
        std::cout << "good; ";
        break;
    default:
        std::cout << "keep going; ";
        break;
    }
    std::cout << "grade=" << grade << ", result=" << result << '\n';
}
