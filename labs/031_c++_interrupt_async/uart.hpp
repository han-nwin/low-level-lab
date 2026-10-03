#pragma once
#include "hardware.hpp"
#include <array>
#include <boost/asio/any_completion_handler.hpp>
#include <cstddef>
#include <cstdint>
#include <span>
#include <stdexcept>

// SIMULATED UART REGISTERS:
// CONTROL_0: live FIFO status, ENABLE, UART_RESET; bit 4 is reserved.
// CONTROL_1: baud [15:0], IRQ enables [22:16], framing [27:23].
// IRQ_STATUS: latched notifications, independent of interrupt enables.
// IRQ_CLEAR: seven write-one-to-clear bits matching IRQ_STATUS and irq:: flags:
//   [0] RX_EMPTY, [1] RX_FULL, [2] TX_EMPTY, [3] TX_FULL,
//   [4] RX_ERROR, [5] FRAME_ERROR, [6] RX_TIMEOUT.
// Example: hw_.write(Reg::IRQ_CLEAR, irq::TX_EMPTY);
// Zero bits preserve notifications. Clearing never changes ENABLE, FIFO data,
// counts, or masks. IRQ_CLEAR reads zero; IRQ_STATUS is read-only.
// RX_EMPTY notifies a hardware FIFO-to-memory drain. Only count == requested
// length or RX_TIMEOUT finishes a transfer; RX_FULL is staging FIFO status.
// Preserve serial settings when changing CONTROL_1's packed interrupt mask.

// One send and one read may overlap; background RX runs without a read.
// All methods/interrupts run on the same executor thread.
class Uart {
  public:
    Uart(asio::any_io_executor executor, Hardware &hw);
    Uart(const Uart &) = delete;
    Uart &operator=(const Uart &) = delete;
    Uart(Uart &&) = delete;
    Uart &operator=(Uart &&) = delete;
    ~Uart();

    // The caller keeps the buffer alive through the awaited call.
    asio::awaitable<void> send(std::span<const std::uint8_t> bytes);
    static constexpr std::size_t RX_CHUNK_BYTES = Hardware::RX_MEM_BYTES;
    static constexpr std::size_t RX_RING_CAPACITY = 512;

    // Read SOME buffered data: return min(buffer.size(), available), or wait
    // for a completed hardware chunk if empty. No application read deadline.
    // Empty buffer returns 0. Output may exceed hardware/ring capacity.
    // Only one read at a time; keep its buffer alive through the await.
    asio::awaitable<std::size_t> read(std::span<std::uint8_t> buffer);

    // Software queue diagnostics (not the hardware staging count).
    std::size_t rxBuffered() const { return rxSize_; }
    std::size_t rxDropped() const { return rxDropped_; }

    // TX masks its completed interrupt; RX wakes a reader without stopping capture.
    void completeTxWait(boost::system::error_code ec);
    void completeRxWait(boost::system::error_code ec);

    // Called by the application's UART_IRQHandler entry point.
    void handleInterrupt();

  private:
    // Publish finished hardware chunks, rearm capture, then wake a reader.
    void armRxChunk();
    void handleRxInterrupt(std::uint32_t active);

    // TX wait: FIFO empty AND TX_BYTE_COUNT == TX_BYTE_NUMBER.
    // Completion means memory consumed, not necessarily wire idle.
    template <typename CompletionToken> auto asyncTxWait(CompletionToken &&token) {
        return asio::async_initiate<CompletionToken, void(boost::system::error_code)>(
            [this](auto handler) {
                // TX state is independent of the background receiver.
                txHandler_ = std::move(handler);

                // enbale tx empty mask
                const auto config = hw_.read(Reg::CONTROL_1);
                const auto mask = control1::irqMask(config) | irq::TX_EMPTY;
                hw_.write(Reg::CONTROL_1, (config & ~control1::IRQ_MASK) | (mask << control1::IRQ_SHIFT));

                // Handle a transfer that completed before the wait was armed.
                if ((hw_.read(Reg::CONTROL_0) & control0::TX_EMPTY) &&
                    (hw_.read(Reg::TX_BYTE_COUNT) == hw_.read(Reg::TX_BYTE_NUMBER))) {
                    completeTxWait({});
                }
            },
            token);
    }

    // Wait for software-ring data, not a particular hardware transfer.
    template <typename CompletionToken> auto asyncRxWait(CompletionToken &&token) {
        return asio::async_initiate<CompletionToken, void(boost::system::error_code)>(
            [this](auto handler) {
                // Capture runs independently; this wait only observes the ring.
                rxHandler_ = std::move(handler);
                if (rxSize_ > 0)
                    completeRxWait({});
            },
            token);
    }

    Hardware &hw_;

    // store current io_context executor
    asio::any_io_executor executor_;

    asio::any_completion_handler<void(boost::system::error_code)> txHandler_;
    asio::any_completion_handler<void(boost::system::error_code)> rxHandler_;

    std::array<std::uint8_t, RX_RING_CAPACITY> rxRing_{};
    std::size_t rxHead_ = 0;    // Index of the next byte to read.
    std::size_t rxSize_ = 0;    // Insert at (rxHead_ + rxSize_) % capacity.
    std::size_t rxDropped_ = 0; // Drop NEW bytes when full; preserve queued bytes.
    bool readPending_ = false;  // Held until the read coroutine has consumed data.
};
