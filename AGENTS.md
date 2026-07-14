<!-- Satellite context file — extends the global hub (~/.claude/CLAUDE.md | ~/.pi/agent/AGENTS.md). Host-neutral; project-specific only. Do not duplicate hub standards here. -->

# SIP Arcium Program

> Arcium MPC program for confidential DeFi on Solana.

**Ecosystem hub:** See [sip-protocol/sip-protocol/AGENTS.md](https://github.com/sip-protocol/sip-protocol/blob/main/AGENTS.md) for full ecosystem context.

## Quick Reference

**Tech Stack:** Rust, Anchor 1.0.2, Arcium SDK 0.10.4 (Arcis circuits)
**Deployment:** Solana Devnet

```bash
arcium build                                   # Build program + circuits
arcium test                                    # Run tests
anchor deploy --provider.cluster devnet        # Deploy to devnet
npx ts-node scripts/init-comp-defs.ts          # Initialize computation definitions
```

## Deployment Info

| Field | Value |
|-------|-------|
| Program ID | `S1P5q5497A6oRCUutUFb12LkNQynTNoEyRyUvotmcX9` |
| MXE Account | `5qy4Njk4jCJE4QgZ5dsg8uye3vzFypFTV7o7RRSQ8vr4` |
| Cluster Offset | 456 (Arcium devnet v0.6.3) |
| Network | Devnet |

## MPC Circuits (encrypted-ixs/)

| Circuit | Purpose | Inputs | Outputs |
|---------|---------|--------|---------|
| `private_transfer` | Validate encrypted balance transfer | sender_balance, amount, min_balance | is_valid, new_balance |
| `check_balance` | Threshold check without revealing | balance, minimum | meets_minimum |
| `validate_swap` | Confidential DEX swap validation | input_balance, input_amount, min_output, actual_output | is_valid, new_balance, slippage_ok |

## Key Files

| Path | Description |
|------|-------------|
| `programs/sip_arcium_transfer/src/lib.rs` | Anchor program (queue computations, callbacks) |
| `encrypted-ixs/src/lib.rs` | Arcis MPC circuits |
| `scripts/init-comp-defs.ts` | Initialize computation definitions on-chain |

## Architecture

```
CLIENT (sip-mobile / sip-app) → encrypt inputs with x25519 keypair
ANCHOR PROGRAM (this repo) → queue computation to Arcium MXE → await callback w/ encrypted result → emit events
ARCIUM MXE CLUSTER → decrypt inputs (threshold MPC) → execute circuit (no single node sees plaintext) → encrypt outputs with requester's key
```

## Integration with SIP Mobile

Used by sip-mobile via `src/privacy-providers/arcium.ts` (MPC ops adapter) and `src/hooks/usePrivateDeFi.ts` (orchestrates C-SPL + Arcium + Stealth).

## Repo-Specific Guidelines

**DO:** run `arcium build` after circuit changes; test with `arcium test` before deploying; use S1P vanity addresses.
**DON'T:** commit `target/` or `node_modules/`; deploy to mainnet without audit; hardcode cluster offsets (use Arcium.toml).