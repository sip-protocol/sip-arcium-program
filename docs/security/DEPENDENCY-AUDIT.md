# Dependency Security Audit

**Scope:** npm dependencies in `package.json` / `package-lock.json`. These are **test tooling only** (mocha/chai/ts-mocha for `anchor test` TypeScript tests plus the Anchor/Arcium client SDKs the tests import). Nothing in this tree ships to users; the on-chain program is Rust (`programs/`, `encrypted-ixs/`) and has its own supply chain (Cargo).

**Last reviewed:** 2026-06-12

## Resolved via `overrides` (package.json)

Seven Dependabot alerts fixed by pinning patched versions. npm overrides have no version-range selectors, so the two coexisting minimatch majors and the rpc-websockets-only ws bump use per-parent nested overrides.

| Alert | Package | Path | Fix |
|-------|---------|------|-----|
| #3 | bn.js | `node_modules/bn.js` | 5.2.2 → 5.2.3 |
| #7 | minimatch | `node_modules/minimatch` (mocha) | 4.2.1 → 4.2.6 |
| #10 | minimatch | `node_modules/glob/node_modules/minimatch` | 3.1.2 → 3.1.5 (kept on glob@7's supported 3.x line) |
| #8, #15 | serialize-javascript | `node_modules/serialize-javascript` (mocha) | 6.0.0 → 7.0.5 (cross-major: no 6.x patch exists; mocha is the only consumer and only loads it in parallel mode; API unchanged) |
| #13 | picomatch | `node_modules/picomatch` | 2.3.1 → 2.3.2 |
| #17 | ws | `node_modules/rpc-websockets/node_modules/ws` | 8.19.0 → 8.21.0 (nested override; jayson's ws@7.5.10 is outside the vulnerable `>=8.0.0 <8.20.1` range and intentionally stays on 7.x) |

## Dismissed alerts

### Alert #16 — uuid < 11.1.1 (GHSA-w5hq-g745-h8pq, medium) — `not_used`

The advisory affects only the `v3()` / `v5()` / `v6()` API methods **when the caller passes an external output buffer** (small `buf` / large `offset` causes silent partial writes). `v1()`, `v4()`, and `v7()` already throw `RangeError` on invalid bounds, per the advisory itself.

Consumers of `uuid@8.3.2` in this tree, verified against installed sources:

- **jayson@4.3.0** (`@solana/web3.js` → jayson) — `lib/generateRequest.js:3` and `lib/utils.js:6`: `const uuid = require('uuid').v4;` invoked as `uuid()` with no arguments.
- **rpc-websockets@9.3.3** (`@solana/web3.js` → rpc-websockets) — `dist/index.cjs`: `socket._id = uuid.v1()` with no arguments.

Neither consumer calls a vulnerable API, and neither passes a buffer. Forcing uuid 8.x → 11.x would be a three-major jump through jayson's CJS `require('uuid')` chain for zero security benefit.

**Re-evaluate if:** a dependency bump changes jayson/rpc-websockets uuid usage to `v3`/`v5`/`v6` with buffers, or a direct uuid dependency is added.

## Known non-alert findings (out of scope)

`npm audit` additionally flags brace-expansion, js-yaml, diff, and nanoid (all via mocha@9, no Dependabot alerts as of the review date). diff and nanoid have no fix within mocha 9's tree — resolving them requires a mocha 9 → 11 migration, tracked as future maintenance. *(Update 2026-06-12: js-yaml subsequently received a Dependabot mapping (alert #19) and was fixed via the `js-yaml: ^4.1.1` override → 4.2.0.)*

## Cargo.lock alerts (2026-06-12 fresh scan — alerts #22–#31)

A full Dependabot re-scan surfaced 10 Rust alerts in the root `Cargo.lock`. **8 fixed via in-semver `cargo update --precise`** (rustls-webpki → 0.103.13, quinn-proto → 0.11.14, bytes → 1.11.1, keccak → 0.1.6, rand 0.8.5 → 0.8.6, rand 0.9.2 → 0.9.3); **2 dismissed** with the evidence below.

**MSRV gate:** platform-tools v1.51 bundles rustc **1.84.1**; modern cargo hard-errors when a locked crate's `rust-version` exceeds the toolchain. Every applied bump was verified MSRV ≤ 1.84.1 (worst: quinn-proto 1.74.1).

**Verification limits (honest):** neither `cargo check` nor `anchor build` passes on a host without the Arcium CLI build pipeline — *including on the unmodified baseline* (arcis proc-macros panic outside `arcium build`). The applied changes are lockfile-only, semver-compatible patch/minor bumps, MSRV-verified; the next `arcium build` from a proper toolchain is the final gate.

### Alert #23 — time >= 0.3.6 < 0.3.47 (GHSA-r6v5-fh4h-64xc, medium) — `no_bandwidth`

The only patched release (0.3.47) declares `rust-version = 1.88.0`, above platform-tools v1.51's rustc 1.84.1. `time` sits in the arcis-compiler / proc-macro graph (`x509-parser`/`rcgen` ← `arcium-primitives` ← `arcis`), which the platform-tools cargo builds — pinning 0.3.47 would make the build refuse outright. Reverted to baseline 0.3.46. **Re-evaluate when platform-tools ships rustc ≥ 1.88** (or the arcium SDK drops the x509-parser path).

### Alert #30 — rand >= 0.7.0 < 0.8.6 (GHSA-cq8v-f236-94qc, low) — residual 0.7.3 `not_used`

The 0.8.5 instance was bumped to patched 0.8.6; the alert continues to match only `rand@0.7.3`, which has no 0.7.x patch. Sole chain: `libsecp256k1 v0.6.0` (key-generation feature) ← `solana-secp256k1-recover` ← `solana-program`. The recover path performs pure signature recovery — it never invokes an RNG (and the on-chain BPF environment has no entropy source; key-generation APIs are unused). Forcing 0.7 → 0.8 across libsecp256k1's declared range is a cross-major gamble into the pinned solana-program graph for zero reachable risk.

## Verification procedure

```bash
rm -rf node_modules
npx -y npm@10 ci      # lockfile is generated with npm@10 so npm 10 and 11 can both consume it
npx tsc --noEmit      # expect only the 2 pre-existing TS2307s for target/types (generated by anchor build)
npm ls bn.js minimatch serialize-javascript picomatch uuid ws
```

Note: regenerate the lockfile with `npx -y npm@10 install` (not npm 11) — npm 11 drops optional-dep lock nodes that npm 10 `npm ci` then fails on ("Missing X from lock file").
