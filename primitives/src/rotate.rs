use crate::{
    compute_authority_set_commitment,
    consts::ROTATE_OUTPUTS_LENGTH,
    hash_encoded_header,
    types::{RotateInputs, RotateOutputs},
    verify_justification,
};
use alloy_primitives::B256;
use alloy_sol_types::SolType;
use codec::{Compact, Decode};

/// Verify the justification from the current authority set on the epoch end header and return the next
/// authority set commitment.
pub fn verify_rotate(rotate_inputs: RotateInputs) -> [u8; ROTATE_OUTPUTS_LENGTH] {
    // Verify the provided justification is valid.
    verify_justification(&rotate_inputs.justification);

    let expected_block_hash = hash_encoded_header(&rotate_inputs.header_rotate_data.header_bytes);

    // The header hash should match the block hash signed by the justification.
    assert_eq!(expected_block_hash, rotate_inputs.justification.block_hash);

    // Extract the public keys of the next validator set from the epoch end header.
    let next_validator_pubkeys = get_next_validator_pubkeys_from_epoch_end_header(
        &rotate_inputs.header_rotate_data.header_bytes,
        rotate_inputs.header_rotate_data.consensus_log_position,
    );

    // Compute the current authority set hash from the public keys used in the justification.
    let current_authority_set_hash =
        compute_authority_set_commitment(&rotate_inputs.justification.valset_pubkeys);

    // Compute the next authority set hash from the public keys that are encoded in the epoch end header.
    let next_authority_set_hash = compute_authority_set_commitment(&next_validator_pubkeys);

    // Return the ABI encoded RotateOutputs.
    RotateOutputs::abi_encode(&(
        rotate_inputs.justification.authority_set_id,
        current_authority_set_hash,
        next_authority_set_hash,
    ))
    .try_into()
    .unwrap()
}

/// Extract the scheduled GRANDPA change from a SCALE-encoded Avail header.
pub fn get_next_validator_pubkeys_from_epoch_end_header(
    header_bytes: &[u8],
    consensus_log_position: usize,
) -> Vec<B256> {
    let mut cursor = header_bytes.get(32..).expect("Missing header number");
    Compact::<u32>::decode(&mut cursor).expect("Invalid header number");
    cursor = cursor.get(64..).expect("Missing header roots");
    let log_count = Compact::<u32>::decode(&mut cursor)
        .expect("Invalid digest length")
        .0;

    for _ in 0..log_count {
        let item_start = header_bytes.len() - cursor.len();
        let variant = u8::decode(&mut cursor).expect("Missing digest item");
        let consensus = match variant {
            0 => {
                Vec::<u8>::decode(&mut cursor).expect("Invalid Other digest");
                None
            }
            4..=6 => {
                let engine = <[u8; 4]>::decode(&mut cursor).expect("Missing consensus ID");
                let data = Vec::<u8>::decode(&mut cursor).expect("Invalid consensus data");
                (variant == 4).then_some((engine, data))
            }
            8 => None,
            _ => panic!("Unknown digest item"),
        };

        if consensus_log_position.checked_add(1) != Some(item_start) {
            continue;
        }
        let (engine, data) = consensus.expect("Selected digest item is not a consensus log");
        assert_eq!(engine, *b"FRNK");
        let mut data = data.as_slice();
        assert_eq!(u8::decode(&mut data).expect("Missing GRANDPA log type"), 1);
        let authority_count = Compact::<u32>::decode(&mut data)
            .expect("Invalid authority count")
            .0 as usize;
        assert!(authority_count > 0 && authority_count <= data.len() / 40);
        let mut pubkeys = Vec::with_capacity(authority_count);
        for _ in 0..authority_count {
            let pubkey = <[u8; 32]>::decode(&mut data).expect("Missing authority public key");
            assert_eq!(u64::decode(&mut data).expect("Missing authority weight"), 1);
            pubkeys.push(B256::from(pubkey));
        }
        assert_eq!(u32::decode(&mut data).expect("Missing change delay"), 0);
        assert!(data.is_empty(), "Trailing GRANDPA log data");
        return pubkeys;
    }
    panic!("Selected GRANDPA consensus log is not a digest item")
}

#[cfg(test)]
mod tests {
    use super::get_next_validator_pubkeys_from_epoch_end_header;
    use alloy_primitives::B256;
    use avail_subxt::ext::subxt_core::config::substrate::DigestItem;
    use codec::{Compact, Encode};

    fn scheduled_change() -> Vec<u8> {
        let mut data = vec![1, 4]; // ScheduledChange with one authority.
        data.extend([7u8; 32]);
        data.extend(1u64.to_le_bytes());
        data.extend(0u32.to_le_bytes());
        data
    }

    fn header_with_logs(logs: Vec<DigestItem>) -> Vec<u8> {
        let mut header = vec![0u8; 32];
        header.extend(Compact(1u32).encode());
        header.extend([0u8; 64]);
        header.extend(logs.encode());
        header
    }

    #[test]
    #[should_panic(expected = "Selected GRANDPA consensus log is not a digest item")]
    fn rejects_scheduled_change_bytes_inside_other_digest_item() {
        let fake_log = DigestItem::Consensus(*b"FRNK", scheduled_change()).encode();
        let mut payload = vec![0u8];
        payload.extend(fake_log);
        let header = header_with_logs(vec![DigestItem::Other(payload)]);
        let offset = header
            .windows(6)
            .position(|bytes| bytes == [0, 4, b'F', b'R', b'N', b'K'])
            .unwrap();
        get_next_validator_pubkeys_from_epoch_end_header(&header, offset);
    }

    #[test]
    fn accepts_scheduled_change_after_another_digest_item() {
        let other = DigestItem::Other(vec![9]);
        let header = header_with_logs(vec![
            other.clone(),
            DigestItem::Consensus(*b"FRNK", scheduled_change()),
        ]);
        let offset = 32 + Compact(1u32).encode().len() + 64 + 1 + other.encode().len() - 1;
        assert_eq!(
            get_next_validator_pubkeys_from_epoch_end_header(&header, offset),
            vec![B256::from([7u8; 32])]
        );
    }
}
