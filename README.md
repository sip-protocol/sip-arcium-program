<div align="center">

<pre>
███████╗ ██╗ ██████╗      █████╗ ██████╗  ██████╗██╗██╗   ██╗███╗   ███╗
██╔════╝ ██║ ██╔══██╗    ██╔══██╗██╔══██╗██╔════╝██║██║   ██║████╗ ████║
███████╗ ██║ ██████╔╝    ███████║██████╔╝██║     ██║██║   ██║██╔████╔██║
╚════██║ ██║ ██╔═══╝     ██╔══██║██╔══██╗██║     ██║██║   ██║██║╚██╔╝██║
███████║ ██║ ██║         ██║  ██║██║  ██║╚██████╗██║╚██████╔╝██║ ╚═╝ ██║
╚══════╝ ╚═╝ ╚═╝         ╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝╚═╝ ╚═════╝ ╚═╝     ╚═╝
</pre>

# SIP Arcium Program

> **Privacy through MPC — no single node sees your data**

**Confidential DeFi on Solana using Arcium's Multi-Party Computation network**

*Encrypted transfers • Confidential swaps • Balance privacy • Jupiter integration*

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75-orange?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Anchor](https://img.shields.io/badge/Anchor-1.0-blueviolet)](https://www.anchor-lang.com/)
[![Arcium](https://img.shields.io/badge/Arcium-0.10.4-00D4AA)](https://arcium.com/)
[![Solana](https://img.shields.io/badge/Solana-Devnet-9945FF?logo=solana&logoColor=white)](https://solana.com/)

**Solana Privacy Hackathon 2026** | [Arcium Track Submission](https://sip-protocol.org/showcase/solana-privacy-2026)

</div>

---

## Table of Contents

- [What is SIP Arcium?](#-what-is-sip-arcium)
- [The Problem](#-the-problem)
- [The Solution](#-the-solution)
- [MPC Circuits](#-mpc-circuits)
- [Architecture](#%EF%B8%8F-architecture)
- [Deployment](#-deployment)
- [Quick Start](#-quick-start)
- [Integration](#-integration)
- [Tech Stack](#%EF%B8%8F-tech-stack)
- [Development](#-development)
- [Security](#-security)
- [Related Projects](#-related-projects)
- [License](#-license)

---

## 🛡️ What is SIP Arcium?

SIP Arcium brings **confidential DeFi** to Solana using Arcium's Multi-Party Computation (MPC) network. Your balance and transfer amounts stay encrypted throughout the entire computation — no single node ever sees your actual values.

```
Traditional DeFi  → Public balances, public amounts, MEV attacks
SIP + Arcium      → Encrypted computation, hidden amounts, MEV protection
```

**Your financial data stays private. Even during computation.**

---

## 🎯 The Problem

Current Solana DeFi exposes **everything** on-chain:

| Data Point | Visibility | Risk |
|------------|------------|------|
| **Token Balances** | Public | Wealth profiling, targeted attacks |
| **Transfer Amounts** | Public | Front-running, sandwich attacks |
| **Swap Details** | Public | MEV extraction, copy trading |
| **Transaction Graph** | Permanent | Surveillance, address clustering |

### Why ZK Alone Isn't Enough

Zero-knowledge proofs verify correctness, but the **computation itself** still happens somewhere. With ZK:
- Prover sees all values during proof generation
- Single point of trust (the prover)
- Complex circuits for arbitrary logic

**MPC solves this** — computation is distributed across multiple nodes, none of which see the plaintext.

---

## 💡 The Solution

SIP Arcium uses **threshold MPC** to perform DeFi operations on encrypted data:

### How It Works

```
┌─────────────────────────────────────────────────────────────────┐
│  1. CLIENT (SIP Mobile / Web App)                               │
│     └── Encrypt inputs with x25519 keypair                      │
│         (balance, amount, slippage — all encrypted)             │
├─────────────────────────────────────────────────────────────────┤
│  2. ANCHOR PROGRAM (this repo)                                  │
│     └── Queue encrypted computation to Arcium MXE               │
│         (on-chain coordination, no plaintext access)            │
├─────────────────────────────────────────────────────────────────┤
│  3. ARCIUM MXE CLUSTER (off-chain MPC network)                  │
│     ├── Threshold decrypt (requires 2/3+ nodes)                 │
│     ├── Execute circuit (distributed computation)               │
│     │   └── No single node sees complete plaintext              │
│     ├── Encrypt result with requester's key                     │
│     └── Return via callback                                     │
├─────────────────────────────────────────────────────────────────┤
│  4. CALLBACK (back on-chain)                                    │
│     └── Emit encrypted result                                   │
│         (only requester can decrypt)                            │
└─────────────────────────────────────────────────────────────────┘
```

### MPC Guarantees

| Property | Description |
|----------|-------------|
| **Confidentiality** | No single node sees plaintext values |
| **Correctness** | Threshold signatures verify computation |
| **Availability** | Tolerates minority node failures |
| **Non-collusion** | Requires 2/3+ nodes to collude to break privacy |

---

## 🔐 MPC Circuits

Three Arcis circuits for confidential DeFi operations:

### 1. Private Transfer (`private_transfer`)

Validate encrypted balance transfers without revealing amounts.

```rust
// Inputs (all encrypted)
struct PrivateTransferInput {
    sender_balance: u64,  // Current balance (encrypted)
    amount: u64,          // Transfer amount (encrypted)
    min_balance: u64,     // Minimum to maintain (encrypted)
}

// Outputs (encrypted with requester's key)
struct PrivateTransferOutput {
    is_valid: bool,           // Sufficient balance?
    new_sender_balance: u64,  // Balance after transfer
}
```

**Use case:** Shielded SOL/token transfers in SIP Mobile

### 2. Balance Check (`check_balance`)

Threshold check without revealing actual balance.

```rust
// Prove balance >= minimum without revealing balance
check_balance(balance: u64, minimum: u64) -> bool
```

**Use case:** Pre-validation, rent exemption checks, minimum balance requirements

### 3. Confidential Swap (`validate_swap`)

Validate DEX swaps with encrypted slippage protection.

```rust
// Inputs (all encrypted)
struct ConfidentialSwapInput {
    input_balance: u64,   // Token balance
    input_amount: u64,    // Swap input
    min_output: u64,      // Slippage protection
    actual_output: u64,   // DEX quote result
}

// Outputs
struct ConfidentialSwapOutput {
    is_valid: bool,          // Sufficient balance?
    new_input_balance: u64,  // Balance after swap
    slippage_ok: bool,       // Within tolerance?
}
```

**Use case:** Jupiter swaps with hidden amounts in SIP Mobile

---

## 🏗️ Architecture

### Project Structure

```
sip-arcium-program/
├── programs/
│   └── sip_arcium_transfer/
│       └── src/
│           └── lib.rs          # Anchor program
│               ├── init_*_comp_def()     # Initialize computation definitions
│               ├── private_transfer()     # Queue transfer computation
│               ├── check_balance()        # Queue balance check
│               ├── validate_swap()        # Queue swap validation
│               └── *_callback()           # Handle MXE responses
├── encrypted-ixs/
│   └── src/
│       └── lib.rs              # Arcis MPC circuits
│           ├── private_transfer()   # Transfer validation circuit
│           ├── check_balance()      # Balance threshold circuit
│           └── validate_swap()      # Swap validation circuit
├── scripts/
│   └── init-comp-defs.ts       # Deploy computation definitions
├── Anchor.toml                 # Anchor config
├── Arcium.toml                 # Arcium MXE config
└── Cargo.toml                  # Rust dependencies
```

### Data Flow

```
User Input → Encrypt (x25519) → Anchor Program → MXE Queue
                                                      │
                                          ┌───────────┴───────────┐
                                          ▼                       ▼
                                    MXE Node 1              MXE Node 2
                                          │                       │
                                          └───────────┬───────────┘
                                                      ▼
                                              Threshold MPC
                                              (compute on shares)
                                                      │
                                                      ▼
                                              Encrypt Result
                                                      │
                                                      ▼
                                              Callback → Client Decrypt
```

---

## 📍 Deployment

### Devnet (Current)

| Field | Value |
|-------|-------|
| **Program ID** | `S1P5q5497A6oRCUutUFb12LkNQynTNoEyRyUvotmcX9` |
| **MXE Account** | `5qy4Njk4jCJE4QgZ5dsg8uye3vzFypFTV7o7RRSQ8vr4` |
| **Cluster Offset** | 456 |
| **Arcium Version** | v0.6.3 (devnet) |

### Explorer Links

- [Program on Solscan](https://solscan.io/account/S1P5q5497A6oRCUutUFb12LkNQynTNoEyRyUvotmcX9?cluster=devnet)
- [MXE Account](https://solscan.io/account/5qy4Njk4jCJE4QgZ5dsg8uye3vzFypFTV7o7RRSQ8vr4?cluster=devnet)

---

## 🚀 Quick Start

### Prerequisites

- Rust 1.89+
- Solana CLI 2.1+
- Anchor 1.0+
- Node.js 20+

### Setup

```bash
# Clone the repository
git clone https://github.com/sip-protocol/sip-arcium-program.git
cd sip-arcium-program

# Install dependencies
yarn install

# Build program + circuits
arcium build

# Run tests
arcium test

# Deploy to devnet
anchor deploy --provider.cluster devnet

# Initialize computation definitions
npx ts-node scripts/init-comp-defs.ts
```

### Local Development with Arcium

```bash
# Start local validator with Arcium
arcium localnet

# Deploy locally
anchor deploy --provider.cluster localnet

# Test with local MXE
arcium test --provider.cluster localnet
```

---

## 🔌 Integration

### From SIP Mobile (TypeScript)

```typescript
import { ArciumAdapter } from '@/privacy-providers/arcium'
import { Connection, Keypair } from '@solana/web3.js'

// Initialize adapter
const adapter = new ArciumAdapter({
  programId: 'S1P5q5497A6oRCUutUFb12LkNQynTNoEyRyUvotmcX9',
  mxeAccount: '5qy4Njk4jCJE4QgZ5dsg8uye3vzFypFTV7o7RRSQ8vr4',
  connection: new Connection('https://api.devnet.solana.com'),
})

// Validate a private transfer
const result = await adapter.validateTransfer({
  senderBalance: 1000000000n,  // 1 SOL in lamports
  amount: 100000000n,          // 0.1 SOL
  minBalance: 890880n,         // Rent exemption
  encryptionKeypair,           // x25519 keypair
})

if (result.isValid) {
  // Proceed with transfer
  console.log('New balance:', result.newSenderBalance)
}
```

### From Rust (Direct)

```rust
use anchor_lang::prelude::*;
use sip_arcium_transfer::cpi::accounts::PrivateTransfer;
use sip_arcium_transfer::cpi::private_transfer;

// Queue computation
private_transfer(
    ctx.accounts.into(),
    computation_offset,
    encrypted_balance,
    encrypted_amount,
    encrypted_min_balance,
    x25519_pubkey,
    nonce,
)?;
```

---

## 🛠️ Tech Stack

| Category | Technology | Purpose |
|----------|------------|---------|
| **Language** | Rust 1.89 | Program + circuits |
| **Framework** | Anchor 1.0 | Solana program framework |
| **MPC** | Arcium SDK 0.10.4 | Confidential computation |
| **Circuits** | Arcis | MPC circuit DSL |
| **Encryption** | x25519 | Input/output encryption |
| **Testing** | Mocha + Chai | Integration tests |
| **Deployment** | Solana CLI | On-chain deployment |

---

## 💻 Development

### Commands

```bash
# Build everything
arcium build

# Run tests
arcium test

# Deploy to devnet
anchor deploy --provider.cluster devnet

# Initialize computation definitions
npx ts-node scripts/init-comp-defs.ts

# Check program logs
solana logs S1P5q5497A6oRCUutUFb12LkNQynTNoEyRyUvotmcX9
```

### Adding a New Circuit

1. Define the circuit in `encrypted-ixs/src/lib.rs`:

```rust
pub struct MyInput { /* encrypted fields */ }
pub struct MyOutput { /* result fields */ }

#[instruction]
pub fn my_circuit(input: Enc<Shared, MyInput>) -> Enc<Shared, MyOutput> {
    // MPC computation logic
}
```

2. Add Anchor instructions in `programs/sip_arcium_transfer/src/lib.rs`:

```rust
pub fn my_circuit(ctx: Context<MyCircuit>, /* params */) -> Result<()> {
    queue_computation(/* ... */)?;
    Ok(())
}

#[arcium_callback(encrypted_ix = "my_circuit")]
pub fn my_circuit_callback(ctx: Context<MyCircuitCallback>, output: /* ... */) -> Result<()> {
    // Handle result
    Ok(())
}
```

3. Rebuild and test:

```bash
arcium build
arcium test
```

---

## 🔐 Security

### Trust Model

| Component | Trust Assumption |
|-----------|------------------|
| **Anchor Program** | Open source, auditable |
| **Arcium MXE** | Honest majority (2/3+ nodes) |
| **x25519 Encryption** | Standard cryptographic security |
| **Client** | User controls their keys |

### Security Properties

- ✅ **Balance privacy** — No one sees actual amounts
- ✅ **MEV protection** — Encrypted until execution
- ✅ **Non-custodial** — User holds encryption keys
- ✅ **Verifiable** — Threshold signatures prove correctness

### Limitations

- ⚠️ **Devnet only** — Not audited for mainnet
- ⚠️ **MXE trust** — Requires honest majority assumption
- ⚠️ **Latency** — MPC adds ~2-5 seconds vs direct transaction

### Reporting Security Issues

If you discover a vulnerability:
- Email: **security@sip-protocol.org**
- Do NOT open public issues for security vulnerabilities

---

## 🔗 Related Projects

| Project | Description | Link |
|---------|-------------|------|
| **sip-protocol** | Core SDK (6,600+ tests) | [GitHub](https://github.com/sip-protocol/sip-protocol) |
| **sip-mobile** | Mobile wallet (uses Arcium adapter) | [GitHub](https://github.com/sip-protocol/sip-mobile) |
| **sip-app** | Web application | [GitHub](https://github.com/sip-protocol/sip-app) |
| **Arcium Docs** | MPC framework documentation | [arcium.com/docs](https://docs.arcium.com) |

---

## 📄 License

[MIT License](LICENSE) — see LICENSE file for details.

---

<div align="center">

**Solana Privacy Hackathon 2026 — Arcium Track**

*Privacy through MPC — no single node sees your data*

[Showcase](https://sip-protocol.org/showcase/solana-privacy-2026) · [Documentation](https://docs.sip-protocol.org) · [Report Bug](https://github.com/sip-protocol/sip-arcium-program/issues)

*Part of the [SIP Protocol](https://github.com/sip-protocol) ecosystem*

</div>
