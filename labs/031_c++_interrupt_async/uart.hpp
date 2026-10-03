#pragma once
#include "hardware.hpp"
#include <boost/asio/any_completion_handler.hpp>
#include <cstdint>
#include <span>
#include <stdexcept>

// YOUR EXERCISE. Implement this header's template and uart.cxx.
// One send OR receive at a time; outgoing messages fit in 256-byte TX MEM.
class Uart {
  public:
    Uart(asio::any_io_executor executor, Hardware &hw);
    ~Uart();
    Uart(const Uart &) = delete;
    Uart &operator=(const Uart &) = delete;
    // The caller keeps the buffer alive through the awaited call.
    asio::awaitable<void> send(std::span<const std::uint8_t> bytes);
    asio::awaitable<char> receive(); // Optional: ./uart_lab --rx

    // Called by the application's UART_IRQHandler entry point.
    void handleInterrupt();

  private:
    using Handler = asio::any_completion_handler<void(boost::system::error_code)>;

    void startWait(std::uint32_t event, Handler handler);

    // event is irq::TX_EMPTY or irq::RX_READY. A TX wait means FIFO empty AND
    // TX_BYTE_COUNT == TX_BYTE_NUMBER (memory consumed, not wire idle).
    template <typename CompletionToken> auto asyncWait(std::uint32_t event, CompletionToken &&token) {
        // TODO 2:
        // return asio::async_initiate<CompletionToken,
        //                            void(boost::system::error_code)>(
        //     YOUR_INITIATION_LAMBDA, token);
        //
        // What should the lambda capture? Who supplies its handler argument?
        // Store the handler before enabling its interrupt mask.
        // Check the ready condition too: for TX, require live TX_EMPTY AND
        // TX_BYTE_COUNT == TX_BYTE_NUMBER. Intermediate FIFO-empty is not done.

        return asio::async_initiate<CompletionToken, void(boost::system::error_code)>(
            [this, event](auto handler) { startWait(event, Handler(std::move(handler))); }, token);
    }

    static asio::awaitable<void> notImplemented() {
        throw std::logic_error("TODO 2: implement asyncWait() in uart.hpp");
        co_return;
    }

    void completeWait(boost::system::error_code ec);
    asio::any_io_executor executor_;
    Hardware &hw_;
    Handler pendingHandler_;
    std::uint32_t waitingFor_ = 0;
};
