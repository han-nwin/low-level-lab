#include "uart.hpp"
#include <algorithm>
#include <array>
#include <chrono>
#include <future>
#include <iostream>
#include <stdexcept>
#include <string_view>
#include <vector>

using namespace std::chrono_literals;

namespace {
void require(bool condition, const char *message) {
    if (!condition) throw std::runtime_error(message);
}
std::vector<std::uint8_t> bytes(std::size_t count, std::size_t start = 0) {
    std::vector<std::uint8_t> result(count);
    for (std::size_t i = 0; i < count; ++i) result[i] = static_cast<std::uint8_t>(start + i);
    return result;
}
struct Bench {
    asio::io_context io;
    Hardware hw{io.get_executor()};
    Uart uart{io.get_executor(), hw};
    bool inInterrupt = false;
    Bench() {
        hw.write(Reg::RX_TIMEOUT_MS, 10);
        hw.registerInterruptHandler([this] {
            inInterrupt = true;
            uart.handleInterrupt();
            inInterrupt = false;
        });
    }
    ~Bench() { hw.registerInterruptHandler({}); io.stop(); }
    asio::awaitable<void> pause(std::chrono::milliseconds duration) {
        asio::steady_timer timer(io, duration);
        co_await timer.async_wait(asio::use_awaitable);
    }
    void capturing() {
        require(hw.read(Reg::CONTROL_0) & control0::RX_BUSY,
                "background RX must be armed at construction and after every chunk");
        require(hw.read(Reg::RX_BYTE_NUMBER) == Uart::RX_CHUNK_BYTES,
                "hardware capture size must be independent of application read size");
        const auto mask = control1::irqMask(hw.read(Reg::CONTROL_1));
        require((mask & (irq::RX_EMPTY | irq::RX_TIMEOUT)) == (irq::RX_EMPTY | irq::RX_TIMEOUT),
                "background RX interrupts must remain enabled");
    }
    // Deliver <=16 bytes per turn so the simulated hardware can drain staging.
    asio::awaitable<void> deliver(std::vector<std::uint8_t> payload) {
        capturing();
        for (std::size_t i = 0; i < payload.size(); ++i) {
            require(hw.injectRx(static_cast<char>(payload[i])), "peer byte rejected: RX not rearmed");
            if ((i + 1) % Hardware::FIFO_CAPACITY == 0 || i + 1 == payload.size())
                co_await pause(1ms);
        }
    }
    asio::awaitable<void> flush() {
        co_await pause(20ms); // Complete a short hardware chunk by inactivity.
        capturing();
    }
    asio::awaitable<std::size_t> read(std::span<std::uint8_t> out) {
        const auto count = co_await uart.read(out);
        require(!inInterrupt, "reader continuation ran inside ISR instead of being posted");
        co_return count;
    }
    asio::awaitable<void> expect(std::size_t capacity, std::vector<std::uint8_t> expected) {
        std::vector<std::uint8_t> storage(capacity + 2, 0xa5);
        const auto count = co_await read(std::span(storage).subspan(1, capacity));
        require(count == expected.size(), "read must return available bytes up to output size");
        require(std::equal(expected.begin(), expected.end(), storage.begin() + 1),
                "ring payload/order mismatch");
        require(storage.front() == 0xa5 &&
                std::all_of(storage.begin() + 1 + count, storage.end(), [](auto v) { return v == 0xa5; }),
                "read overwrote guard or unused suffix");
        capturing();
    }
};

asio::awaitable<void> buffered(Bench &b) {
    co_await b.deliver(bytes(50)); // No application read is active.
    co_await b.flush();
    require(b.uart.rxBuffered() == 50, "background RX did not queue the completed chunk");
    co_await b.expect(10, bytes(10));
    require(b.uart.rxBuffered() == 40, "small read discarded remaining bytes");
    co_await b.expect(100, bytes(40, 10)); // Read-some: must not wait for 100.
    require(b.uart.rxBuffered() == 0, "read did not consume ring bytes");
}

asio::awaitable<void> pending(Bench &b) {
    std::array<std::uint8_t, 20> out;
    out.fill(0xa5);
    auto done = asio::co_spawn(b.io, b.read(out), asio::use_future);
    co_await b.pause(25ms);
    require(done.wait_for(0ms) != std::future_status::ready, "empty ring must wait, not return zero/time out");
    co_await b.deliver(bytes(3));
    require(done.wait_for(0ms) != std::future_status::ready, "intermediate FIFO drain completed a read");
    co_await b.flush();
    require(done.wait_for(0ms) == std::future_status::ready, "timeout chunk did not wake reader");
    require(done.get() == 3 && out[0] == 0 && out[1] == 1 && out[2] == 2 && out[3] == 0xa5,
            "pending read received incorrect partial chunk");
    require(b.uart.rxBuffered() == 0, "pending read left duplicate bytes in ring");
}

asio::awaitable<void> fullChunk(Bench &b) {
    b.hw.write(Reg::RX_TIMEOUT_MS, 0);
    co_await b.deliver(bytes(Uart::RX_CHUNK_BYTES));
    require(b.uart.rxBuffered() == Uart::RX_CHUNK_BYTES, "full capture did not publish/rearm without timeout");
    co_await b.expect(1024, bytes(Uart::RX_CHUNK_BYTES)); // No 256-byte output limit.
    co_await b.deliver(bytes(Uart::RX_CHUNK_BYTES, 37));
    co_await b.expect(1024, bytes(Uart::RX_CHUNK_BYTES, 37));
}

asio::awaitable<void> wrap(Bench &b) {
    co_await b.deliver(bytes(400));
    co_await b.flush();
    co_await b.expect(350, bytes(350));
    co_await b.deliver(bytes(300, 400));
    co_await b.flush();
    co_await b.expect(1024, bytes(350, 350));
    require(b.uart.rxDropped() == 0, "ring wrap dropped data despite free space");
}

asio::awaitable<void> overflow(Bench &b) {
    const auto capacity = Uart::RX_RING_CAPACITY;
    co_await b.deliver(bytes(capacity + 88));
    co_await b.flush();
    require(b.uart.rxBuffered() == capacity && b.uart.rxDropped() == 88,
            "overflow must keep oldest bytes and count dropped incoming bytes");
    co_await b.expect(256, bytes(256));
    co_await b.deliver(bytes(100, capacity + 88));
    co_await b.flush();
    auto expected = bytes(capacity - 256, 256);
    const auto tail = bytes(100, capacity + 88);
    expected.insert(expected.end(), tail.begin(), tail.end());
    co_await b.expect(1024, expected);
    require(b.uart.rxDropped() == 88, "overflow counter must be cumulative");
}

asio::awaitable<void> concurrentTxRx(Bench &b) {
    std::array<std::uint8_t, 32> tx;
    tx.fill('T');
    std::array<std::uint8_t, 10> rx;
    auto sending = asio::co_spawn(b.io, b.uart.send(tx), asio::use_future);
    auto reading = asio::co_spawn(b.io, b.read(rx), asio::use_future);
    co_await b.pause(2ms);
    co_await b.deliver(bytes(5));
    co_await b.flush();
    require(reading.wait_for(0ms) == std::future_status::ready && reading.get() == 5,
            "RX did not complete independently of TX");
    require(std::equal(rx.begin(), rx.begin() + 5, bytes(5).begin()), "concurrent RX corrupted data");
    require(sending.wait_for(0ms) != std::future_status::ready, "TX completed early due to RX");
    co_await b.deliver(bytes(7, 5));
    co_await b.flush();
    co_await b.expect(10, bytes(7, 5));
    while (sending.wait_for(0ms) != std::future_status::ready) co_await b.pause(5ms);
    sending.get();
    co_await b.pause(20ms);
    require(b.hw.transmitted() == std::string(32, 'T'), "TX data lost during background RX");
    b.capturing();
}

asio::awaitable<void> overlappingReads(Bench &b) {
    std::array<std::uint8_t, 8> first{}, second{};
    auto waiting = asio::co_spawn(b.io, b.read(first), asio::use_future);
    co_await b.pause(2ms);
    bool rejected = false;
    try { co_await b.uart.read(second); }
    catch (const std::logic_error &) { rejected = true; }
    require(rejected, "second nonempty read must reject instead of replacing first handler");
    require(co_await b.uart.read({}) == 0, "empty read should be a no-op even while another read waits");
    co_await b.deliver(bytes(2));
    co_await b.flush();
    require(waiting.wait_for(0ms) == std::future_status::ready && waiting.get() == 2,
            "rejected second read damaged first read");
    co_await b.deliver(bytes(1, 99));
    co_await b.flush();
    co_await b.expect(8, bytes(1, 99));
}

asio::awaitable<void> configuration(Bench &b) {
    b.capturing();
    const auto config = 96u | control1::DATA_BITS_8 | control1::PARITY_ENABLE |
        control1::PARITY_ODD | control1::TWO_STOP_BITS |
        ((irq::RX_EMPTY | irq::RX_TIMEOUT | irq::TX_FULL) << control1::IRQ_SHIFT);
    b.hw.write(Reg::CONTROL_1, config);
    require(b.hw.injectRxErrors(irq::FRAME_ERROR), "could not prepare unrelated source");
    co_await b.deliver(bytes(3));
    co_await b.flush();
    require(b.hw.read(Reg::CONTROL_1) == config, "RX rearm changed serial config/unrelated masks");
    co_await b.expect(8, bytes(3));
    require(b.hw.read(Reg::CONTROL_1) == config, "read completion disabled background RX");
    b.uart.handleInterrupt(); // Empty/spurious ISR must not duplicate a chunk.
    co_await b.pause(2ms);
    require(b.uart.rxBuffered() == 0, "stale notification copied a chunk twice");
}

asio::awaitable<void> emptyRead(Bench &b) {
    const auto status = b.hw.read(Reg::CONTROL_0);
    const auto count = b.hw.read(Reg::RX_BYTE_COUNT);
    const auto config = b.hw.read(Reg::CONTROL_1);
    require(co_await b.uart.read({}) == 0, "empty read must return zero");
    require(b.hw.read(Reg::CONTROL_0) == status && b.hw.read(Reg::RX_BYTE_COUNT) == count &&
            b.hw.read(Reg::CONTROL_1) == config && b.uart.rxBuffered() == 0,
            "empty read changed hardware or queue state");
}
} // namespace

