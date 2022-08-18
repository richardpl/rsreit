#[derive(Clone, Eq, PartialEq)]
pub struct Block {
    pub buffer: Vec<u8>,
    pub source: Vec<u8>,
    pub offset: u64,
    pub size: u64,
    pub prev_offset: u64,
    pub prev_size: u64,
}

impl Block {
    pub fn new(size: usize) -> Block {
        Block {
            buffer: Vec::with_capacity(size),
            source: Vec::with_capacity(size),
            offset: 0,
            size: size as u64,
            prev_offset: 0,
            prev_size: 0,
        }
    }
}
