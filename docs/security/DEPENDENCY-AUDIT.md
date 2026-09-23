# Dependency Security Audit

**Scope:** npm dependencies in `package.json` / `package-lock.json`. These are **test tooling only** (mocha/chai/ts-mocha for `anchor test` TypeScript tests plus the Anchor/Arcium client SDKs the tests import). Nothing in this tree ships to users; the on-chain program is Rust (`programs/`, `encrypted-ixs/`) and has its own supply chain (Cargo).

**Last reviewed:** 2026-09-24 (minimatch override redesign — see the Alert #7/#10 addendum; prior review 2026-06-12)

## Resolved via `overrides` (package.json)

Seven Dependabot alerts fixed by pinning patched versions. npm overrides have no version-range selectors, so the coexisting minimatch majors and the rpc-websockets-only ws bump use per-parent nested overrides. *(2026-09-24: both minimatch pins were consolidated into one nested block under `mocha` — they exist only for mocha@9's tree and must be deleted atomically with the mocha 11 bump. See the addendum below.)*

| Alert | Package | Path | Fix |
|-------|---------|------|-----|
| #3 | bn.js | `node_modules/bn.js` | 5.2.2 → 5.2.3 |
| #7 | minimatch | `node_modules/minimatch` (mocha) | 4.2.1 → 4.2.6 (2026 ReDoS family; obsolete at mocha 11 — see addendum) |
| #10 | minimatch | `node_modules/glob/node_modules/minimatch` | 3.1.2 → 3.1.5 (kept on glob@7's supported 3.x line; pin now nested under `mocha` — see addendum) |
| #8, #15 | serialize-javascript | `node_modules/serialize-javascript` (mocha) | 6.0.0 → 7.0.5 (cross-major: no 6.x patch exists; mocha is the only consumer and only loads it in parallel mode; API unchanged) |
| #13 | picomatch | `node_modules/picomatch` | 2.3.1 → 2.3.2 |
| #17 | ws | `node_modules/rpc-websockets/node_modules/ws` | 8.19.0 → 8.21.0 (nested override; jayson's ws@7.5.10 is outside the vulnerable `>=8.0.0 <8.20.1` range and intentionally stays on 7.x) |

### Alert #7 / #10 addendum (2026-09-24) — minimatch ReDoS family and the mocha 11 bump (#9)

**Advisory family** (OSV/GHSA data verified 2026-09-24). Alert #7 flagged `minimatch@4.2.1` — mocha@9.2.2's **exact-pinned** direct dependency (`"minimatch": "4.2.1"` in mocha's own manifest; it cannot float to a patched release, which is why an override is required while mocha 9 is installed). Alert #10 flagged `minimatch@3.1.2` under mocha's glob@7.2.0. Current advisory coverage:

| Advisory | CVE | 3.x fixed | 4.x fixed | 9.x fixed |
|----------|-----|-----------|-----------|-----------|
| GHSA-3ppc-4f35-3m26 | CVE-2026-26996 | 3.1.3 | 4.2.4 | 9.0.6 |
| GHSA-7r86-cg39-jmmj | CVE-2026-27903 | 3.1.3 | 4.2.5 | 9.0.7 |
| GHSA-23c5-xmqv-rm74 | CVE-2026-27904 | 3.1.4 | 4.2.5 | 9.0.7 |
| GHSA-f8q6-p94x-37v3 | CVE-2022-3517 | 3.0.5 | — | — |

The pinned resolutions (4.2.6 under mocha, 3.1.5 under glob) satisfy every range above.

**Why both pins live in one nested block under `mocha`.** npm applies a nested override to *every* matching dependency edge in the parent's subtree, not just the direct edge. Consequences, both demonstrated:

- Deleting the top-level `glob` pin while keeping `mocha.minimatch` leaves glob's edge spec forced to `^4.2.5` with a stale 3.1.5 resolution — `npm ls` reports `minimatch@3.1.5 invalid: "^4.2.5" from node_modules/glob`, and the next honest regen would force minimatch 4.2.6 into glob@7. The 3.x pin cannot be removed while glob@7 exists; it is only meaningful under mocha, so it now nests there.
- Dependabot's own #9 lockfile regen demonstrated the post-bump hazard of the old layout: glob@10.5.0 received minimatch **3.1.5** (top-level glob pin) while mocha@11.7.6's declared `minimatch: ^9.0.5` was forced down to **4.2.6**, crashing every spec-resolving mocha invocation: `TypeError: (intermediate value).hasMagic is not a function` at `glob/dist/commonjs/has-magic.js:21` via `mocha/lib/cli/lookup-files.js:78`.

Grouping the two pins under `mocha` makes them one atomic unit that exists only for mocha@9's tree: deleting the `mocha` overrides entry removes both at once, exactly when mocha 11 lands.

**Supersession at mocha 11.7.6.** mocha declares `minimatch: ^9.0.5` directly plus `glob: ^10.4.5` (→ `minimatch ^9.0.1`); a fresh resolve lands on **9.0.9 ≥ 9.0.7 — patched across the entire family** (and far above the `<3.0.5` range of the 2022 advisory). No minimatch override is needed once mocha 11 lands, and any surviving one breaks glob@10. Removal is therefore **coupled to the mocha bump, not performed before it**: with mocha@9.2.2 still installed, removing the pins would leave its exact-pinned 4.2.1 in the tree and re-open Alert #7 on main.

**End-state verification (2026-09-24):** local regen of master + `mocha ^11.7.6` + atomic deletion of the `mocha` overrides block gives a single `minimatch@9.0.9` in the tree (`npm ls`), `npm audit` reports **zero minimatch advisories**, and the #9 reproducer (`npx mocha`, spec collection) completes normally. Side effects: the mocha@9-subtree findings (brace-expansion, js-yaml ≤4.3.1, nanoid) clear — the tree resolves js-yaml@4.3.2 and brace-expansion@2.1.7; `diff` persists (mocha 11 pulls jsdiff 7.x, still inside the flagged 6.0.0–8.0.2 range); the jayson/uuid/stream-json and @anchor-lang toml chains are mocha-independent and unchanged.

**#9 merge protocol.** Do **not** use `@dependabot rebase` — its lockfile regen inherits the overrides and re-creates the broken pairing (this is exactly how the current #9 branch got its crash). After this redesign merges, regen by hand on the #9 branch: `git merge origin/master`; resolve `package.json` to main's content plus `"mocha": "^11.7.6"` **with the entire `mocha` overrides block deleted**; regenerate the lockfile honestly from the merged manifest (`rm -rf node_modules package-lock.json && npx -y npm@10 install` — a `--lockfile-only` install silently no-ops); then gate on `npm ci && npm ls minimatch && npm audit` (expect: single minimatch@9.x, zero minimatch advisories) before pushing and merging #9.

## Dismissed alerts

### Alert #16 — uuid < 11.1.1 (GHSA-w5hq-g745-h8pq, medium) — `not_used`

The advisory affects only the `v3()` / `v5()` / `v6()` API methods **when the caller passes an external output buffer** (small `buf` / large `offset` causes silent partial writes). `v1()`, `v4()`, and `v7()` already throw `RangeError` on invalid bounds, per the advisory itself.

Consumers of `uuid@8.3.2` in this tree, verified against installed sources:

- **jayson@4.3.0** (`@solana/web3.js` → jayson) — `lib/generateRequest.js:3` and `lib/utils.js:6`: `const uuid = require('uuid').v4;` invoked as `uuid()` with no arguments.
- **rpc-websockets@9.3.3** (`@solana/web3.js` → rpc-websockets) — `dist/index.cjs`: `socket._id = uuid.v1()` with no arguments.

Neither consumer calls a vulnerable API, and neither passes a buffer. Forcing uuid 8.x → 11.x would be a three-major jump through jayson's CJS `require('uuid')` chain for zero security benefit.

**Re-evaluate if:** a dependency bump changes jayson/rpc-websockets uuid usage to `v3`/`v5`/`v6` with buffers, or a direct uuid dependency is added.

## Known non-alert findings (out of scope)

`npm audit` additionally flags brace-expansion, js-yaml, diff, and nanoid (all via mocha@9, no Dependabot alerts as of the review date). diff and nanoid have no fix within mocha 9's tree — resolving them requires a mocha 9 → 11 migration, tracked as future maintenance. *(Update 2026-06-12: js-yaml subsequently received a Dependabot mapping (alert #19) and was fixed via the `js-yaml: ^4.1.1` override → 4.2.0.)* *(2026-09-24: newer js-yaml advisories — GHSA-52cp-r559-cp3m et al., fixed 4.3.2 — again flag the tree's 4.2.0, and brace-expansion now carries advisories as well; both clear with the mocha 11 regen, which resolves js-yaml@4.3.2 and brace-expansion@2.1.7. See the Alert #7/#10 addendum above.)*

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
