pub struct RingBuffer<const SIZE: usize> {
    buffer: [u8; SIZE],
    head: usize,
    tal: usize,
    length: usize,
}

impl<const SIZE: usize> RingBuffer<SIZE> {
    pub fn init() -> RingBuffer<SIZE> {
        RingBuffer {
            buffer: [0; SIZE],
            head: 0,
            tal: 0,
            length: 0,
        }
    }
    /// true: push sucess
    /// false:: push failed, buffer is full
    pub fn push(&mut self, byte: u8) -> bool {
        if self.length < SIZE {
            self.buffer[self.head] = byte;
            self.head = (self.head + 1) % SIZE;
            self.length += 1;
            true
        } else {
            false
        }
    }

    pub fn pop(&mut self) -> Option<u8> {
        if self.length > 0 {
            let byte = self.buffer[self.tal];
            self.tal = (self.tal + 1) % SIZE;
            self.length -= 1;
            Some(byte)
        } else {
            None
        }
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    #[allow(dead_code)]
    pub fn current_len(&self) -> usize {
        self.length
    }

    #[allow(dead_code)]
    pub fn size(&self) -> usize {
        SIZE
    }

    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.head = 0;
        self.tal = 0;
        self.length = 0;
    }
}
