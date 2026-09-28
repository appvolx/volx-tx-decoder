# volx-tx-decoder

**VOLX Ecosystem Tool** — Local Solana transaction decoder (Rust).

Part of the [VOLX](https://volx.app) tooling suite.  
GitHub org: [https://github.com/appvolx](https://github.com/appvolx)

## What it does

`volx-tx-decoder` fetches a transaction by signature from a public Solana RPC and prints a structured summary:

- Slot, block time, fee, status
- Account keys
- Instruction program IDs
- Log messages (when available)
- Whether the Pump.fun or common system programs appear

Pure read-only tool. No signing, no private keys, no credentials.

## Why it exists

When debugging Relay, Pump, or Raydium flows in VOLX, developers need a fast terminal view of a transaction without opening a browser. This utility provides that.

## Features

- Rust 2021 edition
- Minimal dependencies (`serde`, `serde_json`, `reqwest` blocking, `clap`)
- JSON or human output
- Official program ID annotations for Pump and System Program

## Requirements

- Rust toolchain (rustup) ≥ 1.75
- Network access to a Solana RPC

## Installation

```bash
cd volx-tx-decoder
cargo build --release
# binary: target/release/volx-tx-decoder
```

## Quick start

```bash
cargo run -- --signature <TX_SIGNATURE>

# Custom RPC
cargo run -- --signature <TX_SIGNATURE> --rpc https://api.mainnet-beta.solana.com

# JSON
cargo run -- --signature <TX_SIGNATURE> --json
```

## Security model

- Outbound HTTPS to RPC only
- Never stores or accepts private keys
- Safe for public GitHub repositories

## Integration with VOLX

Helpful when validating transactions produced by the VOLX launch, swap, or LP flows against on-chain reality.

## License

MIT — complementary tooling for the VOLX project (https://volx.app).
