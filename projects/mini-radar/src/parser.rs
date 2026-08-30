// AA FF 03 00 0E 03 B1 86 10 00 40 01 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 55 CC
// |---header| |---- goal 1 info ----| |---- goal 2 info ----| |---- goal 3 info ----| |eof|
pub struct Parser {
    buffer: [u8; 30],
    current_byte: usize,
    state: ParserState,
}

#[derive(PartialEq, Eq)]
pub enum ParserState {
    Header,
    Data,
    Eof,
}

const HEADER: [u8; 4] = [0xAA, 0xFF, 0x03, 0x00];
const FOOTER: [u8; 2] = [0x55, 0xCC];

impl Parser {
    pub fn init() -> Parser {
        Parser {
            buffer: [0; 30],
            current_byte: 0,
            state: ParserState::Header,
        }
    }

    pub fn reset(&mut self) {
        self.state = ParserState::Header;
        self.buffer = [0; 30];
        self.current_byte = 0;
    }

    // get data into 3 different array
    pub fn get_data(&self) -> Option<[[u8; 8]; 3]> {
        if self.state != ParserState::Eof {
            return None;
        }

        let mut return_array = [[0; 8]; 3];
        return_array[0].copy_from_slice(&self.buffer[4..12]);
        return_array[1].copy_from_slice(&self.buffer[12..20]);
        return_array[2].copy_from_slice(&self.buffer[20..28]);
        Some(return_array)
    }

    pub fn push(&mut self, byte: u8) -> ParserState {
        // if still at header, check header
        if self.current_byte < 4 {
            self.state = ParserState::Header;
            // correct
            if byte == HEADER[self.current_byte] {
                self.buffer[self.current_byte] = byte;
                self.current_byte += 1;
                return ParserState::Header;
            } else {
                // something went wrong
                // reset and keep searching for next header
                self.reset();
                return ParserState::Header;
            }
        }

        // check footer
        if self.current_byte == 28 || self.current_byte == 29 {
            self.state = ParserState::Eof;
            if byte == FOOTER[self.current_byte - 28] {
                self.buffer[self.current_byte] = byte;
                self.current_byte += 1;
                return ParserState::Eof;
            } else {
                // soomething wrong, reset and go back to header
                self.reset();
                return ParserState::Header;
            }
        }

        // data
        self.state = ParserState::Data;
        self.buffer[self.current_byte] = byte;
        self.current_byte += 1;
        ParserState::Data
    }
}
