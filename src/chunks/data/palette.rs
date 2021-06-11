use std::collections::HashMap;
use block_macro::block_id;

use crate::common::block::Block;

const MIN_BITS: u8 = 4;

pub struct Palette {
    pub entries: Vec<Block>,
    block_to_entry: HashMap<Block, u16>,
}

impl Palette {
    pub fn new() -> Self {
        Self {
            entries: vec![Block(block_id!(air))],
            block_to_entry: HashMap::new(),
        }
    }

    pub fn from_entries(input: &[Block]) -> Self {
        Self {
            entries: input.iter().cloned().collect(),
            block_to_entry: input.iter().enumerate()
                .map(|(index, block)| (*block, index as u16))
                .collect(),
        }
    }

    pub fn get_bits_per_block(&self) -> u8 {
        ((self.entries.len() as f32).log2().ceil() as u8)
            .max(MIN_BITS)
    }

    pub fn get_block(&self, id: u16) -> Block {
        self.entries[id as usize]
    }

    pub fn get_or_add_id(&mut self, block: Block) -> u16 {
        if let Some(id) = self.block_to_entry.get(&block) {
            *id
        } else {
            let id = self.entries.len() as u16;
            self.entries.push(block);
            self.block_to_entry.insert(block, id);
            id
        }
    }
}