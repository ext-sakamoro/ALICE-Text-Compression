//! rle.

use alloc::vec::Vec;

// Run-Length Encoding
// ---------------------------------------------------------------------------

#[must_use]
pub fn rle_encode(data: &[u8]) -> Vec<u8> {
    if data.is_empty() {
        return Vec::new();
    }
    let mut result = Vec::new();
    let mut i = 0;
    while i < data.len() {
        let val = data[i];
        let mut count = 1u8;
        while (i + count as usize) < data.len() && data[i + count as usize] == val && count < 255 {
            count += 1;
        }
        result.push(count);
        result.push(val);
        i += count as usize;
    }
    result
}

#[must_use]
pub fn rle_decode(data: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();
    let mut i = 0;
    while i + 1 < data.len() {
        let count = data[i];
        let val = data[i + 1];
        for _ in 0..count {
            result.push(val);
        }
        i += 2;
    }
    result
}
