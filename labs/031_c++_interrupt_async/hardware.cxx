#include "hardware.hpp"
#include <iostream>
#include <stdexcept>

Hardware::Hardware(asio::any_io_executor executor)
    : executor_(executor), txTimer_(executor), rxTimer_(executor) {}

std::uint32_t Hardware::read(Reg reg) {
    const auto address = static_cast<std::uint32_t>(reg);
    const auto base = static_cast<std::uint32_t>(Reg::TX_MEM_BASE);
    if (address >= base && address < base + TX_MEM_BYTES && (address - base) % 4 == 0)
        return txMemory_[(address - base) / 4];
    switch (reg) {
    case Reg::CONTROL_0:
        return (rx_.empty() ? control0::RX_EMPTY : 0u) |
               (rx_.size() == FIFO_CAPACITY ? control0::RX_FULL : 0u) |
               (txFifo_.empty() ? control0::TX_EMPTY : 0u) |
               (txFifo_.size() == FIFO_CAPACITY ? control0::TX_FULL : 0u) |
               (txActive_ || txShift_ ? control0::TX_BUSY : 0u) |
               (enabled_ ? control0::ENABLE : 0u) |
               ((pending_ & irq::RX_TIMEOUT) ? control0::RX_TIMEOUT : 0u);
    case Reg::CONTROL_1: return control1_;
    case Reg::RX_TIMEOUT_MS: return rxTimeoutMs_;
    case Reg::RX_BYTE_NUMBER: return rxByteNumber_;
    case Reg::RX_BYTE_COUNT: return rxByteCount_;
    case Reg::RX_DATA: {
        if (rx_.empty()) throw std::runtime_error("RX_DATA read while RX_EMPTY");
        const auto byte = static_cast<unsigned char>(rx_.front());
        rx_.pop_front();
        if (rx_.empty()) { ++rxGeneration_; rxTimer_.cancel(); }
        return byte;
    }
    case Reg::TX_OFFSET: return txOffset_;
    case Reg::TX_BYTE_NUMBER: return txByteNumber_;
    case Reg::TX_BYTE_COUNT: return txByteCount_;
    case Reg::TX_COMMAND: return 0;
    case Reg::TX_MEM_BASE: break; // Handled above
    }
    throw std::runtime_error("Unknown register");
}

void Hardware::write(Reg reg, std::uint32_t value) {
    const auto address = static_cast<std::uint32_t>(reg);
    const auto base = static_cast<std::uint32_t>(Reg::TX_MEM_BASE);
    if (address >= base && address < base + TX_MEM_BYTES && (address - base) % 4 == 0) {
        if (txActive_) throw std::runtime_error("TX MEM write during active transfer");
        txMemory_[(address - base) / 4] = value;
        return;
    }
    switch (reg) {
    case Reg::CONTROL_0: {
        if (value & control0::UART_RESET) { reset(); return; }
        if (value & control0::CLEAR) pending_ = 0;
        const bool wasEnabled = enabled_;
        if (enabled_ && !(value & control0::ENABLE)) {
            ++txGeneration_; ++rxGeneration_;
            txTimer_.cancel(); rxTimer_.cancel();
            txScheduled_ = false;
        }
        enabled_ = (value & control0::ENABLE) != 0;
        if (enabled_) {
            scheduleTx();
            if (!wasEnabled) restartRxTimeout();
            queueInterrupt();
        }
        return;
    }
    case Reg::CONTROL_1:
        control1_ = value & control1::WRITABLE_MASK;
        queueInterrupt();
        return;
    case Reg::TX_OFFSET:
    case Reg::TX_BYTE_NUMBER:
        if (txActive_) throw std::runtime_error("TX configuration write during active transfer");
        if (value > TX_MEM_BYTES) throw std::out_of_range("TX offset/length exceeds memory size");
        if (reg == Reg::TX_OFFSET) txOffset_ = value;
        else txByteNumber_ = value;
        return;
    case Reg::TX_BYTE_COUNT: throw std::runtime_error("TX_BYTE_COUNT is read-only");
    case Reg::TX_COMMAND:
        if (value & txcommand::START) startTx();
        return;
    case Reg::TX_MEM_BASE: break; // Handled above
    case Reg::RX_TIMEOUT_MS:
        rxTimeoutMs_ = value;
        restartRxTimeout();
        return;
    case Reg::RX_DATA: throw std::runtime_error("RX_DATA is read-only");
    case Reg::RX_BYTE_COUNT: throw std::runtime_error("RX_BYTE_COUNT is read-only");
    case Reg::RX_BYTE_NUMBER:
        if (value > FIFO_CAPACITY) throw std::out_of_range("RX length exceeds 16-byte FIFO");
        if (!rx_.empty()) throw std::runtime_error("Drain RX_DATA before arming a new receive");
        ++rxGeneration_;
        rxTimer_.cancel();
        rxByteNumber_ = value;
        rxByteCount_ = 0;
        pending_ &= ~(irq::RX_READY | irq::RX_FULL | irq::RX_TIMEOUT);
        return;
    }
    throw std::runtime_error("Unknown register");
}

Reg Hardware::txMemWord(std::size_t index) {
    if (index >= TX_MEM_WORDS) throw std::out_of_range("TX MEM word index");
    return static_cast<Reg>(static_cast<std::uint32_t>(Reg::TX_MEM_BASE) + 4 * index);
}

Hardware::TxMemoryView Hardware::getTxMemory() {
    return TxMemoryView(*this);
}

Hardware::TxMemoryWord Hardware::getTxMemory(std::size_t index) {
    return getTxMemory()[index];
}

