use ziskos::syscalls::syscall_sha256_f;

const INITIAL_STATE: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

/// SHA-256 using aligned words for ZisK's compression syscall.
pub fn sha256_hash(input: &[u8]) -> [u8; 32] {
    let mut state = core::array::from_fn(|i| {
        u64::from(INITIAL_STATE[2 * i]) | (u64::from(INITIAL_STATE[2 * i + 1]) << 32)
    });

    let mut chunks = input.chunks_exact(64);
    for block in &mut chunks {
        compress(&mut state, block);
    }

    let remaining = chunks.remainder();
    let mut tail = [0u8; 128];
    tail[..remaining.len()].copy_from_slice(remaining);
    tail[remaining.len()] = 0x80;
    let tail_len = if remaining.len() >= 56 { 128 } else { 64 };
    tail[tail_len - 8..tail_len]
        .copy_from_slice(&(input.len() as u64).wrapping_mul(8).to_be_bytes());
    for block in tail[..tail_len].chunks_exact(64) {
        compress(&mut state, block);
    }

    let mut digest = [0u8; 32];
    for (i, word) in digest.chunks_exact_mut(4).enumerate() {
        let value = (state[i / 2] >> ((i % 2) * 32)) as u32;
        word.copy_from_slice(&value.to_be_bytes());
    }
    digest
}

fn compress(state: &mut [u64; 4], block: &[u8]) {
    let input =
        core::array::from_fn(|i| u64::from_le_bytes(block[i * 8..(i + 1) * 8].try_into().unwrap()));
    syscall_sha256_f(state, &input);
}

#[cfg(test)]
mod tests {
    use super::sha256_hash;
    use sha2::{Digest, Sha256};

    #[test]
    fn matches_sha2_for_unaligned_inputs_and_block_boundaries() {
        for length in [0, 1, 55, 56, 63, 64, 65, 127, 128, 129, 1024] {
            let mut bytes = vec![0u8];
            bytes.extend((0..length).map(|i| (i * 17 + 3) as u8));
            let input = &bytes[1..];
            let expected = Sha256::digest(input);
            assert_eq!(
                sha256_hash(input).as_slice(),
                expected.as_slice(),
                "length {length}"
            );
        }
    }
}
