#pragma once
#include <utility>
#include <boost/asio.hpp>
#include <chrono>
#include <array>
#include <cstdint>
#include <deque>
#include <functional>
#include <optional>
#include <string>

namespace asio = boost::asio;

// Fictional UART register offsets. See README.md for read/write semantics.
enum class Reg : std::uint32_t {
    CONTROL_0 = 0x00,
    CONTROL_1 = 0x04, // IRQ mask: 1 = unmasked/enabled
    IRQ_STATUS = 0x08, // Latched events; write 1 to acknowledge each bit
    RX_DATA = 0x0c,
    RX_TIMEOUT_MS = 0x14,
    TX_OFFSET = 0x18,      // Byte offset into TX MEM
    TX_BYTE_NUMBER = 0x1c, // Number of bytes to send
    TX_BYTE_COUNT = 0x20,  // Read-only: bytes consumed from TX MEM into FIFO
    TX_COMMAND = 0x24,     // Write txcommand::START to begin
    TX_MEM_BASE = 0x100,   // 64 consecutive 32-bit words, through 0x1fc
};
namespace txcommand {
inline constexpr std::uint32_t START = 1u << 0; // Write-one command, reads as zero
}
namespace control0 {
inline constexpr std::uint32_t RX_EMPTY   = 1u << 0;
inline constexpr std::uint32_t RX_FULL    = 1u << 1;
inline constexpr std::uint32_t TX_EMPTY   = 1u << 2;
inline constexpr std::uint32_t TX_FULL    = 1u << 3;
inline constexpr std::uint32_t CLEAR      = 1u << 4; // Clear all IRQ latches
inline constexpr std::uint32_t ENABLE     = 1u << 5;
inline constexpr std::uint32_t UART_RESET = 1u << 6;
inline constexpr std::uint32_t RX_TIMEOUT = 1u << 7;
inline constexpr std::uint32_t TX_BUSY    = 1u << 8; // Transfer/FIFO/shift register active
}
// These bits apply to BOTH CONTROL_1 and IRQ_STATUS.
namespace irq {
inline constexpr std::uint32_t RX_READY   = 1u << 0; // Empty -> nonempty
inline constexpr std::uint32_t RX_FULL    = 1u << 1;
inline constexpr std::uint32_t TX_EMPTY   = 1u << 2; // FIFO drains; check transfer count!
inline constexpr std::uint32_t TX_FULL    = 1u << 3;
inline constexpr std::uint32_t RX_TIMEOUT = 1u << 7;
inline constexpr std::uint32_t ALL = RX_READY | RX_FULL | TX_EMPTY | TX_FULL | RX_TIMEOUT;
}

// All access/callbacks run on one io_context thread. Desktop simulation;
// keep Hardware alive until io.run() ends. See README for lifetime contract.
class Hardware {
  public:
    static constexpr std::size_t FIFO_CAPACITY = 16;
    static constexpr std::size_t TX_MEM_WORDS = 64;
    static constexpr std::size_t TX_MEM_BYTES = TX_MEM_WORDS * 4;
    explicit Hardware(asio::any_io_executor executor);
    std::uint32_t read(Reg reg);
    void write(Reg reg, std::uint32_t value);
    // A view of 64 simulated MMIO words. Views refer to this Hardware object;
    // copying a view does not copy the memory. Hardware must outlive the view.
    class TxMemoryWord {
      public:
        TxMemoryWord(Hardware &hw, std::size_t index) : hw_(hw), index_(index) {}
        void setTxMem(std::uint32_t value);
        std::uint32_t getTxMem() const;
      private:
        Hardware &hw_;
        std::size_t index_;
    };
    class TxMemoryView {
      public:
        explicit TxMemoryView(Hardware &hw) : hw_(hw) {}
        TxMemoryWord operator[](std::size_t index) const;
        constexpr std::size_t size() const { return TX_MEM_WORDS; }
      private:
        Hardware &hw_;
    };
    TxMemoryView getTxMemory();
    TxMemoryWord getTxMemory(std::size_t index);
    // Use read/write(txMemWord(i), value) for 32-bit register access instead.
    static Reg txMemWord(std::size_t index);
    void registerInterruptHandler(std::function<void()> handler);

    // Test-bench interface; do not use these in your Uart implementation.
    bool injectRx(char byte);
    const std::string &transmitted() const { return transmitted_; }

  private:
    void scheduleTx();
    void startTx();
    void refillTxFifo();
    void restartRxTimeout();
    void latch(std::uint32_t bits);
    void queueInterrupt();
    void reset();
    asio::any_io_executor executor_;
    asio::steady_timer txTimer_, rxTimer_;
    std::deque<char> rx_, txFifo_;
    std::optional<char> txShift_; // Byte currently being serialized onto the wire
    std::array<std::uint32_t, TX_MEM_WORDS> txMemory_{};
    std::uint32_t txOffset_ = 0, txByteNumber_ = 0, txByteCount_ = 0;
    bool txActive_ = false;
    std::string transmitted_;
    std::function<void()> interruptHandler_;
    std::uint32_t mask_ = 0, pending_ = 0, rxTimeoutMs_ = 100;
    bool enabled_ = false, txScheduled_ = false, irqQueued_ = false;
    std::uint64_t txGeneration_ = 0, rxGeneration_ = 0;
};