Hardware::TxMemoryWord Hardware::TxMemoryView::operator[](std::size_t index) const {
    (void)Hardware::txMemWord(index); // Validate the index before returning a word.
    return TxMemoryWord(hw_, index);
}

void Hardware::TxMemoryWord::setTxMem(std::uint32_t value) {
    hw_.write(Hardware::txMemWord(index_), value);
}

std::uint32_t Hardware::TxMemoryWord::getTxMem() const {
    return hw_.read(Hardware::txMemWord(index_));
}

void Hardware::startTx() {
    if (!enabled_) throw std::runtime_error("TX START while UART disabled");
    if (txActive_) throw std::runtime_error("TX START during active transfer");
    if (txByteNumber_ > TX_MEM_BYTES - txOffset_)
        throw std::out_of_range("TX offset + byte number exceeds TX MEM");
    pending_ &= ~(irq::TX_EMPTY | irq::TX_FULL);
    txByteCount_ = 0;
    txActive_ = txByteNumber_ != 0;
    if (!txActive_) { latch(irq::TX_EMPTY); return; }
    refillTxFifo();
    scheduleTx();
}

void Hardware::refillTxFifo() {
    // Deliberately refill in bursts, after FIFO-empty is observable. This
    // models a stalled memory feeder and makes intermediate IRQs reproducible.
    while (txByteCount_ < txByteNumber_ && txFifo_.size() < FIFO_CAPACITY) {
        const auto offset = txOffset_ + txByteCount_;
        txFifo_.push_back(static_cast<char>((txMemory_[offset / 4] >> ((offset % 4) * 8)) & 0xffu));
        ++txByteCount_;
    }
    if (txFifo_.size() == FIFO_CAPACITY) latch(irq::TX_FULL);
}

void Hardware::registerInterruptHandler(std::function<void()> handler) {
    interruptHandler_ = std::move(handler);
    queueInterrupt();
}

void Hardware::scheduleTx() {
    if (!enabled_ || (!txActive_ && !txShift_) || txScheduled_) return;
    txScheduled_ = true;
    const auto generation = txGeneration_;
    txTimer_.expires_after(std::chrono::milliseconds(10));
    txTimer_.async_wait([this, generation](boost::system::error_code ec) {
        if (ec || generation != txGeneration_) return;
        txScheduled_ = false;
        // First finish the byte already in the shift register.
        if (txShift_) {
            transmitted_.push_back(*txShift_);
            std::cout << "[hw] Wire byte: '" << *txShift_ << "'\n";
            txShift_.reset();
        }
        if (txActive_) {
            if (txFifo_.empty()) refillTxFifo();
            txShift_ = txFifo_.front();
            txFifo_.pop_front();
            if (txFifo_.empty()) {
                if (txByteCount_ == txByteNumber_) txActive_ = false;
                std::cout << "[hw] FIFO empty, consumed " << txByteCount_
                          << '/' << txByteNumber_ << " bytes; final FIFO byte still shifting\n";
                latch(irq::TX_EMPTY);
            }
        }
        scheduleTx();
    });
}

bool Hardware::injectRx(char byte) {
    if (!enabled_ || rx_.size() == FIFO_CAPACITY) return false;
    if (rxByteNumber_ != 0 && rxByteCount_ >= rxByteNumber_) return false;
    const bool wasEmpty = rx_.empty();
    rx_.push_back(byte);
    ++rxByteCount_;
    // A later target-reaching IRQ is needed even if software acknowledged the
    // initial RX_READY and left the partial payload in the FIFO.
    if (wasEmpty || (rxByteNumber_ != 0 && rxByteCount_ == rxByteNumber_)) latch(irq::RX_READY);
    if (rx_.size() == FIFO_CAPACITY) latch(irq::RX_FULL);
    restartRxTimeout();
    return true;
}

void Hardware::restartRxTimeout() {
    const auto generation = ++rxGeneration_;
    rxTimer_.cancel();
    if (!enabled_ || rx_.empty() || rxTimeoutMs_ == 0) return;
    if (rxByteNumber_ != 0 && rxByteCount_ == rxByteNumber_) return;
    rxTimer_.expires_after(std::chrono::milliseconds(rxTimeoutMs_));
    rxTimer_.async_wait([this, generation](boost::system::error_code ec) {
        if (!ec && generation == rxGeneration_ && !rx_.empty()) latch(irq::RX_TIMEOUT);
    });
}

void Hardware::latch(std::uint32_t bits) {
    pending_ |= bits;
    queueInterrupt();
}

void Hardware::queueInterrupt() {
    if (!enabled_ || irqQueued_ || !(pending_ & control1::irqMask(control1_)) || !interruptHandler_) return;
    irqQueued_ = true;
    asio::post(executor_, [this] {
        irqQueued_ = false;
        if (!enabled_ || !(pending_ & control1::irqMask(control1_)) || !interruptHandler_) return;
        std::cout << "[hw] IRQ, active bits = " << (pending_ & control1::irqMask(control1_)) << '\n';
        interruptHandler_();
        queueInterrupt(); // Remains asserted until acknowledged or masked
    });
}

void Hardware::reset() {
    ++txGeneration_; ++rxGeneration_;
    txTimer_.cancel(); rxTimer_.cancel();
    txScheduled_ = false;
    enabled_ = false;
    control1_ = control1::RESET_VALUE;
    pending_ = 0;
    rxTimeoutMs_ = 100;
    rx_.clear();
    txFifo_.clear();
    txShift_.reset();
    txMemory_.fill(0);
    txOffset_ = txByteNumber_ = txByteCount_ = 0;
    rxByteNumber_ = rxByteCount_ = 0;
    txActive_ = false;
    // transmitted_ is test-bench history, not a hardware register.
}
