#pragma once

#include "hardware.hpp"
#include <condition_variable>
#include <mutex>
#include <span>
#include <vector>

// Blocking counterpart to Uart::send(). Hardware and its interrupt handler
// run on a separate Asio thread so this caller can block without stopping TX.
class UartNoAsync {
  public:
    UartNoAsync(asio::any_io_executor hardwareExecutor, Hardware &hw);
    UartNoAsync(const UartNoAsync &) = delete;
    UartNoAsync &operator=(const UartNoAsync &) = delete;

    void send(std::span<const std::uint8_t> bytes);

  private:
    void handleInterrupt();

    asio::any_io_executor hardwareExecutor_;
    Hardware &hw_;
    std::mutex mutex_;
    std::condition_variable completed_;
    bool done_ = false;
};
