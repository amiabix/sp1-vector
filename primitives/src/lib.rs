use alloy_primitives::B256;
use codec::{Compact, Decode, Encode};

pub mod consts;
pub mod header_range;
mod justification;
pub mod merkle;
pub mod rotate;
pub mod types;
mod zisk_blake2b;
mod zisk_sha256;

pub use zisk_sha256::sha256_hash;

pub use justification::verify_justification;

/// Blake2B hash of an encoded header. Note: This is a generic hash fn for any data.
pub(crate) fn hash_encoded_header(encoded_header: &[u8]) -> B256 {
    B256::from(zisk_blake2b::blake2b_256(encoded_header))
}

/// Compute the new authority set hash from the encoded pubkeys.
pub fn compute_authority_set_commitment(pubkeys: &[B256]) -> B256 {
    let mut commitment_so_far = sha256_hash(pubkeys[0].as_slice());
    for pubkey in pubkeys.iter().skip(1) {
        let mut input = [0u8; 64];
        input[..32].copy_from_slice(&commitment_so_far);
        input[32..].copy_from_slice(pubkey.as_slice());
        commitment_so_far = sha256_hash(&input);
    }
    B256::from(commitment_so_far)
}

/// Decode a SCALE-encoded compact int and get the value and the number of bytes it took to encode.
pub(crate) fn decode_scale_compact_int(bytes: Vec<u8>) -> (u64, usize) {
    let value = Compact::<u64>::decode(&mut bytes.as_slice())
        .expect("Failed to decode SCALE-encoded compact int.");
    (value.into(), value.encoded_size())
}

#[cfg(test)]
mod tests {
    use super::*;
    use avail_subxt::ext::avail_rust_core::header::Digest;
    use avail_subxt::ext::avail_rust_core::header::HeaderExtension;
    use avail_subxt::ext::avail_rust_core::header::V3HeaderExtension;
    use avail_subxt::AvailHeader;
    use avail_subxt::CompactDataLookup;
    use avail_subxt::KateCommitment;
    use avail_subxt::H256;
    use codec::{Compact, Encode};
    use sha2::{Digest as Sha256Digest, Sha256};

    #[test]
    fn test_decode_scale_compact_int() {
        let nums = [
            u32::MIN,
            1u32,
            63u32,
            64u32,
            16383u32,
            16384u32,
            1073741823u32,
            1073741824u32,
            4294967295u32,
            u32::MAX,
        ];
        let encoded_nums: Vec<Vec<u8>> = nums.iter().map(|num| Compact(*num).encode()).collect();
        let zipped: Vec<(&Vec<u8>, &u32)> = encoded_nums.iter().zip(nums.iter()).collect();
        for (encoded_num, num) in zipped {
            let (value, _) = decode_scale_compact_int(encoded_num.to_vec());
            assert_eq!(value, *num as u64);
        }
    }

    #[test]
    fn test_header_parent_hash_extracting() {
        let hash = H256::random();
        let h = AvailHeader {
            parent_hash: hash,
            number: 1,
            state_root: H256::zero(),
            extrinsics_root: H256::zero(),
            extension: HeaderExtension::V3(V3HeaderExtension {
                app_lookup: CompactDataLookup {
                    size: 0,
                    index: Vec::default(),
                },
                commitment: KateCommitment {
                    rows: 0,
                    cols: 0,
                    commitment: Vec::default(),
                    data_root: H256::default(),
                },
            }),
            digest: Digest {
                ..Default::default()
            },
        };

        let encoded = h.encode();

        let n: [u8; 32] = encoded[0..32].try_into().unwrap();
        let extracted_hash = H256::from(n);
        assert_eq!(extracted_hash, hash, "Hashes don't match")
    }

    #[test]
    fn proof_output_encoding_binds_type_and_payload() {
        use crate::consts::{HEADER_OUTPUTS_LENGTH, PROOF_OUTPUT_LENGTH, ROTATE_OUTPUTS_LENGTH};
        use crate::types::{encode_proof_output, ProofType};

        let header = [0u8; HEADER_OUTPUTS_LENGTH];
        let rotate = [0u8; ROTATE_OUTPUTS_LENGTH];
        let range = encode_proof_output(ProofType::HeaderRangeProof, header, rotate);
        let rotation = encode_proof_output(ProofType::RotateProof, header, rotate);

        assert_eq!(range.len(), PROOF_OUTPUT_LENGTH);
        assert_ne!(Sha256::digest(&range), Sha256::digest(&rotation));

        let mut changed_header = header;
        changed_header[0] = 1;
        let changed = encode_proof_output(ProofType::HeaderRangeProof, changed_header, rotate);
        assert_ne!(Sha256::digest(&range), Sha256::digest(&changed));
    }
}
