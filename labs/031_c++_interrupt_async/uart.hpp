#pragma once
#include "hardware.hpp"
#include <boost/asio/any_completion_handler.hpp>
#include <cstdint>
#include <span>
#include <stdexcept>

// SIMULATED UART REGISTERS: CONTROL_0 exposes status; CONTROL_1 masks its IRQs.
//
// CONTROL_0 = live status + commands. Its low bits are status conditions,
// not a separate pending-IRQ bitmap.
//   [3:0] RX_EMPTY, RX_FULL, TX_EMPTY, TX_FULL: read-only FIFO status.
//   [4] CLEAR simulator's latched notifications; [5] ENABLE; [6] UART_RESET.
//   [7] latched RX timeout status; [8] TX_BUSY includes the shift register.
//   Interrupt source conditions are derived from these live status bits.
//   CONTROL_0.CLEAR acknowledges simulator notifications; preserve ENABLE
//   in the same write: ENABLE | CLEAR. CLEAR doesn't clear FIFO/status conditions.
//   A CONTROL_0 write sets ENABLE according to bit 5.
//
// CONTROL_1 = packed configuration (a write replaces ALL writable fields):
//   [15:0] baud divisor; [22:16] 7-bit interrupt-enable mask (1=enabled);
//   [23] parity enable; [24] odd parity; [25] two stop bits;
//   [27:26] data bits (0=5, 1=6, 2=7, 3=8); [31:28] reserved.
//   Extract the mask: control1::irqMask(hw_.read(Reg::CONTROL_1)).
//   Update it: (config & ~control1::IRQ_MASK) | (mask << control1::IRQ_SHIFT).
//   This preserves baud/framing. Raw irq:: flags aren't whole register values.
//   Disabling an interrupt does not acknowledge it or stop the peripheral.
//   Baud/framing are stored settings here; simulation timing stays fixed.
//
// ISR checks live status in CONTROL_0 to identify sources; CONTROL_1 decides
// which sources can interrupt. CLEAR acknowledges latched notifications.
// These are this simulator's rules; real register access semantics vary.

// YOUR EXERCISE. Implement this header's template and uart.cxx.
// One send OR receive at a time; outgoing messages fit in 256-byte TX MEM.
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
    asio::awaitable<char> receive(); // Optional: ./uart_lab --rx

    // The actions we take when waiting is complete
    void completeWait(boost::system::error_code ec);

    // Called by the application's UART_IRQHandler entry point.
    void handleInterrupt();

    // what event we're waiting for
    enum struct EventType {
        TX = 1,
        RX = 2,
        NONE = 0,
    };

  private:
    // event is irq::TX_EMPTY or irq::RX_READY. A TX wait means FIFO empty AND
    // TX_BYTE_COUNT == TX_BYTE_NUMBER (memory consumed, not wire idle).
    template <typename CompletionToken> auto asyncWait(EventType event, CompletionToken &&token) {
        // TODO 2:
        // return asio::async_initiate<CompletionToken,
        //                            void(boost::system::error_code)>(
        //     YOUR_INITIATION_LAMBDA, token);
        //
        // What should the lambda capture? Who supplies its handler argument?
        // Store the handler before enabling its interrupt mask.
        // Check the ready condition too: for TX, require live TX_EMPTY AND
        // TX_BYTE_COUNT == TX_BYTE_NUMBER. Intermediate FIFO-empty is not done.

        // void(error_code) is the handler signature. Call it later as handler(ec).
        return asio::async_initiate<CompletionToken, void(boost::system::error_code)>(
            [this, event](auto handler) {
                // bring asio handler out and store to the middle man
                pendingHandler_ = std::move(handler);

                // store the event type
                event_ = event;

                // Now turn on interrupt (unmask) based on what event is requesting
                if (event == EventType::TX) {
                    const auto config = hw_.read(Reg::CONTROL_1);
                    const auto mask = control1::irqMask(config) | irq::TX_EMPTY;
                    hw_.write(Reg::CONTROL_1,
                              (config & ~control1::IRQ_MASK) | (mask << control1::IRQ_SHIFT));

                    // Race protection: tx may become empty, and transfer done while we enabling the interrupt mask
                    if ((hw_.read(Reg::CONTROL_0) & control0::TX_EMPTY) &&
                        (hw_.read(Reg::TX_BYTE_COUNT) == hw_.read(Reg::TX_BYTE_NUMBER))) {

                        completeWait({}); // no ec
                    }

                } else if (event == EventType::RX) {
                    const auto config = hw_.read(Reg::CONTROL_1);
                    const auto mask = control1::irqMask(config) | irq::RX_READY | irq::RX_TIMEOUT;
                    hw_.write(Reg::CONTROL_1,
                              (config & ~control1::IRQ_MASK) | (mask << control1::IRQ_SHIFT));

                    // Race protection: rx may become full while we enabling the interrupt mask
                    if ((hw_.read(Reg::CONTROL_0) & control0::RX_FULL) &&
                        (hw_.read(Reg::RX_BYTE_COUNT) == hw_.read(Reg::RX_BYTE_NUMBER))) {

                        completeWait({}); // no ec
                    }
                }
            },
            token);
    }

    static asio::awaitable<void> notImplemented() {
        throw std::logic_error("TODO 2: implement asyncWait() in uart.hpp");
        co_return;
    }

    Hardware &hw_;

    // store current io_context executor
    asio::any_io_executor executor_;

    // this is the handler middle man, get the handler from asio and move here, then let interrupt handler post it back
    // to executor runnable queue
    asio::any_completion_handler<void(boost::system::error_code)> pendingHandler_;

    EventType event_ = EventType::NONE;
};
