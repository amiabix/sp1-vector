# Deployment

1. Build and compare the guest ELF as described in [Reproducible Builds](reproducible-builds.md). Run `cargo run --release --bin vkey` and record its `VK` output.
2. Deploy the [ZisK PLONK verifier](https://github.com/0xPolygonHermez/zisk/tree/v1.3.2-alpha/zisk-contracts) for the proving-key release used by the operator. Confirm that it exposes `verifySnarkProof` and `getRootCVadcopFinal`, and record its address. A verifier from a different proving-key release can reject valid proofs.
3. Generate the Avail genesis values with `AVAIL_URL=<Avail RPC> cargo run --release --bin genesis`. Put those values, `ZISK_VECTOR_PROGRAM_VKEY=<VK>`, and `ZISK_VERIFIER_ADDRESS=<verifier address>` in `contracts/.env` alongside the target chain settings from `.env.example`.
4. From `contracts/`, run `forge script script/Deploy.s.sol --private-key "$PRIVATE_KEY" --multi --broadcast --verify` with `CHAINS` set. For an existing proxy, use `Upgrade.s.sol`; it freezes the proxy, upgrades it, sets the verifier and key, and unfreezes it in one broadcast script. Review the existing guardian and relayer settings first.
5. Run the operator on a Linux host with the matching ZisK proving keys and GPU toolchain, or set `ZISK_COORDINATOR_URL` for a compatible remote coordinator. Set the Avail RPC, `AVAIL_CHAIN_ID`, `VECTORX_QUERY_URL`, target chain RPC and contract address, and relayer signer values from `script/.env.example`; then run `cargo run --release --bin operator` on the host or start the operator image.

The operator verifies PLONK proofs locally before relaying. A host installation needs `snarkjs@0.7.5` on `PATH` (`npm install --global snarkjs@0.7.5`). The release operator image includes Node and `snarkjs`; set `ZISK_COORDINATOR_URL` when using that image unless the container has the GPU toolchain and proving keys.

The operator sends PLONK proof bytes, ZisK public values, and the ABI-encoded Vector output. The contract verifies that the output's digest is bound to the proof before accepting the state transition.
