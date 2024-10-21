#![no_std]

use gstd::{exec, msg, Vec};

pub trait Rng {
    fn gen(&mut self) -> u32;
}

pub struct RealRng;

impl Rng for RealRng {
    fn gen(&mut self) -> u32 {
        let salt = msg::id();
        let (hash, _num) = exec::random(salt.into()).expect("random call failed");
        u32::from_le_bytes([hash[0], hash[1], hash[2], hash[3]])
    }
}

pub struct MockRng {
    values: Vec<u32>,
    index: usize,
}

impl MockRng {
    pub fn new(values: Vec<u32>) -> Self {
        Self { values, index: 0 }
    }
}

impl Rng for MockRng {
    fn gen(&mut self) -> u32 {
        let value = self.values[self.index];
        self.index = (self.index + 1) % self.values.len();
        value
    }
}
