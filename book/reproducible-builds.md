# Reproducible ZisK guest build

The contract accepts proofs only for the verification key derived from the deployed guest ELF. Build and review the ELF before changing `ZISK_VECTOR_PROGRAM_VKEY`.

The committed ELF includes the SHA-256 and BLAKE2b precompiles and the rotation digest-boundary check. It was built on Girona with the pinned `v1.3.2-alpha` builder and `zisk-4.0.0` Rust toolchain. The included verification key and proof fixtures were generated from this ELF and passed the ZisK and Solidity verifiers.

Install [ZisK](https://github.com/0xPolygonHermez/zisk/tree/v1.3.2-alpha) `v1.3.2-alpha` and its `zisk-4.0.0` Rust toolchain, matching the `zisk-sdk` and `ziskos` dependencies. Then run:

```sh
cd program
cargo-zisk build --release
sha256sum target/elf/riscv64ima-zisk-zkvm-elf/release/sp1-vectorx-program
cd ..
# Compare the rebuilt ELF against the committed copy:
cmp program/target/elf/riscv64ima-zisk-zkvm-elf/release/sp1-vectorx-program elf/vector-elf
# If deliberately updating the guest:
cp program/target/elf/riscv64ima-zisk-zkvm-elf/release/sp1-vectorx-program elf/vector-elf
cargo-zisk setup --elf elf/vector-elf --proving-key "$HOME/.zisk/provingKey"
cargo run --release --bin vkey
```

The committed ELF SHA-256 is `095e31138d9871c608db33e301b4331b8d511cec0befc1185d14c123bf268fb9`. Its program verification key is `0xf27b07410e8543b675c8091dfd4819b71e9f05328af9d2bfee4a30eda372a4e7`. The verifier's root commitment is `0x6b2490fba336fb00f0c8719a961de309a8db6f43a9bfb7575ca5f014d609b265`. The `setup` step creates the ROM verification-key artifact; `vkey` reads it. Build and proving toolchains, the ZisK verifier contract, and its root commitment must be from a mutually compatible ZisK release. Set the contract's program key to the new value before submitting these proofs.
