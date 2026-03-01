# CLAUDE.md - SIP Arcium Program

> **Ecosystem Hub:** See [sip-protocol/CLAUDE.md](https://github.com/sip-protocol/sip-protocol/blob/main/CLAUDE.md) for full ecosystem context

**Repository:** https://github.com/sip-protocol/sip-arcium-program
**Purpose:** Arcium MPC program for confidential DeFi on Solana

---

## Quick Reference

**Tech Stack:** Rust, Anchor, Arcium SDK (Arcis circuits)
**Deployment:** Solana Devnet

```bash
# Build
anchor build

# Test
anchor test

# Deploy
anchor deploy --provider.cluster devnet

# Initialize computation definitions
npx ts-node scripts/init-comp-defs.ts
```

---

## Deployment Info

| Field | Value |
|-------|-------|
| Program ID | `S1P5q5497A6oRCUutUFb12LkNQynTNoEyRyUvotmcX9` |
| MXE Account | `5qy4Njk4jCJE4QgZ5dsg8uye3vzFypFTV7o7RRSQ8vr4` |
| Cluster Offset | 456 (Arcium devnet v0.6.3) |
| Network | Devnet |

---

## MPC Circuits (encrypted-ixs/)

| Circuit | Purpose | Inputs | Outputs |
|---------|---------|--------|---------|
| `private_transfer` | Validate encrypted balance transfer | sender_balance, amount, min_balance | is_valid, new_balance |
| `check_balance` | Threshold check without revealing | balance, minimum | meets_minimum |
| `validate_swap` | Confidential DEX swap validation | input_balance, input_amount, min_output, actual_output | is_valid, new_balance, slippage_ok |

---

## Key Files

| Path | Description |
|------|-------------|
| `programs/sip_arcium_transfer/src/lib.rs` | Anchor program (queue computations, callbacks) |
| `encrypted-ixs/src/lib.rs` | Arcis MPC circuits |
| `scripts/init-comp-defs.ts` | Initialize computation definitions on-chain |
| `tests/sip_arcium_transfer.ts` | Integration tests |

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  CLIENT (sip-mobile / sip-app)                              │
│  └── Encrypt inputs with x25519 keypair                     │
├─────────────────────────────────────────────────────────────┤
│  ANCHOR PROGRAM (this repo)                                 │
│  ├── Queue computation to Arcium MXE                        │
│  ├── Await callback with encrypted result                   │
│  └── Emit events with encrypted outputs                     │
├─────────────────────────────────────────────────────────────┤
│  ARCIUM MXE CLUSTER                                         │
│  ├── Decrypt inputs (threshold MPC)                         │
│  ├── Execute circuit (no single node sees plaintext)        │
│  └── Encrypt outputs with requester's key                   │
└─────────────────────────────────────────────────────────────┘
```

---

## Repo-Specific Guidelines

**DO:**
- Run `anchor build` after circuit changes
- Test with `anchor test` before deploying
- Use S1P vanity addresses for consistency

**DON'T:**
- Commit target/ or node_modules/
- Deploy to mainnet without audit
- Hardcode cluster offsets (use Arcium.toml)

---

## Integration with SIP Mobile

This program is used by sip-mobile via:
- `src/privacy-providers/arcium.ts` - Adapter for MPC operations
- `src/hooks/usePrivateDeFi.ts` - Orchestrates C-SPL + Arcium + Stealth

---

**Last Updated:** 2026-01-29
