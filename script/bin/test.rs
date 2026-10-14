//! A simple script to test the generation of proofs.

use alloy::primitives::{B256, B512};
use anyhow::{ensure, Context};
use avail_subxt::{
    ext::avail_rust_core::{
        grandpa::{AuthorityId, ConsensusLog, ScheduledChange},
        header::{Digest as HeaderDigest, HeaderExtension, V3HeaderExtension},
    },
    ext::subxt_core::config::substrate::DigestItem,
    AvailHeader, CompactDataLookup, KateCommitment, H256,
};
use clap::Parser;
use codec::{Compact, Encode};
use ed25519_consensus::SigningKey;
use services::input::{HeaderRangeRequestData, RpcDataFetcher};
use sha2::{Digest, Sha256};
use sp1_vector_primitives::{
    consts::{HEADER_OUTPUTS_LENGTH, ROTATE_OUTPUTS_LENGTH},
    header_range::verify_header_range,
    rotate::verify_rotate,
    types::{
        encode_proof_output, CircuitJustification, HeaderRangeInputs, HeaderRotateData, Precommit,
        ProofType, RotateInputs,
    },
};
use sp1_vectorx_script::zisk_vector_program;
use std::path::PathBuf;
use zisk_sdk::{ZiskClient, ZiskStdin};

// Live input requires AVAIL_URL and AVAIL_CHAIN_ID. A justification indexer is
// needed only when --epoch-end and --recent-finality are both absent.

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct ScriptArgs {
    /// Trusted block.
    #[clap(long)]
    trusted_block: Option<u32>,

    /// Target block.
    #[clap(long, env)]
    target_block: Option<u32>,

    /// Save the guest input for cargo-zisk execute/prove.
    #[clap(long)]
    save_input: Option<PathBuf>,

    /// Save the ABI-encoded Vector output bound by the guest digest.
    #[clap(long)]
    save_output: Option<PathBuf>,

    /// Run a previously saved guest input without RPC access.
    #[clap(long)]
    input_file: Option<PathBuf>,

    /// Read finality proof from Avail RPC for an epoch-end block.
    #[clap(long)]
    epoch_end: bool,

    /// Use a recent Avail finality proof without an external justification indexer.
    #[clap(long)]
    recent_finality: bool,

    /// Generate an authority-set rotation input from Avail RPC.
    #[clap(long)]
    rotate_authority_set: Option<u64>,

    /// Generate a deterministic signed rotation for testing without Avail RPC.
    #[clap(long)]
    synthetic_rotate: bool,

    /// Use the Rust emulator when Assembly is unavailable.
    #[clap(long)]
    emulator: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = ScriptArgs::parse();

    let stdin = if let Some(path) = args.input_file {
        ZiskStdin::from_file(path)?
    } else {
        let stdin = ZiskStdin::new();
        if args.synthetic_rotate {
            stdin.write(&ProofType::RotateProof);
            stdin.write(&synthetic_rotate_inputs());
        } else if let Some(set_id) = args.rotate_authority_set {
            let fetcher = RpcDataFetcher::new().await;
            stdin.write(&ProofType::RotateProof);
            stdin.write(&fetcher.get_rotate_inputs(set_id).await);
        } else {
            let fetcher = RpcDataFetcher::new().await;
            let (trusted_block, target_block, epoch_end) = if args.recent_finality {
                let requested = fetcher.get_head().await.number.saturating_sub(50);
                let target = fetcher
                    .get_justification_data_for_block_unsafe(requested)
                    .await?
                    .commit
                    .target_number;
                ensure!(target > 0, "finality proof has no target block");
                (target - 1, target, true)
            } else {
                (
                    args.trusted_block.context("--trusted-block is required")?,
                    args.target_block.context("--target-block is required")?,
                    args.epoch_end,
                )
            };
            println!("Proving header range {trusted_block}..={target_block}");
            let header_range_inputs = fetcher
                .get_header_range_inputs(
                    HeaderRangeRequestData {
                        trusted_block,
                        target_block,
                        is_target_epoch_end_block: epoch_end,
                    },
                    Some(512),
                )
                .await;
            stdin.write(&ProofType::HeaderRangeProof);
            stdin.write(&header_range_inputs);
        }
        stdin
    };

    let proof_type: ProofType = stdin.read()?;
    let output = match proof_type {
        ProofType::HeaderRangeProof => encode_proof_output(
            proof_type,
            verify_header_range(stdin.read::<HeaderRangeInputs>()?),
            [0u8; ROTATE_OUTPUTS_LENGTH],
        ),
        ProofType::RotateProof => encode_proof_output(
            proof_type,
            [0u8; HEADER_OUTPUTS_LENGTH],
            verify_rotate(stdin.read::<RotateInputs>()?),
        ),
    };
    if let Some(path) = args.save_output {
        std::fs::write(path, &output)?;
    }
    stdin.reset();

    if let Some(path) = args.save_input {
        stdin.save(path)?;
    }
    let client = if args.emulator {
        ZiskClient::embedded().execute_only().build()?
    } else {
        ZiskClient::embedded().assembly().execute_only().build()?
    };
    let program = zisk_vector_program();
    client.setup(&program, false)?;
    let report = client.execute(&program, stdin, None)?;

    let mut committed = [0u8; 32];
    report.get_public_values_slice(&mut committed);
    ensure!(committed.as_slice() == Sha256::digest(output).as_slice());

    println!("Total steps: {}", report.get_execution_steps());

    Ok(())
}

fn synthetic_rotate_inputs() -> RotateInputs {
    let current = SigningKey::from([7u8; 32]);
    let next = SigningKey::from([8u8; 32]);
    let current_pubkey = B256::from(current.verification_key().to_bytes());
    let header = AvailHeader {
        parent_hash: H256::zero(),
        number: 1,
        state_root: H256::zero(),
        extrinsics_root: H256::zero(),
        extension: HeaderExtension::V3(V3HeaderExtension {
            app_lookup: CompactDataLookup {
                size: 0,
                index: Vec::new(),
            },
            commitment: KateCommitment {
                rows: 0,
                cols: 0,
                commitment: Vec::new(),
                data_root: H256::zero(),
            },
        }),
        digest: HeaderDigest {
            logs: vec![DigestItem::Consensus(
                *b"FRNK",
                ConsensusLog::<u32>::ScheduledChange(ScheduledChange {
                    next_authorities: vec![(AuthorityId(next.verification_key().to_bytes()), 1)],
                    delay: 0,
                })
                .encode(),
            )],
        },
    };
    let header_bytes = header.encode();
    let block_hash = B256::from_slice(header.hash().as_bytes());
    let round = 1u64;
    let authority_set_id = 1u64;
    let signed_message = (1u8, block_hash.0, header.number, round, authority_set_id).encode();
    RotateInputs {
        justification: CircuitJustification {
            round,
            authority_set_id,
            valset_pubkeys: vec![current_pubkey],
            precommits: vec![Precommit {
                target_number: header.number,
                target_hash: block_hash,
                pubkey: current_pubkey,
                signature: B512::from(current.sign(&signed_message).to_bytes()),
            }],
            block_hash,
            ancestries_encoded: Vec::new(),
        },
        header_rotate_data: HeaderRotateData {
            header_bytes,
            consensus_log_position: 32 + Compact(header.number).encode().len() + 32 + 32,
        },
    }
}
