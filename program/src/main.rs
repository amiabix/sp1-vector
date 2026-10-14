//! A simple program to be proven inside the zkVM.

#![no_main]
ziskos::entrypoint!(main);

use sp1_vector_primitives::{
    consts::HEADER_OUTPUTS_LENGTH,
    consts::ROTATE_OUTPUTS_LENGTH,
    header_range::verify_header_range,
    rotate::verify_rotate,
    sha256_hash,
    types::{encode_proof_output, HeaderRangeInputs, ProofType, RotateInputs},
};

/// Generate a Vector proof for a given proof type.
pub fn main() {
    // Read the proof type requested from the inputs.
    let proof_type: ProofType = ziskos::io::read();

    let mut header_range_outputs = [0u8; HEADER_OUTPUTS_LENGTH];
    let mut rotate_outputs = [0u8; ROTATE_OUTPUTS_LENGTH];

    match proof_type {
        ProofType::HeaderRangeProof => {
            // Read the header range inputs from the inputs.
            let header_range_inputs = ziskos::io::read::<HeaderRangeInputs>();
            header_range_outputs = verify_header_range(header_range_inputs);
        }
        ProofType::RotateProof => {
            // Read the rotate inputs from the inputs.
            let rotate_inputs = ziskos::io::read::<RotateInputs>();
            rotate_outputs = verify_rotate(rotate_inputs);
        }
    }

    // ZisK has 256 public-output bytes, so bind the full ABI output by its digest.
    let output = encode_proof_output(proof_type, header_range_outputs, rotate_outputs);
    ziskos::io::commit_slice(&sha256_hash(&output));
}
