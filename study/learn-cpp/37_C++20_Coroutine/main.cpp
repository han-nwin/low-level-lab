#include <boost/asio.hpp>
#include <utility>

#include <chrono>
#include <iostream>
#include <thread>

namespace asio = boost::asio;
using namespace std::chrono_literals;

asio::awaitable<void> task(const char *name) {
    auto executor = co_await asio::this_coro::executor;
    asio::steady_timer timer{executor, 1s};

    std::cout << name << " started on " << std::this_thread::get_id() << '\n';
    co_await timer.async_wait(asio::use_awaitable);
    std::cout << name << " finished on " << std::this_thread::get_id() << '\n';
}

void single_threaded() {
    std::cout << "\nSingle threaded:\n";

    asio::io_context runtime;

    asio::co_spawn(runtime, task("A"), asio::detached);
    asio::co_spawn(runtime, task("B"), asio::detached);

    runtime.run();
}

void multi_threaded() {
    std::cout << "\nMulti threaded:\n";

    asio::io_context runtime;

    asio::co_spawn(runtime, task("A"), asio::detached);
    asio::co_spawn(runtime, task("B"), asio::detached);

    std::jthread thread1{[&] { runtime.run(); }};
    std::jthread thread2{[&] { runtime.run(); }};
}

int main() {
    single_threaded();
    multi_threaded();
}
