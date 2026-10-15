# ZisK Vector proof fixtures

The included PLONK proofs use the committed ELF built with ZisK `v1.3.2-alpha` (SHA-256 `095e31138d9871c608db33e301b4331b8d511cec0befc1185d14c123bf268fb9`). Their program verification key is `0xf27b07410e8543b675c8091dfd4819b71e9f05328af9d2bfee4a30eda372a4e7`.

`header-range-turing-4007140-4007141.bin` contains a public Turing finality proof and headers for blocks 4,007,140–4,007,141. It was produced with `cargo run --bin test -- --recent-finality --save-input <path>` against `https://avail-turing-rpc.publicnode.com` on 2026-10-13. The input has a Merkle tree size of 512.

On Linux with ZisK `v1.3.2-alpha` and the matching ELF:

```sh
cargo run --bin test -- --input-file script/test_assets/header-range-turing-4007140-4007141.bin
```

The test computes the Vector output natively, executes the guest with Assembly, and compares the committed digest. Pass `--emulator` to use the Rust emulator when Assembly resources are unavailable.

`rotate-synthetic.bin` is a deterministic, signed GRANDPA authority rotation generated with `cargo run --bin test -- --synthetic-rotate --save-input script/test_assets/rotate-synthetic.bin --save-output script/test_assets/rotate-synthetic-output.bin`. It exercises the rotate path without an archive RPC. Its keys and header are test data, not an Avail chain transition.

Both fixtures have a 576-byte `-output.bin` claim, a `-plonk.proof` from the pinned ZisK GPU prover on Girona, and a `-solidity.json` export containing its contract calldata. Both proofs passed pinned ZisK CPU verification. The committed [Foundry test](../../contracts/test/ZiskRealProof.t.sol) replays their calldata through the pinned official `v1.3.2-alpha` Solidity verifier and `SP1Vector`, then checks the resulting state. Run it with `cd contracts && forge test --match-path test/ZiskRealProof.t.sol`.
