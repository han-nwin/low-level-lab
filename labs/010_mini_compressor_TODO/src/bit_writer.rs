#[derive(Debug)]
pub struct Writer {
    pub packed: Vec<u8>,
    pub current_byte: u8,
    pub num_bits_filled: u8, // 0-8
}

#[derive(Debug)]
pub enum WriterError {
    InvalidBitCount(u8),
    Io(std::io::Error),
}

impl Writer {
    pub fn new() -> Writer {
        let packed: Vec<u8> = Vec::new();
        Writer {
            packed,
            current_byte: 0,
            num_bits_filled: 0,
        }
    }

    pub fn write(&mut self, value: u64, bit_count: u8) -> Result<(), WriterError> {
        //check invalid bit count
        if bit_count > 64 || bit_count == 0 {
            return Err(WriterError::InvalidBitCount(bit_count));
        }

        // mask the value first so only selected bits are visible
        // *prevent overflow
        let value_mask = if bit_count == 64 {
            u64::MAX // 1111111...1111
        } else {
            (1 << bit_count) - 1
        };
        // if bit_count = 3 -> 1000 - 1 = 0111 -> exactly 3 bit visible
        let masked_value = value & value_mask;

        // write the bits and add packed values
        let mut free_bit_left_in_current_byte = 8 - self.num_bits_filled;
        let mut bit_count = bit_count; // shadowing the input
        while free_bit_left_in_current_byte < bit_count {
            // just write the value from higher bit
            // move all the bits needed to be written to the end
            let mask = (masked_value >> (bit_count - free_bit_left_in_current_byte)) as u8;
            self.current_byte |= mask;

            //  because now num_bits_filled surely == 8
            // add it to the packed bytes
            self.packed.push(self.current_byte);
            // then reset the states
            self.current_byte = 0;
            self.num_bits_filled = 0;

            bit_count -= free_bit_left_in_current_byte;
            free_bit_left_in_current_byte = 8 - self.num_bits_filled;
        }

        // Wirte the left over bits
        // example:
        // current = 11......
        // filled = 2a
        // value = .....1110011
        // bit count = 3
        // masked_value = 00000011
        // then we shift the last 3 bit of masked_value to position 3 of current
        // shift by 8-2-3 = 3 -> masked_value after shift 00011000 -> as u8
        // current after mask now = 11011...
        let mask = (masked_value << (free_bit_left_in_current_byte - bit_count)) as u8;
        // have to - bit_count here

        self.current_byte |= mask;
        self.num_bits_filled += bit_count;

        if self.num_bits_filled == 8 {
            // if we filled the current byte
            // add it to the packed bytes
            self.packed.push(self.current_byte);
            // then reset the states
            self.current_byte = 0;
            self.num_bits_filled = 0;
        }

        Ok(())
    }

    /// Return a copy of writer.packed
    pub fn finalize(&mut self) -> Result<Vec<u8>, WriterError> {
        self.packed.push(self.current_byte);
        self.num_bits_filled = 0;

        Ok(self.packed.clone())
    }
}

impl std::fmt::Display for WriterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WriterError::InvalidBitCount(bit_count) => {
                write!(f, "Invalid bit count: {}", bit_count)
            }
            WriterError::Io(io_error) => write!(f, "IO Error: {}", io_error),
        }
    }
}

impl std::error::Error for WriterError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            WriterError::Io(io_error) => Some(io_error),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_writer() -> Result<(), WriterError> {
        let mut writer = Writer::new();
        writer.write(9764609, 20)?; // 1001[0100 11111111 00000001]
        assert_eq!(writer.packed, vec![79_u8, 240_u8]);
        Ok(())
    }

    #[test]
    fn test_finalize() -> Result<(), WriterError> {
        let mut writer = Writer::new();
        writer.write(9764609, 20)?; // 1001[0100 11111111 00000001]
        let packed = writer.finalize()?;
        assert_eq!(packed, vec![79_u8, 240_u8, 16_u8]);

        Ok(())
    }
}
