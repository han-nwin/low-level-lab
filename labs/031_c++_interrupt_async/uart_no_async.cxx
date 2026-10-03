#include "uart_no_async.hpp"

#include <future>
#include <stdexcept>

UartNoAsync::UartNoAsync(asio::any_io_executor hardwareExecutor, Hardware &hw)
    : hardwareExecutor_(std::move(hardwareExecutor)), hw_(hw) {
    // This callback runs on the hardware io_context thread.
    hw_.registerInterruptHandler([this] { handleInterrupt(); });
}

void UartNoAsync::send(std::span<const std::uint8_t> bytes) {
    if (bytes.size() > Hardware::TX_MEM_BYTES)
        throw std::length_error("send exceeds 256-byte TX memory");

    {
        std::lock_guard lock(mutex_);
        done_ = false;
    }

    // All simulated MMIO access stays on the hardware thread. The caller
    // blocks here; the hardware timer and IRQ callback continue on that thread.
    auto setup = std::make_shared<std::promise<void>>();
    auto setupFinished = setup->get_future();
    std::vector<std::uint8_t> payload(bytes.begin(), bytes.end());
    asio::post(hardwareExecutor_, [this, payload = std::move(payload), setup] {
        try {
            hw_.write(Reg::TX_OFFSET, 0);
            hw_.write(Reg::TX_BYTE_NUMBER, static_cast<std::uint32_t>(payload.size()));

            auto memory = hw_.getTxMemory();
            for (std::size_t i = 0; i < payload.size(); ++i) {
                const auto wordIndex = i / 4;
                const auto byteIndex = i % 4;
                auto word = memory[wordIndex].getTxMem();
                word |= static_cast<std::uint32_t>(payload[i]) << (8 * byteIndex);
                memory[wordIndex].setTxMem(word);
            }

            auto config = hw_.read(Reg::CONTROL_1);
            auto mask = control1::irqMask(config) | irq::TX_EMPTY;
            hw_.write(Reg::CONTROL_1,
                      (config & ~control1::IRQ_MASK) | (mask << control1::IRQ_SHIFT));
            hw_.write(Reg::TX_COMMAND, txcommand::START);
            setup->set_value();
        } catch (...) {
            setup->set_exception(std::current_exception());
        }
    });
    setupFinished.get();

    std::unique_lock lock(mutex_);
    completed_.wait(lock, [this] { return done_; });
}

void UartNoAsync::handleInterrupt() {
    const auto status = hw_.read(Reg::CONTROL_0);
    if (!(status & control0::TX_EMPTY)) return;

    // Acknowledge every FIFO-empty notification, including intermediate ones.
    hw_.write(Reg::IRQ_CLEAR, irq::TX_EMPTY);
    if (hw_.read(Reg::TX_BYTE_COUNT) != hw_.read(Reg::TX_BYTE_NUMBER))
        return; // Not finished: hardware will refill the FIFO and interrupt again.

    const auto config = hw_.read(Reg::CONTROL_1);
    const auto mask = control1::irqMask(config) & ~irq::TX_EMPTY;
    hw_.write(Reg::CONTROL_1,
              (config & ~control1::IRQ_MASK) | (mask << control1::IRQ_SHIFT));

    {
        std::lock_guard lock(mutex_);
        done_ = true;
    }
    completed_.notify_one();
}
