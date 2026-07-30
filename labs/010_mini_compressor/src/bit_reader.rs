#[derive(Debug)]
pub struct Reader {
    pub packed: Vec<u8>,
}

#[derive(Debug)]
pub enum ReaderError {
    InvalidBitcount(u8),
}

impl Reader {
    pub fn new(packed: Vec<u8>) -> Self {
        Reader { packed }
    }
}
