use js_sys::Uint8Array;
use wasm_bindgen_futures::JsFuture;
use core::iter::Once;
use core::iter::Chain;
use core::iter::once;
use core::hash::Hasher;
use twox_hash::XxHash32;
use wasm_bindgen::prelude::*;
use web_sys::{SubtleCrypto};

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
#[derive(Clone, Copy)]
pub struct BloomFilter {
  filter: [u8; 256],
}

impl BloomFilter {
  pub fn new() -> BloomFilter {
    BloomFilter {
      filter: [0; 256],
    }
  }

  pub fn add(&mut self, element: &[u8]) -> &mut Self {
    for index in hashes_for(element).take(30) {
      set_bit(&mut self.filter, (index % 2048) as usize)
    }
    self
  }

  pub fn has(&self, element: &[u8]) -> bool {
    for index in hashes_for(element).take(30) {
      if !get_bit(&self.filter, (index % 2048) as usize) {
        return false
      }
    }
    return true
  }

  pub fn count_ones(&self) -> u32 {
    let mut count: u32 = 0;
    for byte in self.filter.iter() {
      count += byte.count_ones();
    }
    count
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
pub fn count_ones(filter: &BloomFilter) -> u32 {
  filter.count_ones()
}

// #[wasm_bindgen]
// pub async fn saturate(webcrypto: &web_sys::SubtleCrypto, filter: &BloomFilter) -> BloomFilter {
//   let mut working_filter = filter.clone();

//   let mut ones = working_filter.count_ones();
//   let mut remaining_steps_at_least = (1019 - ones) / 30;

//   while remaining_steps_at_least>= 1 {
//     for _ in 0..remaining_steps_at_least {
//       saturate_step(webcrypto, &mut working_filter).await;
//     }

//     ones = working_filter.count_ones();
//     remaining_steps_at_least = (1019 - ones) / 30;
//   }

//   working_filter

//   // saturate_slow_step(webcrypto, &working_filter).await
// }

// #[wasm_bindgen]
// pub async fn saturate_slow_step(webcrypto: &SubtleCrypto, filter: &BloomFilter) -> BloomFilter {
//   let mut filter_stepped = filter.clone();
//   saturate_step(webcrypto, &mut filter_stepped).await;
//   if filter_stepped.count_ones() >= 1019 {
//     *filter
//   } else {
//     saturate_slow_step(webcrypto, &filter_stepped).await
//   }
// }

#[wasm_bindgen]
pub async fn saturate_step(webcrypto: &SubtleCrypto, filter: &mut BloomFilter) {
  let mut sha256 = [0 as u8, 256];
  Uint8Array::new(&JsFuture::from(
    webcrypto
      .digest_with_str_and_u8_array("sha-256", &mut filter.filter) // Why do I provide a mutable filter here? wut? I guess because *in theory* that's what the JS api *could* do?
      .expect("should have access to the webcrypto api")
    ).await
    .expect("should not throw the promise")
  ).copy_to(&mut sha256);
  filter.add(&sha256);
}

#[wasm_bindgen]
pub fn hashes(element: &[u8]) -> u32 {
  hashes_for(element).take(30).last().unwrap()
}

pub fn test_bloom_filter() {
  let mut filter = BloomFilter::new();
  filter.add(&[0;4]);
  filter.add(&[1;4]);

  assert_eq!(filter.has(&[0;4]), true, "Filter has element added to it");
  assert_eq!(filter.has(&[1;4]), true, "Filter has element added to it");
  assert_eq!(filter.has(&[2;4]), false, "Filter doesn't have non-added element");
}

#[wasm_bindgen(start)]
pub fn main() {
  console_error_panic_hook::set_once();
}
