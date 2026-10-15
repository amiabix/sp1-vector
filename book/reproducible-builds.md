# Reproducible ZisK guest build

The contract accepts proofs only for the verification key derived from the deployed guest ELF. Build and review the ELF before changing `ZISK_VECTOR_PROGRAM_VKEY`.

The committed ELF was built on Girona from [ZisK source `fa82851`](https://github.com/0xPolygonHermez/zisk/tree/fa82851b8707a6f2560fc96fd8e4f5acc406be1b) with the `zisk-4.0.0` Rust toolchain. `Cargo.lock` pins the matching Proofman revision `a6ae24aa`. The CI ELF job rebuilds this source and compares its output byte for byte with the committed file.

On Linux, install the `zisk-4.0.0` toolchain with the `v1.3.2-alpha` installer, then build the pinned `1.4.0-alpha` guest builder:

```sh
mkdir -p "$HOME/.zisk/bin"
curl -fsSL https://github.com/0xPolygonHermez/zisk/releases/download/v1.3.2-alpha/cargo_zisk_linux_amd64.tar.gz | tar -xz -C "$HOME/.zisk"
"$HOME/.zisk/bin/cargo-zisk-cpu" toolchain install --toolchain-version zisk-4.0.0
git clone https://github.com/0xPolygonHermez/zisk.git ../zisk-source
git -C ../zisk-source checkout fa82851b8707a6f2560fc96fd8e4f5acc406be1b
cargo build --release --locked -p cargo-zisk --features cargo-zisk/cpu-only --manifest-path ../zisk-source/Cargo.toml
cd program
../../zisk-source/target/release/cargo-zisk build --release
cd ..
sha256sum program/target/elf/riscv64ima-zisk-zkvm-elf/release/sp1-vectorx-program
cmp program/target/elf/riscv64ima-zisk-zkvm-elf/release/sp1-vectorx-program elf/vector-elf
```

The committed ELF SHA-256 is `93d7b4aa32c2df3a97d513594d33e4ad286d407e8e2231417f91a50d39df9494`. The [included PLONK proofs](../script/test_assets/README.md) and their Solidity verifier remain tied to the previous `v1.3.2-alpha` ELF. A new proving-key setup, program verification key, PLONK proofs, and Solidity verification are required before this ELF can be deployed.
