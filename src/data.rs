#[derive(Clone, Eq, PartialEq)]
pub struct Data {
    pub offset: u64,
    pub data: Vec<u8>,
}

impl Data {
    pub fn new(offset: u64, data: Vec<u8>) -> Data {
        Data { offset, data }
    }
}
