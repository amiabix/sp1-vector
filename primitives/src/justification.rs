use crate::{hash_encoded_header, types::CircuitJustification};
use codec::Encode;
use std::collections::{HashMap, HashSet};
use ziskos::zisklib::{
    add_points_ed25519, decompress_ed25519, double_scalar_mul_with_base_ed25519,
    is_canonical_scalar_ed25519, is_small_order_ed25519, le_bytes_to_u64_4, neg_point_ed25519,
    reduce_wide_scalar_ed25519, Sha512,
};

use alloy_primitives::B256;

/// ZIP-215 verification, matching Avail's existing Ed25519 consensus rule.
fn verify_signature(pubkey_bytes: [u8; 32], signed_message: &[u8], signature: [u8; 64]) -> bool {
    let r_bytes: [u8; 32] = signature[..32].try_into().unwrap();
    let s = le_bytes_to_u64_4(signature[32..].try_into().unwrap());
    if !is_canonical_scalar_ed25519(&s) {
        return false;
    }
    let (Some(a), Some(r)) = (
        decompress_ed25519(&pubkey_bytes),
        decompress_ed25519(&r_bytes),
    ) else {
        return false;
    };

    let mut h = Sha512::new();
    h.update(&r_bytes);
    h.update(&pubkey_bytes);
    h.update(signed_message);
    let k = reduce_wide_scalar_ed25519(&h.finalize());
    let r_prime = double_scalar_mul_with_base_ed25519(&k, &neg_point_ed25519(&a), &s);

    // ZIP-215 accepts R when R - R' has order dividing the cofactor (8).
    is_small_order_ed25519(&add_points_ed25519(&r, &neg_point_ed25519(&r_prime)))
}

/// Confirm ancestry of a child block by traversing the ancestry_map until root_hash is reached.
/// Sourced from https://github.com/availproject/avail-light/blob/main/core/src/finality.rs with some
/// small refactors for readability.  
fn confirm_ancestry(
    child_hash: &B256,
    root_hash: &B256,
    ancestry_map: &HashMap<B256, B256>,
) -> bool {
    let mut current_hash = child_hash;

    while current_hash != root_hash {
        match ancestry_map.get(current_hash) {
            Some(parent_hash) => current_hash = parent_hash,
            None => return false,
        }
    }

    true
}

/// Determine if a supermajority is achieved.
fn is_signed_by_supermajority(num_signatures: usize, validator_set_size: usize) -> bool {
    num_signatures * 3 > validator_set_size * 2
}

/// Verify a justification on a block from the specified authority set. Confirms that a supermajority
/// of the validator set is achieved on the specific block. Sourced from
/// https://github.com/availproject/avail-light/blob/main/core/src/finality.rs with some minor
/// modifications to fit into SP1 Vector, and small refactors for readability.
pub fn verify_justification(justification: &CircuitJustification) {
    // 1. Form an ancestry map from votes_ancestries in the justification. This maps header hashes to their parents' hashes.
    // Since we only get encoded headers, ensure that the parent is contained in the encoded header, no need to decode it.
    let ancestry_map: HashMap<B256, B256> = justification
        .ancestries_encoded
        .iter()
        .map(|encoded_header| {
            let parent_hash_array: [u8; 32] = encoded_header[0..32].try_into().unwrap();
            let parent_hash = B256::from(parent_hash_array);
            let header_hash = hash_encoded_header(encoded_header);

            (header_hash, parent_hash.to_owned())
        })
        .collect();

    // 2. Get the signer addresses of the accounts with valid precommits for the justification. Invalidate
    // precommits from the same signer address with a set.
    let signer_addresses: HashSet<B256> = justification
        .precommits
        .iter()
        .filter_map(|p| {
            // Form the message which is signed in the Justification.
            // Combination of the precommit flag, block data, round number and set_id.
            let signed_message = Encode::encode(&(
                1u8,
                p.target_hash.0,
                p.target_number,
                &justification.round,
                &justification.authority_set_id,
            ));

            // Verify the signature is valid on the precommit, and panic if this is not the case.
            assert!(
                verify_signature(p.pubkey.0, &signed_message, p.signature.0),
                "Failed to verify Ed25519 signature."
            );

            // Confirm the ancestry of the child block.
            let ancestry_confirmed =
                confirm_ancestry(&p.target_hash, &justification.block_hash, &ancestry_map);

            if ancestry_confirmed {
                Some(p.pubkey)
            } else {
                None
            }
        })
        .collect();

    // 3. Count the accounts which are in validator set of the justification.
    let num_matched_addresses = signer_addresses
        .iter()
        .filter(|x| justification.valset_pubkeys.iter().any(|e| e.0.eq(&x[..])))
        .count();

    // 4. Confirm that the supermajority of the validator set is achieved.
    assert!(
        is_signed_by_supermajority(num_matched_addresses, justification.valset_pubkeys.len()),
        "More than 2/3 of signatures are not verifie!"
    );
}

#[cfg(test)]
mod tests {
    use super::verify_signature;
    use ed25519_consensus::{Signature, SigningKey, VerificationKey};

    fn assert_matches_consensus(pk: [u8; 32], msg: &[u8], sig: [u8; 64]) -> bool {
        let expected = VerificationKey::try_from(pk)
            .and_then(|key| key.verify(&Signature::from(sig), msg))
            .is_ok();
        assert_eq!(verify_signature(pk, msg, sig), expected);
        expected
    }

    #[test]
    fn signature_verification_matches_consensus() {
        let signer = SigningKey::from([7u8; 32]);
        let pk = signer.verification_key().to_bytes();
        let msg = b"Avail GRANDPA precommit";
        let sig = signer.sign(msg).to_bytes();
        assert!(assert_matches_consensus(pk, msg, sig));
        assert!(!assert_matches_consensus(pk, b"changed precommit", sig));

        let mut noncanonical_s = sig;
        noncanonical_s[63] = 0xff;
        assert!(!assert_matches_consensus(pk, msg, noncanonical_s));
    }

    #[test]
    fn accepts_zip215_noncanonical_identity() {
        let mut identity = [0u8; 32];
        identity[0] = 1;
        let mut noncanonical_identity = [0xffu8; 32];
        noncanonical_identity[0] = 0xee;
        noncanonical_identity[31] = 0x7f;

        let mut sig = [0u8; 64];
        sig[..32].copy_from_slice(&noncanonical_identity);
        assert!(assert_matches_consensus(identity, b"cofactor case", sig));
        assert!(!ziskos::zisklib::ed25519_verify(
            &identity,
            b"cofactor case",
            &sig
        ));

        sig[..32].copy_from_slice(&identity);
        assert!(assert_matches_consensus(
            noncanonical_identity,
            b"noncanonical key",
            sig
        ));
    }
}
