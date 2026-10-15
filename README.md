# ZisK Vector

Vector brings Avail block-header commitments to an EVM chain. Starting from a trusted Avail header and validator set, it proves that subsequent headers connect and that GRANDPA validators finalized the target header. The contract then records the proved state and data-root commitments. The proof covers headers and finality, not block bodies or transactions.

## What we completed

- Ported the existing SP1 header-range and authority-rotation checks to a ZisK guest. GRANDPA votes are checked as Ed25519 signatures inside the guest.
- Updated the operator to build ZisK inputs, execute the guest with Assembly, generate PLONK proofs with GPU acceleration, and relay the proofs and public output.
- Connected the official ZisK Solidity verifier to the Vector contract and bound each proof to its output before state updates. The proxy contract remains named `SP1Vector` to preserve its upgrade and storage layout.
- Rebuilt the revised guest with pinned ZisK `v1.3.2-alpha` and proved both paths on Girona: a real Avail header update and a synthetic authority rotation. Both proofs passed ZisK verification and the official Solidity verifier through `SP1Vector`.

This branch builds a new guest with pinned ZisK `1.4.0-alpha` source. The proof fixtures and measurements below belong to the earlier `v1.3.2-alpha` guest; they do not verify the new ELF.

The path is **Avail headers and validator votes → operator → ZisK guest → GPU proof → EVM verifier → Vector contract**. See [components](book/components.md), [reproducible builds](book/reproducible-builds.md), and [deployment](book/deployment.md) for details.

## Measured results

Measurements used ZisK `v1.3.2-alpha` on Girona's RTX 5090. The header test used real Avail Turing data from block **4,007,140 to 4,007,141**, proving **one new block**. Its 3,504-byte input contains two 431-byte headers and a finality justification with **14 signed votes from 20 validators**. The configured Merkle tree has 512 slots, but only one new-block entry is populated; this is not a 512-block benchmark.

| Measurement | Real header update | Synthetic authority rotation |
| --- | ---: | ---: |
| ZisK guest steps (virtual instructions) | 10,832,220 | 767,823 |
| Assembly guest execution | 19 ms | 8 ms |
| GPU proof generation (STARK and PLONK) | **8.405 s** | **8.301 s** |
| Estimated EVM relay transaction | **about 472,000 gas** | **about 399,000 gas** |
| Contract calldata | 2,052 bytes | 2,052 bytes |

The rotation input is a signed test fixture with one validator, not a real Avail validator-set change. Its result demonstrates the proof and contract path, not production rotation performance. Each contract call carries a 768-byte proof within the 2,052 bytes of calldata. Gas estimates add transaction base and calldata to the measured Foundry contract calls (431,221 and 358,724 gas); they exclude deployment and chain-specific data fees. The prover allocated about 28.2 GB of GPU memory. These are ZisK measurements for the revised guest; no same-workload SP1 comparison or full 512-block batch has been measured.

The committed [fixtures and reproduction steps](script/test_assets/README.md) include the real header input, both PLONK proofs, public outputs, and Solidity calldata.

This migration branch starts from [Avail's `v2.0.1` source](https://github.com/availproject/sp1-vector/tree/c649310e79de376926863504d49eddbf09532980). Each migration step is a separate commit on top of that version.
