use sp1_vectorx_script::{zisk_vector_program, zisk_vector_vkey};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("VK: {:#x}", zisk_vector_vkey(&zisk_vector_program())?);

    Ok(())
}
