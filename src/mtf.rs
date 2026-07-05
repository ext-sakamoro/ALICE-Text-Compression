//! mtf.

use alloc::vec::Vec;

// Move-to-Front Transform
// ---------------------------------------------------------------------------

/// # Panics
///
/// Panics if a byte is not found in the alphabet (should never happen).
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn mtf_encode(data: &[u8]) -> Vec<u8> {
    let mut alphabet: Vec<u8> = (0..=255).collect();
    let mut result = Vec::with_capacity(data.len());
    for &b in data {
        let pos = alphabet.iter().position(|&x| x == b).unwrap();
        result.push(pos as u8);
        alphabet.remove(pos);
        alphabet.insert(0, b);
    }
    result
}

#[must_use]
pub fn mtf_decode(data: &[u8]) -> Vec<u8> {
    let mut alphabet: Vec<u8> = (0..=255).collect();
    let mut result = Vec::with_capacity(data.len());
    for &idx in data {
        let b = alphabet[idx as usize];
        result.push(b);
        alphabet.remove(idx as usize);
        alphabet.insert(0, b);
    }
    result
}
