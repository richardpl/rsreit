use crate::data::Data;

#[derive(Clone, Eq, PartialEq)]
pub struct UndoRedo {
    e: Vec<Data>,
}

impl UndoRedo {
    pub fn new() -> UndoRedo {
        UndoRedo { e: Vec::new() }
    }

    pub fn pop(&mut self) -> Option<Data> {
        self.e.pop()
    }

    pub fn push(&mut self, data: Data) {
        self.e.push(data);
    }
}
