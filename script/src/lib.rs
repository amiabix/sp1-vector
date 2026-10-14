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

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use services::input::RpcDataFetcher;
    use sha2::{Digest, Sha256};
    use zisk_sdk::{Proof, ZiskClient, ZiskStdin};

    #[test]
    fn saved_inputs_execute_with_current_guest() -> Result<()> {
        let client = ZiskClient::embedded().execute_only().build()?;
        let program = super::zisk_vector_program();
        client.setup(&program, false)?;
        for fixture in ["header-range-turing-4007140-4007141", "rotate-synthetic"] {
            let path = format!("{}/test_assets/{fixture}", env!("CARGO_MANIFEST_DIR"));
            let stdin = ZiskStdin::from_file(format!("{path}.bin"))?;
            let report = client.execute(&program, stdin, None)?;
            let mut committed = [0u8; 32];
            report.get_public_values_slice(&mut committed);
            let output = std::fs::read(format!("{path}-output.bin"))?;
            assert_eq!(committed.as_slice(), Sha256::digest(output).as_slice());
        }
        Ok(())
    }

    #[test]
    fn plonk_calldata_matches_zisk_export() -> Result<()> {
        for fixture in ["header-range-turing-4007140-4007141", "rotate-synthetic"] {
            let path = format!("{}/test_assets/{fixture}", env!("CARGO_MANIFEST_DIR"));
            let proof = Proof::load(format!("{path}-plonk.proof"))?;
            let export: serde_json::Value =
                serde_json::from_slice(&std::fs::read(format!("{path}-solidity.json"))?)?;
            let (proof_bytes, public_values) = super::plonk_calldata(&proof)?;
            assert_eq!(
                hex::encode(proof_bytes),
                export["proofBytes"]
                    .as_str()
                    .unwrap()
                    .trim_start_matches("0x")
            );
            assert_eq!(
                hex::encode(public_values),
                export["publicValues"]
                    .as_str()
                    .unwrap()
                    .trim_start_matches("0x")
            );
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_get_justification_query_service() -> Result<()> {
        let client = RpcDataFetcher::new().await;
        let justification = client.get_justification(2075796).await?;
        println!("Justification: {:?}", justification);
        Ok(())
    }
}
