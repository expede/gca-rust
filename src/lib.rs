use core::iter::Once;
use core::iter::Chain;
use core::iter::once;
use core::hash::Hasher;
use twox_hash::XxHash32;
use wasm_bindgen::prelude::*;

struct EnhancedDoubleHash {
  x: u32,
  y: u32,
  n: u32,
}

impl Iterator for EnhancedDoubleHash {
  type Item = u32;

  fn next(&mut self) -> Option<Self::Item> {
    self.x = self.x.wrapping_add(self.y);
    self.y = self.y.wrapping_add(self.n);
    self.n += 1;

    Some(self.x)
  }
}

fn enhanced_double_hash(hash_one: u32, hash_two: u32) -> EnhancedDoubleHash {
  EnhancedDoubleHash {
    x: hash_one,
    y: hash_two,
    n: 1,
  }
}

fn hashes_for(element: &[u8]) -> Chain<Once<u32>, EnhancedDoubleHash> {
  let mut hasher_a: XxHash32 = XxHash32::with_seed(0);
  hasher_a.write(element);
  let x: u32 = hasher_a.finish() as u32;

  let mut hasher_b: XxHash32 = XxHash32::with_seed(1);
  hasher_b.write(element);
  let y: u32 = hasher_b.finish() as u32;

  once(x).chain(enhanced_double_hash(x, y))
}

fn set_bit(filter: &mut [u8], index: usize) {
  let byte_idx = index / 8;
  let bit_idx = index % 8;
  filter[byte_idx] |= 1 << bit_idx;
}

fn get_bit(filter: &[u8], index: usize) -> bool {
  let byte_idx = index / 8;
  let bit_idx = index % 8;
  filter[byte_idx] & (1 << bit_idx) != 0
}

#[wasm_bindgen]
pub struct BloomFilter {
  filter: [u8; 256],
}

impl BloomFilter {
  pub fn new() -> BloomFilter {
    BloomFilter {
      filter: [0; 256],
    }
  }

  pub fn add(&mut self, element: &[u8]) {
    for index in hashes_for(element).take(30) {
      set_bit(&mut self.filter, (index % 2048) as usize)
    }
  }

  pub fn has(&self, element: &[u8]) -> bool {
    for index in hashes_for(element).take(30) {
      if !get_bit(&self.filter, (index % 2048) as usize) {
        return false
      }
    }
    return true
  }
}

#[wasm_bindgen]
pub fn empty() -> BloomFilter {
  BloomFilter::new()
}

#[wasm_bindgen]
pub fn add(filter: &mut BloomFilter, element: &[u8]) {
  filter.add(element);
}

#[wasm_bindgen]
pub fn has(filter: &BloomFilter, element: &[u8]) -> bool {
  filter.has(element)
}

#[wasm_bindgen]
pub fn hashes(element: &[u8]) -> u32 {
  let mut acc: [u32; 30] = [0; 30];

  let mut i: usize = 0;
  for hash in hashes_for(element).take(30) {
    acc[i] = hash;
    i += 1;
  }

  return acc[29];
}

pub fn test_bloom_filter() {
  let mut filter = BloomFilter::new();
  filter.add(&[0;4]);
  filter.add(&[1;4]);

  assert_eq!(filter.has(&[0;4]), true, "Filter has element added to it");
  assert_eq!(filter.has(&[1;4]), true, "Filter has element added to it");
  assert_eq!(filter.has(&[2;4]), false, "Filter doesn't have non-added element");
}
