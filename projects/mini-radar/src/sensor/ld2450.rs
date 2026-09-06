// Entry point of ld2450 sensor logic

use crate::sensor::*;
use embedded_hal_nb::serial::{Read, Write};

// Use <Uart> generic to avoid using
// uart: UartPeripheral<Enable, UART0,.....> -> this is enormous
// Also Rust doesn't allow uart: impl Read<u8>
pub struct Ld2450<Uart> {
    uart: Uart,
    parser: Parser,
    ring_buffer: RingBuffer<128>,
}

pub struct SensorInfo {
    pub targets_info: [TargetInfo; 3], // we have 3 targets
}

#[derive(Debug)]
pub enum SensorError<UartError> {
    BufferFull,
    Uart(UartError),
}

// Example data: 30 bytes.
// AA FF 03 00 0E 03 B1 86 10 00 40 01 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 55 CC
// |---header| |---- goal 1 info ----| |---- goal 2 info ----| |---- goal 3 info ----| |eof|

// NOTE: ========================
// DESIGN:
// LD2450
//    ↓
// UART hardware FIFO
//    ↓ read()
// 128-byte ring buffer
//    ↓ pop()
// header/frame parser
//    ↓
// 30-byte frame buffer
//    ↓
// validate and process
// Ring buffer isn't needed here but for learning producer/consumer design purpose
// ========================

impl<Uart> Ld2450<Uart>
where
    // uart peri will have these traits
    Uart: Read<u8> + Write<u8>,
{
    // take the uart owner ship
    pub fn new(uart: Uart) -> Self {
        Self {
            uart,
            parser: Parser::init(),
            ring_buffer: RingBuffer::<128>::init(),
        }
    }

    pub fn poll(&mut self) -> Result<Option<SensorInfo>, SensorError<Uart::Error>> {
        match self.uart.read() {
            Ok(byte) => {
                // Push byte into ring.
                // if push fail -> ring buffer full, stop reading
                if !self.ring_buffer.push(byte) {
                    self.ring_buffer.reset();
                    self.parser.reset();
                    return Err(SensorError::BufferFull);
                }
            }
            Err(nb::Error::WouldBlock) => {
                // normal, keep processing
            }
            Err(nb::Error::Other(error)) => {
                self.ring_buffer.reset();
                self.parser.reset();
                return Err(SensorError::Uart(error));
            }
        }

        // process the bytes in the ring_buffer
        if let Some(process_byte) = self.ring_buffer.pop() // pop a byte
            // process it
            && self.parser.push(process_byte) == ParserState::Eof
        {
            if let Some(all_data) = self.parser.get_data() {
                let targets = all_data.map(|data| process_data(&data));

                self.parser.reset();
                return Ok(Some(SensorInfo {
                    targets_info: targets,
                }));
            }

            self.parser.reset();
        }

        // no byte in ring buffer
        Ok(None)
    }
}
