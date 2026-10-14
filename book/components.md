# Components

- `SP1Vector` is the existing proxy-compatible contract name. It tracks Avail headers, authority sets, and data commitments. Its verifier and program-key storage slots are preserved across upgrades.
- A deployed [ZisK PLONK verifier](https://github.com/0xPolygonHermez/zisk/tree/v1.3.2-alpha/zisk-contracts) verifies the proof against the guest program key and verifier root.
- The ZisK guest verifies the same header-range and authority-rotation statements as before. It commits the SHA-256 digest of the ABI-encoded Vector output. The contract checks that the supplied output matches this digest before updating state.
- The operator fetches Avail data, produces proofs with ZisK Assembly and GPU proving (or a configured coordinator), and relays them to `SP1Vector`.