int main() {
    struct Case { std::string_view name; asio::awaitable<void> (*run)(Bench &); };
    const Case cases[] = {
        {"capture without read / 50 bytes -> read 10 + 40", buffered},
        {"pending reader / no first-byte deadline / timeout publication", pending},
        {"full chunks / large output / rearm with timeout disabled", fullChunk},
        {"ring wrap and binary byte order", wrap},
        {"drop-new overflow policy / cumulative counter / recovery", overflow},
        {"simultaneous TX, pending read, and background RX", concurrentTxRx},
        {"overlapping read rejection and recovery", overlappingReads},
        {"config preservation / continuous masks / no duplicate publication", configuration},
        {"empty read is a no-op", emptyRead},
    };
    unsigned failures = 0;
    for (const auto &test : cases) {
        try {
            Bench b;
            auto work = asio::make_work_guard(b.io);
            auto done = asio::co_spawn(b.io, test.run(b), asio::use_future);
            const auto deadline = std::chrono::steady_clock::now() + 2s;
            while (done.wait_for(0ms) != std::future_status::ready && std::chrono::steady_clock::now() < deadline)
                b.io.run_one_for(10ms);
            require(done.wait_for(0ms) == std::future_status::ready, "timed out: check capture/rearm, ring, and reader completion");
            done.get();
            std::cout << "PASS: " << test.name << '\n';
        } catch (const std::exception &e) {
            ++failures;
            std::cerr << "FAIL: " << test.name << ": " << e.what() << '\n';
        }
    }
    std::cout << (std::size(cases) - failures) << '/' << std::size(cases) << " UART ring RX tests passed\n";
    return failures ? 1 : 0;
}
