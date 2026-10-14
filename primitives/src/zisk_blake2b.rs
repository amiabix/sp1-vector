// BLAKE2b-256 framing around ZisK's accelerated BLAKE2b compression function.
const IV: [u64; 8] = [
    0x6a09e667f3bcc908,
    0xbb67ae8584caa73b,
    0x3c6ef372fe94f82b,
    0xa54ff53a5f1d36f1,
    0x510e527fade682d1,
    0x9b05688c2b3e6c1f,
    0x1f83d9abfb41bd6b,
    0x5be0cd19137e2179,
];

pub(crate) fn blake2b_256(input: &[u8]) -> [u8; 32] {
    let mut state = IV;
    state[0] ^= 0x0101_0020; // Unkeyed, sequential BLAKE2b with a 32-byte output.

    let mut offset = 0;
    while input.len() - offset > 128 {
        compress(
            &mut state,
            &input[offset..offset + 128],
            (offset + 128) as u64,
            false,
        );
        offset += 128;
    }
    compress(&mut state, &input[offset..], input.len() as u64, true);

    let mut digest = [0u8; 32];
    for (chunk, word) in digest.chunks_exact_mut(8).zip(state) {
        chunk.copy_from_slice(&word.to_le_bytes());
    }
    digest
}

fn compress(state: &mut [u64; 8], input: &[u8], bytes_processed: u64, last: bool) {
    let mut block = [0u8; 128];
    block[..input.len()].copy_from_slice(input);
    let words =
        core::array::from_fn(|i| u64::from_le_bytes(block[i * 8..(i + 1) * 8].try_into().unwrap()));
    ziskos::zisklib::blake2b_compress(12, state, &words, &[bytes_processed, 0], last);
}

#[cfg(test)]
mod tests {
    use super::blake2b_256;
    use blake2::{
        digest::{Update, VariableOutput},
        Blake2bVar,
    };

    #[test]
    fn matches_rustcrypto_across_block_boundaries() {
        for length in [0, 1, 32, 127, 128, 129, 256, 431, 1024] {
            let input: Vec<u8> = (0..length).map(|i| (i * 17 + 3) as u8).collect();
            let mut reference = Blake2bVar::new(32).unwrap();
            reference.update(&input);
            let mut expected = [0u8; 32];
            reference.finalize_variable(&mut expected).unwrap();
            assert_eq!(blake2b_256(&input), expected, "input length {length}");
        }
    }
}
