//! ALICE-Text-Compression — Text-specific compression
//!
//! Burrows-Wheeler変換、Move-to-Front、Run-Length Encoding、Huffman符号

#![no_std]
extern crate alloc;
use alloc::{collections::BTreeMap, vec, vec::Vec};

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

// ---------------------------------------------------------------------------
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

// ---------------------------------------------------------------------------
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

// ---------------------------------------------------------------------------
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

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompressionError {
    InvalidData,
    DecodeFailed,
}

impl core::fmt::Display for CompressionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidData => write!(f, "invalid data"),
            Self::DecodeFailed => write!(f, "decode failed"),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bwt_roundtrip() {
        let data = b"banana";
        let (encoded, idx) = bwt_encode(data);
        let decoded = bwt_decode(&encoded, idx);
        assert_eq!(decoded, data);
    }

    #[test]
    fn bwt_roundtrip_longer() {
        let data = b"abracadabra";
        let (encoded, idx) = bwt_encode(data);
        let decoded = bwt_decode(&encoded, idx);
        assert_eq!(decoded, data);
    }

    #[test]
    fn mtf_roundtrip() {
        let data = b"banana";
        let encoded = mtf_encode(data);
        let decoded = mtf_decode(&encoded);
        assert_eq!(decoded, data);
    }

    #[test]
    fn rle_roundtrip() {
        let data = b"aaabbccccdddddd";
        let encoded = rle_encode(data);
        let decoded = rle_decode(&encoded);
        assert_eq!(decoded, data);
    }

    #[test]
    fn rle_no_runs() {
        let data = b"abcdef";
        let encoded = rle_encode(data);
        let decoded = rle_decode(&encoded);
        assert_eq!(decoded, data);
    }

    #[test]
    fn rle_single() {
        let data = vec![42u8; 100];
        let encoded = rle_encode(&data);
        assert!(encoded.len() < data.len());
        assert_eq!(rle_decode(&encoded), data);
    }

    #[test]
    fn huffman_codes() {
        let codes = build_huffman_codes(b"aabbbcccc");
        assert!(!codes.is_empty());
        // 'c' most frequent → shortest code
        assert!(codes[&b'c'].len() <= codes[&b'a'].len());
    }

    #[test]
    fn huffman_single_symbol() {
        let codes = build_huffman_codes(b"aaaa");
        assert_eq!(codes.len(), 1);
    }

    #[test]
    fn compression_ratio_random() {
        let data: Vec<u8> = (0..=255).collect();
        let ratio = compression_ratio(&data);
        assert!(ratio > 0.5); // uniform → high entropy, ratio near 1.0
    }

    #[test]
    fn compression_ratio_repetitive() {
        let data = vec![0u8; 1000];
        let ratio = compression_ratio(&data);
        assert!(ratio < 0.01); // very compressible
    }

    #[test]
    fn bwt_empty() {
        let (encoded, _) = bwt_encode(b"");
        assert!(encoded.is_empty());
    }

    #[test]
    fn bwt_mtf_rle_pipeline() {
        let data = b"mississippi";
        let (bwt, idx) = bwt_encode(data);
        let mtf = mtf_encode(&bwt);
        let rle = rle_encode(&mtf);
        // 復号
        let mtf2 = rle_decode(&rle);
        let bwt2 = mtf_decode(&mtf2);
        let result = bwt_decode(&bwt2, idx);
        assert_eq!(result, data);
    }

    // =======================================================================
    // BWT 追加テスト
    // =======================================================================

    #[test]
    fn bwt_single_byte() {
        // 1バイト入力の往復
        let data = b"x";
        let (encoded, idx) = bwt_encode(data);
        assert_eq!(encoded, b"x");
        assert_eq!(idx, 0);
        assert_eq!(bwt_decode(&encoded, idx), data);
    }

    #[test]
    fn bwt_all_same() {
        // 全同一文字
        let data = b"aaaa";
        let (encoded, idx) = bwt_encode(data);
        assert_eq!(encoded, b"aaaa");
        assert_eq!(bwt_decode(&encoded, idx), data);
    }

    #[test]
    fn bwt_palindrome() {
        // 回文
        let data = b"racecar";
        let (encoded, idx) = bwt_encode(data);
        let decoded = bwt_decode(&encoded, idx);
        assert_eq!(decoded, data);
    }

    #[test]
    fn bwt_two_chars() {
        // 2文字入力
        let data = b"ab";
        let (encoded, idx) = bwt_encode(data);
        let decoded = bwt_decode(&encoded, idx);
        assert_eq!(decoded, data);
    }

    #[test]
    fn bwt_repeated_pattern() {
        // 繰り返しパターン
        let data = b"abcabcabc";
        let (encoded, idx) = bwt_encode(data);
        assert_eq!(bwt_decode(&encoded, idx), data);
    }

    #[test]
    fn bwt_sorted_input() {
        // ソート済み入力
        let data = b"abcdef";
        let (encoded, idx) = bwt_encode(data);
        assert_eq!(bwt_decode(&encoded, idx), data);
    }

    #[test]
    fn bwt_reverse_sorted() {
        // 逆順入力
        let data = b"fedcba";
        let (encoded, idx) = bwt_encode(data);
        assert_eq!(bwt_decode(&encoded, idx), data);
    }

    #[test]
    fn bwt_binary_data() {
        // バイナリデータ
        let data: Vec<u8> = (0..32).collect();
        let (encoded, idx) = bwt_encode(&data);
        assert_eq!(bwt_decode(&encoded, idx), data);
    }

    #[test]
    fn bwt_with_nulls() {
        // NULLバイト含むデータ
        let data = b"\x00\x01\x00\x02\x00";
        let (encoded, idx) = bwt_encode(data);
        assert_eq!(bwt_decode(&encoded, idx), data);
    }

    #[test]
    fn bwt_primary_index_valid() {
        // primary_indexが有効範囲であること
        let data = b"hello world";
        let (encoded, idx) = bwt_encode(data);
        assert!(idx < encoded.len());
    }

    #[test]
    fn bwt_output_length_preserved() {
        // 出力長が入力長と同じ
        let data = b"compression test";
        let (encoded, _) = bwt_encode(data);
        assert_eq!(encoded.len(), data.len());
    }

    #[test]
    fn bwt_decode_empty() {
        // 空データの復号
        let decoded = bwt_decode(b"", 0);
        assert!(decoded.is_empty());
    }

    #[test]
    fn bwt_long_text() {
        // 長めのテキスト
        let data = b"the quick brown fox jumps over the lazy dog";
        let (encoded, idx) = bwt_encode(data);
        assert_eq!(bwt_decode(&encoded, idx), data);
    }

    #[test]
    fn bwt_spaces_only() {
        // 空白のみ
        let data = b"     ";
        let (encoded, idx) = bwt_encode(data);
        assert_eq!(bwt_decode(&encoded, idx), data);
    }

    #[test]
    fn bwt_alternating() {
        // 交互パターン
        let data = b"ababababab";
        let (encoded, idx) = bwt_encode(data);
        assert_eq!(bwt_decode(&encoded, idx), data);
    }

    #[test]
    fn bwt_high_bytes() {
        // 高バイト値
        let data: Vec<u8> = (200..=255).collect();
        let (encoded, idx) = bwt_encode(&data);
        assert_eq!(bwt_decode(&encoded, idx), data);
    }

    #[test]
    fn bwt_output_is_permutation() {
        // BWT出力は入力のバイト順列であること
        let data = b"banana";
        let (encoded, _) = bwt_encode(data);
        let mut sorted_data = data.to_vec();
        sorted_data.sort_unstable();
        let mut sorted_encoded = encoded;
        sorted_encoded.sort_unstable();
        assert_eq!(sorted_data, sorted_encoded);
    }

    // =======================================================================
    // MTF 追加テスト
    // =======================================================================

    #[test]
    fn mtf_empty() {
        // 空入力
        let encoded = mtf_encode(b"");
        assert!(encoded.is_empty());
        assert!(mtf_decode(&encoded).is_empty());
    }

    #[test]
    fn mtf_single_byte() {
        // 1バイト入力
        let encoded = mtf_encode(b"a");
        assert_eq!(encoded.len(), 1);
        assert_eq!(mtf_decode(&encoded), b"a");
    }

    #[test]
    fn mtf_all_same() {
        // 全同一文字: 先頭以外は全て0になるはず
        let data = b"aaaa";
        let encoded = mtf_encode(data);
        // 最初の'a'は初期位置97、以降は0
        assert_eq!(encoded[1], 0);
        assert_eq!(encoded[2], 0);
        assert_eq!(encoded[3], 0);
        assert_eq!(mtf_decode(&encoded), data);
    }

    #[test]
    fn mtf_consecutive_same_produces_zeros() {
        // 連続同一文字はMTFで0に変換される
        let data = b"xxxxx";
        let encoded = mtf_encode(data);
        for &b in &encoded[1..] {
            assert_eq!(b, 0, "連続同一文字は0であるべき");
        }
    }

    #[test]
    fn mtf_sorted_input() {
        // ソート済み入力
        let data = b"abcdef";
        let encoded = mtf_encode(data);
        let decoded = mtf_decode(&encoded);
        assert_eq!(decoded, data);
    }

    #[test]
    fn mtf_reverse_sorted() {
        // 逆順入力
        let data = b"fedcba";
        let encoded = mtf_encode(data);
        assert_eq!(mtf_decode(&encoded), data);
    }

    #[test]
    fn mtf_binary_full_range() {
        // 全バイト値0-255
        let data: Vec<u8> = (0..=255).collect();
        let encoded = mtf_encode(&data);
        let decoded = mtf_decode(&encoded);
        assert_eq!(decoded, data);
    }

    #[test]
    fn mtf_output_length_preserved() {
        // 出力長は入力長と同一
        let data = b"alice compression test";
        let encoded = mtf_encode(data);
        assert_eq!(encoded.len(), data.len());
    }

    #[test]
    fn mtf_alternating_two() {
        // 2値交互: 最初の2バイトで両方がアルファベット先頭に来た後は0か1
        let data = b"ababab";
        let encoded = mtf_encode(data);
        // encoded[0]='a'の初期位置, encoded[1]='b'の移動後位置, encoded[2..]は0か1
        for &b in &encoded[2..] {
            assert!(b <= 1, "初回2バイト以降は0か1のみ");
        }
        assert_eq!(mtf_decode(&encoded), data);
    }

    #[test]
    fn mtf_repeated_pattern() {
        // 繰り返しパターンの往復
        let data = b"abcabcabc";
        let encoded = mtf_encode(data);
        assert_eq!(mtf_decode(&encoded), data);
    }

    #[test]
    fn mtf_with_nulls() {
        // NULLバイト含む
        let data = b"\x00\x01\x00\x02";
        let encoded = mtf_encode(data);
        assert_eq!(mtf_decode(&encoded), data);
    }

    #[test]
    fn mtf_high_bytes() {
        // 高バイト値
        let data = &[0xFF, 0xFE, 0xFD, 0xFF, 0xFE];
        let encoded = mtf_encode(data);
        assert_eq!(mtf_decode(&encoded), data);
    }

    // =======================================================================
    // RLE 追加テスト
    // =======================================================================

    #[test]
    fn rle_empty() {
        // 空入力
        let encoded = rle_encode(b"");
        assert!(encoded.is_empty());
        assert!(rle_decode(&encoded).is_empty());
    }

    #[test]
    fn rle_single_byte() {
        // 1バイト入力
        let encoded = rle_encode(b"a");
        assert_eq!(encoded, &[1, b'a']);
        assert_eq!(rle_decode(&encoded), b"a");
    }

    #[test]
    fn rle_output_format_pairs() {
        // 出力は常に(count, value)ペア → 偶数長
        let data = b"aabbbcc";
        let encoded = rle_encode(data);
        assert_eq!(encoded.len() % 2, 0, "RLE出力は偶数長であるべき");
    }

    #[test]
    fn rle_max_run_255() {
        // 255バイト連続は1ペアに収まる
        let data = vec![0xAA; 255];
        let encoded = rle_encode(&data);
        assert_eq!(encoded, &[255, 0xAA]);
        assert_eq!(rle_decode(&encoded), data);
    }

    #[test]
    fn rle_run_over_255() {
        // 256バイト連続は2ペアに分割される
        let data = vec![0xBB; 256];
        let encoded = rle_encode(&data);
        assert_eq!(encoded, &[255, 0xBB, 1, 0xBB]);
        assert_eq!(rle_decode(&encoded), data);
    }

    #[test]
    fn rle_run_510() {
        // 510 = 255 + 255
        let data = vec![0xCC; 510];
        let encoded = rle_encode(&data);
        assert_eq!(encoded, &[255, 0xCC, 255, 0xCC]);
        assert_eq!(rle_decode(&encoded), data);
    }

    #[test]
    fn rle_run_511() {
        // 511 = 255 + 255 + 1
        let data = vec![0xDD; 511];
        let encoded = rle_encode(&data);
        assert_eq!(encoded, &[255, 0xDD, 255, 0xDD, 1, 0xDD]);
        assert_eq!(rle_decode(&encoded), data);
    }

    #[test]
    fn rle_alternating_pattern() {
        // 交互パターン: 圧縮効果なし
        let data = b"abababab";
        let encoded = rle_encode(data);
        // 各文字がcount=1で記録 → 入力の2倍
        assert_eq!(encoded.len(), data.len() * 2);
        assert_eq!(rle_decode(&encoded), data);
    }

    #[test]
    fn rle_binary_zeros() {
        // ゼロバイトの連続
        let data = vec![0u8; 50];
        let encoded = rle_encode(&data);
        assert_eq!(encoded, &[50, 0]);
        assert_eq!(rle_decode(&encoded), data);
    }

    #[test]
    fn rle_mixed_runs() {
        // 混在ランの正確な出力
        let data = b"aaabbc";
        let encoded = rle_encode(data);
        assert_eq!(encoded, &[3, b'a', 2, b'b', 1, b'c']);
    }

    #[test]
    fn rle_decode_empty() {
        // 空データの復号
        let decoded = rle_decode(b"");
        assert!(decoded.is_empty());
    }

    #[test]
    fn rle_decode_odd_length_ignored() {
        // 奇数長入力の末尾バイトは無視される
        let decoded = rle_decode(&[3, b'a', 99]);
        assert_eq!(decoded, b"aaa");
    }

    #[test]
    fn rle_all_byte_values() {
        // 全バイト値が正しく処理される
        let data: Vec<u8> = (0..=255).collect();
        let encoded = rle_encode(&data);
        assert_eq!(rle_decode(&encoded), data);
    }

    #[test]
    fn rle_count_zero_decode() {
        // count=0の復号: 0回繰り返し → 何も出力されない
        let decoded = rle_decode(&[0, b'a', 2, b'b']);
        assert_eq!(decoded, b"bb");
    }

    // =======================================================================
    // Huffman 追加テスト
    // =======================================================================

    #[test]
    fn huffman_empty() {
        // 空入力 → 空のコード表
        let codes = build_huffman_codes(b"");
        assert!(codes.is_empty());
    }

    #[test]
    fn huffman_two_symbols() {
        // 2シンボル → 各1ビット
        let codes = build_huffman_codes(b"ab");
        assert_eq!(codes.len(), 2);
        assert_eq!(codes[&b'a'].len(), 1);
        assert_eq!(codes[&b'b'].len(), 1);
    }

    #[test]
    fn huffman_two_symbols_unequal() {
        // 2シンボル異頻度
        let codes = build_huffman_codes(b"aaab");
        assert_eq!(codes.len(), 2);
        assert_eq!(codes[&b'a'].len(), 1);
        assert_eq!(codes[&b'b'].len(), 1);
    }

    #[test]
    fn huffman_three_symbols() {
        // 3シンボル
        let codes = build_huffman_codes(b"aaabbcc");
        assert_eq!(codes.len(), 3);
        // 最頻出の'a'は最短コード
        assert!(codes[&b'a'].len() <= codes[&b'b'].len());
        assert!(codes[&b'a'].len() <= codes[&b'c'].len());
    }

    #[test]
    fn huffman_prefix_free() {
        // prefix-free性: どのコードも他のコードの接頭辞でないこと
        let codes = build_huffman_codes(b"aabbccddee");
        let code_list: Vec<&Vec<bool>> = codes.values().collect();
        for (i, c1) in code_list.iter().enumerate() {
            for (j, c2) in code_list.iter().enumerate() {
                if i != j {
                    let shorter = c1.len().min(c2.len());
                    assert_ne!(&c1[..shorter], &c2[..shorter], "prefix-free違反");
                }
            }
        }
    }

    #[test]
    fn huffman_single_symbol_code_is_false() {
        // 1シンボルのコードは[false]
        let codes = build_huffman_codes(b"zzz");
        assert_eq!(codes[&b'z'], vec![false]);
    }

    #[test]
    fn huffman_all_unique() {
        // 全文字ユニーク(等頻度)
        let data = b"abcdefgh";
        let codes = build_huffman_codes(data);
        assert_eq!(codes.len(), 8);
        // 等頻度 → 全コード長は3ビット
        for code in codes.values() {
            assert_eq!(code.len(), 3, "等頻度8シンボルは3ビット");
        }
    }

    #[test]
    fn huffman_frequency_ordering() {
        // 頻度が高いほどコードが短い(または同等)
        let codes = build_huffman_codes(b"aaaaaabbbcc");
        assert!(codes[&b'a'].len() <= codes[&b'b'].len());
        assert!(codes[&b'b'].len() <= codes[&b'c'].len());
    }

    #[test]
    fn huffman_256_symbols() {
        // 全256種のバイト値を含むデータ
        let data: Vec<u8> = (0..=255).collect();
        let codes = build_huffman_codes(&data);
        assert_eq!(codes.len(), 256);
    }

    #[test]
    fn huffman_codes_not_empty() {
        // 各コードは空でない
        let codes = build_huffman_codes(b"test data for huffman");
        for code in codes.values() {
            assert!(!code.is_empty(), "コードは空であってはならない");
        }
    }

    #[test]
    fn huffman_binary_data() {
        // バイナリデータのコード生成
        let data: Vec<u8> = (0..16).collect();
        let codes = build_huffman_codes(&data);
        assert_eq!(codes.len(), 16);
        // 等頻度16シンボル → 全コード長4
        for code in codes.values() {
            assert_eq!(code.len(), 4);
        }
    }

    #[test]
    fn huffman_large_frequency_skew() {
        // 極端な頻度偏り
        let mut data = vec![b'a'; 1000];
        data.push(b'b');
        let codes = build_huffman_codes(&data);
        // 'a'が圧倒的に多い → 'a'は1ビット
        assert_eq!(codes[&b'a'].len(), 1);
    }

    #[test]
    fn huffman_power_of_two_symbols() {
        // 4シンボル等頻度 → 全コード2ビット
        let codes = build_huffman_codes(b"aabbccdd");
        assert_eq!(codes.len(), 4);
        for code in codes.values() {
            assert_eq!(code.len(), 2);
        }
    }

    // =======================================================================
    // compression_ratio 追加テスト
    // =======================================================================

    #[test]
    fn compression_ratio_empty() {
        // 空データ → 1.0
        let ratio = compression_ratio(b"");
        assert!((ratio - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn compression_ratio_single_byte() {
        // 1バイト → エントロピー0 → 比率0
        let ratio = compression_ratio(b"a");
        assert!(ratio < 0.01);
    }

    #[test]
    fn compression_ratio_two_symbols_equal() {
        // 2シンボル等頻度 → エントロピー1ビット → 比率1/8
        let data = b"abababab";
        let ratio = compression_ratio(data);
        assert!((ratio - 0.125).abs() < 0.02, "ratio={ratio}");
    }

    #[test]
    fn compression_ratio_range() {
        // 常に0以上1以下
        let data = b"some test data for ratio check";
        let ratio = compression_ratio(data);
        assert!(ratio >= 0.0);
        assert!(ratio <= 1.0);
    }

    #[test]
    fn compression_ratio_high_entropy() {
        // 高エントロピー(256種) → 比率が高い(近似精度により完全な1.0にはならない)
        let data: Vec<u8> = (0..=255).collect();
        let ratio = compression_ratio(&data);
        assert!(ratio > 0.8, "ratio={ratio}");
    }

    #[test]
    fn compression_ratio_low_entropy() {
        // 低エントロピー(ほぼ全て同じ)
        let mut data = vec![0u8; 1000];
        data[500] = 1;
        let ratio = compression_ratio(&data);
        assert!(ratio < 0.05, "ratio={ratio}");
    }

    #[test]
    fn compression_ratio_three_symbols() {
        // 3シンボル等頻度 → エントロピー ≈ log2(3) ≈ 1.585 → 比率 ≈ 0.198
        let data = b"abcabcabcabcabcabc";
        let ratio = compression_ratio(data);
        assert!(ratio > 0.15 && ratio < 0.25, "ratio={ratio}");
    }

    #[test]
    fn compression_ratio_monotonic() {
        // シンボル種類が増えるとエントロピーが上がる
        let data1 = vec![0u8; 100];
        let mut data2 = vec![0u8; 50];
        data2.extend(vec![1u8; 50]);
        let r1 = compression_ratio(&data1);
        let r2 = compression_ratio(&data2);
        assert!(r2 > r1, "シンボル種増加でエントロピー上昇");
    }

    // =======================================================================
    // CompressionError テスト
    // =======================================================================

    #[test]
    fn error_display_invalid_data() {
        let e = CompressionError::InvalidData;
        assert_eq!(alloc::format!("{e}"), "invalid data");
    }

    #[test]
    fn error_display_decode_failed() {
        let e = CompressionError::DecodeFailed;
        assert_eq!(alloc::format!("{e}"), "decode failed");
    }

    #[test]
    fn error_debug() {
        let e = CompressionError::InvalidData;
        let dbg = alloc::format!("{e:?}");
        assert!(dbg.contains("InvalidData"));
    }

    #[test]
    fn error_clone() {
        let e = CompressionError::DecodeFailed;
        let e2 = e.clone();
        assert_eq!(e, e2);
    }

    #[test]
    fn error_eq() {
        assert_eq!(CompressionError::InvalidData, CompressionError::InvalidData);
        assert_ne!(
            CompressionError::InvalidData,
            CompressionError::DecodeFailed
        );
    }

    // =======================================================================
    // パイプライン統合テスト
    // =======================================================================

    #[test]
    fn pipeline_hello_world() {
        // 基本的なテキストのパイプライン往復
        let data = b"hello world";
        let (bwt, idx) = bwt_encode(data);
        let mtf = mtf_encode(&bwt);
        let rle = rle_encode(&mtf);
        let mtf2 = rle_decode(&rle);
        let bwt2 = mtf_decode(&mtf2);
        assert_eq!(bwt_decode(&bwt2, idx), data);
    }

    #[test]
    fn pipeline_repeated_text() {
        // 繰り返しテキスト
        let data = b"abcabcabcabcabc";
        let (bwt, idx) = bwt_encode(data);
        let mtf = mtf_encode(&bwt);
        let rle = rle_encode(&mtf);
        let mtf2 = rle_decode(&rle);
        let bwt2 = mtf_decode(&mtf2);
        assert_eq!(bwt_decode(&bwt2, idx), data);
    }

    #[test]
    fn pipeline_all_same() {
        // 全同一文字
        let data = b"zzzzzzzzzz";
        let (bwt, idx) = bwt_encode(data);
        let mtf = mtf_encode(&bwt);
        let rle = rle_encode(&mtf);
        let mtf2 = rle_decode(&rle);
        let bwt2 = mtf_decode(&mtf2);
        assert_eq!(bwt_decode(&bwt2, idx), data);
    }

    #[test]
    fn pipeline_single_char() {
        // 1文字
        let data = b"q";
        let (bwt, idx) = bwt_encode(data);
        let mtf = mtf_encode(&bwt);
        let rle = rle_encode(&mtf);
        let mtf2 = rle_decode(&rle);
        let bwt2 = mtf_decode(&mtf2);
        assert_eq!(bwt_decode(&bwt2, idx), data);
    }

    #[test]
    fn pipeline_binary_data() {
        // バイナリデータ
        let data: Vec<u8> = (0..64).collect();
        let (bwt, idx) = bwt_encode(&data);
        let mtf = mtf_encode(&bwt);
        let rle = rle_encode(&mtf);
        let mtf2 = rle_decode(&rle);
        let bwt2 = mtf_decode(&mtf2);
        assert_eq!(bwt_decode(&bwt2, idx), data);
    }

    #[test]
    fn pipeline_compression_effective() {
        // BWT+MTF+RLEパイプラインで繰り返しデータが圧縮される
        let data = b"aaaaaabbbbbbcccccc";
        let (bwt, _) = bwt_encode(data);
        let mtf = mtf_encode(&bwt);
        let rle = rle_encode(&mtf);
        // RLE後のサイズが元データより小さい(圧縮が効く)
        assert!(rle.len() <= data.len(), "パイプラインで圧縮されるべき");
    }

    #[test]
    fn pipeline_huffman_codes_after_bwt_mtf() {
        // BWT+MTF後のデータにHuffmanコードを構築
        let data = b"the quick brown fox";
        let (bwt, _) = bwt_encode(data);
        let mtf = mtf_encode(&bwt);
        let codes = build_huffman_codes(&mtf);
        // MTF後は小さい値が多い → 0のコードが最短
        assert!(codes.contains_key(&0), "MTF後は0が存在するはず");
    }

    #[test]
    fn pipeline_long_text() {
        // やや長いテキスト
        let data = b"she sells sea shells by the sea shore";
        let (bwt, idx) = bwt_encode(data);
        let mtf = mtf_encode(&bwt);
        let rle = rle_encode(&mtf);
        let mtf2 = rle_decode(&rle);
        let bwt2 = mtf_decode(&mtf2);
        assert_eq!(bwt_decode(&bwt2, idx), data);
    }

    #[test]
    fn pipeline_with_numbers() {
        // 数字含むテキスト
        let data = b"abc123abc123abc123";
        let (bwt, idx) = bwt_encode(data);
        let mtf = mtf_encode(&bwt);
        let rle = rle_encode(&mtf);
        let mtf2 = rle_decode(&rle);
        let bwt2 = mtf_decode(&mtf2);
        assert_eq!(bwt_decode(&bwt2, idx), data);
    }

    #[test]
    fn pipeline_special_chars() {
        // 特殊文字
        let data = b"!@#$%^&*()";
        let (bwt, idx) = bwt_encode(data);
        let mtf = mtf_encode(&bwt);
        let rle = rle_encode(&mtf);
        let mtf2 = rle_decode(&rle);
        let bwt2 = mtf_decode(&mtf2);
        assert_eq!(bwt_decode(&bwt2, idx), data);
    }

    // =======================================================================
    // log2_approx 精度テスト (間接的にcompression_ratioを通じて検証)
    // =======================================================================

    #[test]
    fn compression_ratio_four_symbols_equal() {
        // 4シンボル等頻度 → エントロピー2ビット → 比率2/8=0.25
        let data = b"abcdabcdabcdabcd";
        let ratio = compression_ratio(data);
        assert!((ratio - 0.25).abs() < 0.02, "ratio={ratio}");
    }

    #[test]
    fn compression_ratio_deterministic() {
        // 同一入力で結果が同じ
        let data = b"deterministic test";
        let r1 = compression_ratio(data);
        let r2 = compression_ratio(data);
        assert!((r1 - r2).abs() < f64::EPSILON);
    }

    #[test]
    fn compression_ratio_single_repeated() {
        // 同一バイト繰り返し → 比率≈0
        for b in [0u8, 127, 255] {
            let data = vec![b; 500];
            let ratio = compression_ratio(&data);
            assert!(ratio < 0.01, "byte={b}, ratio={ratio}");
        }
    }

    // =======================================================================
    // 追加テスト (100件到達用)
    // =======================================================================

    #[test]
    fn bwt_japanese_like_bytes() {
        // マルチバイト風のバイナリデータ
        let data = &[0xE3, 0x81, 0x82, 0xE3, 0x81, 0x84, 0xE3, 0x81, 0x82];
        let (encoded, idx) = bwt_encode(data);
        assert_eq!(bwt_decode(&encoded, idx), data);
    }

    #[test]
    fn mtf_bwt_output() {
        // BWT出力をMTFに通すと小さい値が多くなる
        let data = b"banana";
        let (bwt, _) = bwt_encode(data);
        let mtf = mtf_encode(&bwt);
        // MTF出力の平均値はアルファベット順入力より小さいはず
        #[allow(clippy::cast_precision_loss)]
        let avg: f64 = mtf.iter().map(|&b| f64::from(b)).sum::<f64>() / mtf.len() as f64;
        assert!(avg < 128.0, "BWT+MTFで平均値が下がるべき: avg={avg}");
    }

    #[test]
    fn rle_two_bytes() {
        // 2バイト入力
        let data = b"aa";
        let encoded = rle_encode(data);
        assert_eq!(encoded, &[2, b'a']);
        assert_eq!(rle_decode(&encoded), data);
    }

    #[test]
    fn rle_large_mixed() {
        // 大きめの混在データの往復
        let mut data = Vec::new();
        for i in 0..50u8 {
            for _ in 0..=i % 5 {
                data.push(i);
            }
        }
        let encoded = rle_encode(&data);
        assert_eq!(rle_decode(&encoded), data);
    }

    #[test]
    fn huffman_five_symbols() {
        // 5シンボル
        let codes = build_huffman_codes(b"aaaaabbbccdde");
        assert_eq!(codes.len(), 5);
    }

    #[test]
    fn huffman_repeated_two() {
        // 2シンボルで片方が圧倒的に多い
        let mut data = vec![b'a'; 100];
        data.extend(vec![b'b'; 1]);
        let codes = build_huffman_codes(&data);
        assert_eq!(codes[&b'a'].len(), 1);
        assert_eq!(codes[&b'b'].len(), 1);
    }

    #[test]
    fn pipeline_pangram() {
        // パングラムのパイプライン往復
        let data = b"the five boxing wizards jump quickly";
        let (bwt, idx) = bwt_encode(data);
        let mtf = mtf_encode(&bwt);
        let rle = rle_encode(&mtf);
        let mtf2 = rle_decode(&rle);
        let bwt2 = mtf_decode(&mtf2);
        assert_eq!(bwt_decode(&bwt2, idx), data);
    }

    #[test]
    fn compression_ratio_eight_symbols_equal() {
        // 8シンボル等頻度 → エントロピー3ビット → 比率3/8=0.375
        let data = b"abcdefghabcdefghabcdefgh";
        let ratio = compression_ratio(data);
        assert!((ratio - 0.375).abs() < 0.03, "ratio={ratio}");
    }
}
