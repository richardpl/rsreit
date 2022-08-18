#[derive(Clone, Eq, PartialEq)]
pub struct Hits {
    pub hits: Vec<u64>,
    pub flag: String,
    pub selected: usize,
}

#[derive(Clone, Eq, PartialEq)]
pub struct HHits {
    pub hits: Vec<Hits>,
    pub selected: usize,
}

impl Hits {
    pub fn new(flag: String) -> Hits {
        Hits {
            hits: Vec::new(),
            flag,
            selected: 0usize,
        }
    }

    pub fn is_empty(&mut self) -> bool {
        self.hits.is_empty()
    }
}

impl HHits {
    pub fn default() -> HHits {
        HHits {
            hits: Vec::new(),
            selected: 0usize,
        }
    }

    pub fn add(&mut self, hits: Hits) {
        self.hits.push(hits);
    }
}
