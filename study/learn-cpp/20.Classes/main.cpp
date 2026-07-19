#include <iostream>
#include <stdexcept>

class BankAccount {
  public:
    // A one-argument constructor can otherwise be used as an implicit
    // conversion:
    //   BankAccount account = 100.0;
    // explicit prevents that surprising conversion. Construction must be
    // written intentionally instead:
    //   BankAccount account{100.0};
    // As a rule of thumb, make single-argument constructors explicit unless the
    // conversion is deliberately part of the type's design.
    explicit BankAccount(double opening_balance) : balance_{opening_balance} {
        if (opening_balance < 0)
            throw std::invalid_argument{"negative balance"};
    }

    void deposit(double amount) {
        if (amount <= 0)
            throw std::invalid_argument{"deposit must be positive"};
        balance_ += amount;
    }

    [[nodiscard]] double balance() const { return balance_; }

  private:
    // Invariants are protected by keeping representation details private.
    double balance_{};
};

int main() {
    BankAccount account{100.0};
    account.deposit(25.0);
    std::cout << "balance=" << account.balance() << '\n';
}
