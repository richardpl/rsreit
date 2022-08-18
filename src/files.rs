use crate::block::Block;
use crate::hits::HHits;
use crate::tabs::Tabs;
use crate::undo::UndoRedo;
use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::ops::Bound::Excluded;
use std::ops::Bound::Included;
use std::os::unix::prelude::FileExt;

#[derive(Clone, Eq, PartialEq)]
pub struct File {
    pub path: String,
    pub size: u64,
    pub block: Block,
    pub patch: BTreeMap<u64, Vec<u8>>,
    pub undo: UndoRedo,
    pub redo: UndoRedo,
    pub hhits: HHits,
}

#[derive(Eq, PartialEq)]
pub struct Files {
    pub files: Vec<File>,
    pub index: usize,
}

const WRITE_BLOCK: u64 = 2048u64;

impl Files {
    pub fn default() -> Files {
        Files {
            files: Vec::new(),
            index: 0,
        }
    }

    pub fn next(&mut self) {
        if !self.files.is_empty() {
            self.index = (self.index + 1) % self.files.len();
        }
    }

    pub fn previous(&mut self) {
        if !self.files.is_empty() {
            if self.index > 0 {
                self.index -= 1;
            } else {
                self.index = self.files.len() - 1;
            }
        }
    }

    fn new(path: String) -> File {
        File {
            path,
            size: 0u64,
            block: Block::new(2048usize),
            patch: BTreeMap::new(),
            undo: UndoRedo::new(),
            redo: UndoRedo::new(),
            hhits: HHits::default(),
        }
    }

    pub fn add(&mut self, path: String, tabs: &mut Tabs) {
        self.files.push(Self::new(path));
        tabs.add(String::from(format!("tab{}", tabs.tabs.len())));
    }

    pub fn current(&mut self, index: usize) -> &mut File {
        &mut self.files[index]
    }

    pub fn current_path(&mut self, tabs: &mut Tabs) -> &String {
        let file_index = tabs.file_index();
        &self.files[file_index].path
    }

    pub fn read_block(
        file: &mut std::fs::File,
        size: u64,
        offset: u64,
        len: u64,
        buffer: &mut Vec<u8>,
    ) -> io::Result<()> {
        let mut nb_read = 0;
        if offset < len {
            file.seek(SeekFrom::Start(offset))?;
            buffer.resize(size.try_into().unwrap(), 0);
            let mut handle = file.take(size);
            nb_read = handle.read(buffer)?;
        }
        buffer[nb_read..size as usize].fill(0xFF);
        Ok(())
    }

    pub fn write(&mut self, index: usize) -> io::Result<()> {
        let mut block = Block::new(2048usize);
        let fi = self.current(index);
        let path = fi.path.clone();
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)?;
        let len = std::fs::metadata(path)?.len();
        let r = fi.patch.range((Included(&0), Excluded(&fi.size)));
        let mut next_offset;
        let mut prev_offset = 0u64;
        for (offset, bytes) in r {
            let at = offset & !(WRITE_BLOCK - 1);
            let size = ((bytes.len() as u64 - 1) | (WRITE_BLOCK - 1)) + 1;

            next_offset = at + size as u64;
            if prev_offset < next_offset {
                Self::read_block(&mut file, size, at, len, &mut block.buffer)?;
                block.offset = at;
                Files::do_apply_patch(&mut block, &fi.patch);
                let bsize = std::cmp::min(size, len - at) as usize;
                file.write_at(&block.buffer[0..bsize], at)?;
                prev_offset = next_offset;
            }
        }
        fi.patch.clear();
        Ok(())
    }

    pub fn do_apply_patch(block: &mut Block, patch: &BTreeMap<u64, Vec<u8>>) {
        let min = block.offset;
        let max = block.offset + block.size;
        let r = patch.range((Included(&min), Excluded(&max)));
        for (key, value) in r {
            let bmin = (key - min) as usize;
            let bmax = bmin + value.len() as usize;
            block.buffer.splice(bmin..bmax, value.clone());
        }
    }
}
