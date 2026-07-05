//! bwt.

use alloc::vec;
use alloc::vec::Vec;

// ---------------------------------------------------------------------------
// Burrows-Wheeler Transform
// ---------------------------------------------------------------------------

/// BWT変換: O(n^2) 簡易実装
#[must_use]
pub fn bwt_encode(data: &[u8]) -> (Vec<u8>, usize) {
    let n = data.len();
    if n == 0 {
        return (Vec::new(), 0);
    }

    let mut indices: Vec<usize> = (0..n).collect();
    indices.sort_by(|&a, &b| {
        for k in 0..n {
            let ca = data[(a + k) % n];
            let cb = data[(b + k) % n];
            match ca.cmp(&cb) {
                core::cmp::Ordering::Equal => {}
                other => return other,
            }
        }
        core::cmp::Ordering::Equal
    });

    let mut result = vec![0u8; n];
    let mut primary_index = 0;
    for (i, &idx) in indices.iter().enumerate() {
        result[i] = data[(idx + n - 1) % n];
        if idx == 0 {
            primary_index = i;
        }
    }
    (result, primary_index)
}

/// BWT逆変換
#[must_use]
pub fn bwt_decode(bwt: &[u8], primary_index: usize) -> Vec<u8> {
    let n = bwt.len();
    if n == 0 {
        return Vec::new();
    }

    // Count sort
    let mut counts = [0usize; 256];
    for &b in bwt {
        counts[b as usize] += 1;
    }

    let mut starts = [0usize; 256];
    let mut sum = 0;
    for i in 0..256 {
        starts[i] = sum;
        sum += counts[i];
    }

    // Build transformation vector
    let mut t = vec![0usize; n];
    let mut used = [0usize; 256];
    for i in 0..n {
        let c = bwt[i] as usize;
        t[i] = starts[c] + used[c];
        used[c] += 1;
    }

    let mut result = vec![0u8; n];
    let mut idx = primary_index;
    for i in (0..n).rev() {
        result[i] = bwt[idx];
        idx = t[idx];
    }
    result
}
