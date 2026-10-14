pub mod relay;

use alloy::primitives::B256;
use anyhow::{bail, ensure, Context, Result};
use zisk_sdk::{GuestProgram, Proof, ProofBody};

pub const ZISK_VECTOR_ELF: &[u8] = include_bytes!("../../elf/vector-elf");

pub fn zisk_vector_program() -> GuestProgram {
    GuestProgram::from_bytes("vector-elf", ZISK_VECTOR_ELF.to_vec())
}

pub fn zisk_vector_vkey(program: &GuestProgram) -> Result<B256> {
    let vk = program
        .vk()
        .context("Run ZisK setup for the Vector ELF first")?;
    ensure!(vk.vk.len() == 4, "Unexpected ZisK program vkey length");
    let mut bytes = [0u8; 32];
    for (i, word) in vk.vk.iter().enumerate() {
        bytes[i * 8..(i + 1) * 8].copy_from_slice(&word.to_be_bytes());
    }
    Ok(B256::from(bytes))
}

pub fn plonk_calldata(proof: &Proof) -> Result<(Vec<u8>, Vec<u8>)> {
    match &proof.body {
        ProofBody::Plonk {
            proof_bytes,
            publics_full,
            ..
        } => {
            ensure!(
                publics_full.len() == 68,
                "Unexpected ZisK PLONK public length"
            );
            // The SNARK hashes full-width words; the u32 public view loses high bits.
            let public_values = publics_full[4..]
                .iter()
                .flat_map(|word| word.to_le_bytes())
                .collect();
            Ok((proof_bytes.clone(), public_values))
        }
        _ => bail!("ZisK prover returned a non-PLONK proof"),
    }
}
