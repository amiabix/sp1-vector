# ZisK Vector

Vector brings Avail block-header commitments to an EVM chain. Starting from a trusted Avail header and validator set, it proves that subsequent headers connect and that GRANDPA validators finalized the target header. The contract then records the proved state and data-root commitments. The proof covers headers and finality, not block bodies or transactions.

## What we completed

- Ported the existing SP1 header-range and authority-rotation checks to a ZisK guest. This guest checks GRANDPA Ed25519 signatures with ZisK's curve precompile.
- Updated the operator to build ZisK inputs, execute the guest with Assembly, generate GPU proofs, and relay them with their public output. The full PLONK relay path was tested with the earlier 1.3 guest.
- Connected the ZisK Solidity verifier to the Vector contract and bound each proof to its output before state updates. The proxy contract remains named `SP1Vector` to preserve its upgrade and storage layout.
- Built the Ed25519 guest with pinned ZisK `1.4.0-alpha` source and generated GPU STARK proofs for both saved inputs on Girona.

The committed PLONK fixtures and Solidity verification results belong to the earlier `v1.3.2-alpha` guest; they do not verify this branch's ELF.

The intended path is **Avail headers and validator votes → operator → ZisK guest → GPU proof → EVM verifier → Vector contract**. See [components](book/components.md), [reproducible builds](book/reproducible-builds.md), and [deployment](book/deployment.md) for details.

## Measured results

Measurements used the pinned ZisK `1.4.0-alpha` guest, Assembly execution, and GPU STARK proving on Girona's RTX 5090. The real header input covers Avail Turing blocks **4,007,140 to 4,007,141**: one new block, with **14 signed votes from 20 validators**. The rotation input is a synthetic one-validator fixture.

| Measurement | Real header update | Synthetic authority rotation |
| --- | ---: | ---: |
| Input size | 3,504 bytes | 424 bytes |
| ZisK guest steps (virtual instructions) | **548,231** | **23,339** |
| Assembly guest execution | 28 ms | 5 ms |
| GPU STARK proof generation | **6.880 s** | **6.728 s** |
| STARK proof size | 415,232 bytes | 415,232 bytes |

These are single runs. The header input contains two 431-byte headers; the configured Merkle tree has 512 slots, but only one new-block entry is populated. The prover allocated about 29.1 GB of GPU memory. The rotation result is not a production validator-set change.

The earlier 1.3 guest used 10,832,220 header steps and 767,823 rotation steps. These 1.4 results are about 95% and 97% fewer steps, respectively; the change also includes a ZisK version upgrade.

Both STARK proofs verified with a local diagnostic correction to the pinned 1.4 verifier's transcript challenge count. The unmodified verifier rejects them; matching PLONK proofs and Solidity verification are still outstanding. See [reproducible builds](book/reproducible-builds.md). The [committed PLONK fixtures](script/test_assets/README.md) remain tied to the earlier 1.3 guest. Its proof times include PLONK, so they are not directly comparable with the STARK times above. No same-workload SP1 comparison or full 512-block batch has been measured.

This migration branch starts from [Avail's `v2.0.1` source](https://github.com/availproject/sp1-vector/tree/c649310e79de376926863504d49eddbf09532980). Each migration step is a separate commit on top of that version.
