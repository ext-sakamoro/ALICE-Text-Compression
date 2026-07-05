//! huffman.

use alloc::collections::BTreeMap;
use alloc::vec;
use alloc::vec::Vec;

// Huffman Coding
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct HuffNode {
    freq: u64,
    symbol: Option<u8>,
    left: Option<usize>,
    right: Option<usize>,
}

/// Huffman符号表の構築
///
/// # Panics
///
/// Panics if internal node indexing is inconsistent (should not happen with valid input).
#[must_use]
pub fn build_huffman_codes(data: &[u8]) -> BTreeMap<u8, Vec<bool>> {
    let mut freq = [0u64; 256];
    for &b in data {
        freq[b as usize] += 1;
    }

    let mut nodes: Vec<HuffNode> = Vec::new();
    let mut active: Vec<usize> = Vec::new();

    for (i, &f) in freq.iter().enumerate() {
        if f > 0 {
            let idx = nodes.len();
            nodes.push(HuffNode {
                freq: f,
                #[allow(clippy::cast_possible_truncation)]
                symbol: Some(i as u8),
                left: None,
                right: None,
            });
            active.push(idx);
        }
    }

    if active.len() == 1 {
        let mut codes = BTreeMap::new();
        codes.insert(nodes[active[0]].symbol.unwrap(), vec![false]);
        return codes;
    }

    while active.len() > 1 {
        // Find two smallest
        active.sort_by_key(|&i| nodes[i].freq);
        let left = active.remove(0);
        let right = active.remove(0);
        let idx = nodes.len();
        nodes.push(HuffNode {
            freq: nodes[left].freq + nodes[right].freq,
            symbol: None,
            left: Some(left),
            right: Some(right),
        });
        active.push(idx);
    }

    let mut codes = BTreeMap::new();
    if !active.is_empty() {
        build_codes_recursive(&nodes, active[0], &mut Vec::new(), &mut codes);
    }
    codes
}

fn build_codes_recursive(
    nodes: &[HuffNode],
    idx: usize,
    path: &mut Vec<bool>,
    codes: &mut BTreeMap<u8, Vec<bool>>,
) {
    if let Some(symbol) = nodes[idx].symbol {
        codes.insert(symbol, path.clone());
        return;
    }
    if let Some(left) = nodes[idx].left {
        path.push(false);
        build_codes_recursive(nodes, left, path, codes);
        path.pop();
    }
    if let Some(right) = nodes[idx].right {
        path.push(true);
        build_codes_recursive(nodes, right, path, codes);
        path.pop();
    }
}

/// 圧縮率の推定 (Shannon entropy vs 8 bits)
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn compression_ratio(data: &[u8]) -> f64 {
    if data.is_empty() {
        return 1.0;
    }
    let mut freq = [0u64; 256];
    for &b in data {
        freq[b as usize] += 1;
    }
    let n = data.len() as f64;
    let mut entropy = 0.0;
    for &f in &freq {
        if f > 0 {
            let p = f as f64 / n;
            entropy -= p * log2_approx(p);
        }
    }
    entropy / 8.0
}

fn log2_approx(x: f64) -> f64 {
    if x <= 0.0 {
        return -100.0;
    }
    let y = (x - 1.0) / (x + 1.0);
    let y2 = y * y;
    let mut sum = y;
    let mut term = y;
    for k in 1..20 {
        term *= y2;
        #[allow(clippy::cast_precision_loss)]
        let denom = f64::from(2 * k + 1);
        sum += term / denom;
    }
    2.0 * sum / core::f64::consts::LN_2 // ln(x)/ln(2)
}
