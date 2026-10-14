//! To build the binary:
//!
//!     `cargo build --release --bin genesis`
//!
use clap::Parser;
use services::input::RpcDataFetcher;
use sp1_vectorx_script::{zisk_vector_program, zisk_vector_vkey};

#[derive(Parser, Debug, Clone)]
#[command(about = "Get the genesis parameters from a block.")]
pub struct GenesisArgs {
    #[arg(long)]
    pub block: Option<u32>,
}

const HEADER_RANGE_COMMITMENT_TREE_SIZE: u32 = 1024;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let fetcher = RpcDataFetcher::new().await;
    let vk = zisk_vector_vkey(&zisk_vector_program())?;

    let args = GenesisArgs::parse();

    let header;
    if let Some(block) = args.block {
        header = fetcher.get_header(block).await;
    } else {
        header = fetcher.get_head().await;
    }
    let header_hash = header.hash();
    let authority_set_id = fetcher.get_authority_set_id(header.number).await;
    let authority_set_hash = fetcher
        .compute_authority_set_hash_for_block(header.number)
        .await;

    struct GenesisOutput {
        genesis_height: u32,
        genesis_header: String,
        genesis_authority_set_id: u64,
        genesis_authority_set_hash: String,
        zisk_vector_program_vkey: String,
        header_range_commitment_tree_size: u32,
    }

    let output = GenesisOutput {
        genesis_height: header.number,
        genesis_header: format!("{:#x}", header_hash),
        genesis_authority_set_id: authority_set_id,
        genesis_authority_set_hash: format!("{:#x}", authority_set_hash),
        zisk_vector_program_vkey: format!("{vk:#x}"),
        header_range_commitment_tree_size: HEADER_RANGE_COMMITMENT_TREE_SIZE,
    };

    println!("GENESIS_HEIGHT={}\nGENESIS_HEADER={}\nGENESIS_AUTHORITY_SET_ID={}\nGENESIS_AUTHORITY_SET_HASH={}\nZISK_VECTOR_PROGRAM_VKEY={}\nHEADER_RANGE_COMMITMENT_TREE_SIZE={}",
             output.genesis_height,
             output.genesis_header,
             output.genesis_authority_set_id,
             output.genesis_authority_set_hash,
             output.zisk_vector_program_vkey,
             output.header_range_commitment_tree_size,
    );

    Ok(())
}
