use wasm_bindgen::prelude::*;
use core::hash::Hasher;
use twox_hash::XxHash32;

#[wasm_bindgen]
pub fn hashes(element: u32) -> u32 {
    let mut acc: [u32; 30] = [0; 30];

    let mut x: u32;
    let mut y: u32;

    let mut hasher_a: XxHash32 = XxHash32::with_seed(0);
    hasher_a.write_u32(element);
    x = hasher_a.finish() as u32;

    let mut hasher_b: XxHash32 = XxHash32::with_seed(1);
    hasher_b.write_u32(element);
    y = hasher_b.finish() as u32;

    acc[0] = x;

    for n in 1..29 {
      x = (x + y);
      y = (y + n);
      acc[n as usize] = x;
    };

    return acc[29];
}
