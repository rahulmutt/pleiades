# Devkit Phase 3 Slice — cargo-fuzz for the untrusted-byte boundaries

**Date:** 2026-07-18
**Status:** Approved design, pending implementation plan
**Parent:** [`2026-07-17-devkit-adoption-design.md`](./2026-07-17-devkit-adoption-design.md) — Phase 3, second slice
**Sibling:** [`2026-07-17-devkit-phase3-proptest-slice-design.md`](./2026-07-17-devkit-phase3-proptest-slice-design.md) — Phase 3, first slice (merged)

## Goal

Land cargo-fuzz coverage over every untrusted-byte parser named by
[`docs/threat-model.md`](../../threat-model.md) boundary #1, and fix the
robustness defects it surfaces. This makes AGENTS.md's "treat ingestion as
untrusted input" rule — and the threat model's "parsers return structured
errors and must never panic" control — executable rather than aspirational.

Aligns with `devkit:security-practices` (fuzzing the trust boundary that takes
attacker-controlled bytes) and `devkit:testing-practices` (fuzzing as the right
form when the oracle is "no panic, no UB, no hang" rather than a value check).

## Scope correction to the parent design

The parent design's Phase 3 fuzzing bullet names only `pleiades-jpl::ingest`
and the `pleiades-compression` decode path. `docs/threat-model.md:31` defines
the same boundary more broadly:

> Data ingestion (`pleiades-jpl::ingest`, **kernel/corpus loading**,
> `pleiades-compression` decode) | Untrusted bytes

Kernel loading — the SPK/DAF binary parser — is inside the threat model's
boundary but was omitted from the parent design's fuzz scope. A pre-design
audit of the three areas found that the omitted parser is the one carrying live
defects, while `ingest` appears panic-free. **This slice follows the threat
model, not the narrower parent bullet.** The parent design's bullet is treated
as an oversight; no separate amendment to it is required, since this document
supersedes it for the fuzzing work.

## Non-goals

- No `cargo-mutants` — the remaining Phase 3 slice.
- No Phase 4 navigability work.
- No change to `release-gate`, the numeric gates, or the blocking CI tier.
  Fuzzing is nightly-tier work; nothing here may slow `mise run ci`.
- No widening of `pub(crate)` visibility and no `#[cfg(fuzzing)]` hooks in
  production code. Harnesses call the public API only.
- No new format support, no parser rewrites. Fixes are minimal robustness
  changes that must be invisible to valid input.

## Findings that motivate the slice

A pre-design audit identified four defects reachable from public APIs. Each is
a fail-open robustness bug: hostile bytes crash or hang the process instead of
producing `Err`.

| # | Defect | Location |
| --- | --- | --- |
| a | Unbounded record-chain walk. `next` is read from the file and assigned to `rec_no` with no visited-set and no bound; a NEXT field pointing at itself (or backward) loops forever, accumulating descriptors until memory is exhausted. | `crates/pleiades-jpl/src/spk/daf.rs:76-104` |
| b | `usize` overflow on a hostile record number. `record_byte` computes `(record_number - 1) * RECORD_BYTES` unchecked; `next` is `f64 as usize`, which saturates to `usize::MAX` for a large float, so the multiply overflows. Same exposure at `daf.rs:80`, `:82`, `:89` via the attacker-supplied `nsum`. | `crates/pleiades-jpl/src/spk/daf.rs:38` |
| c | `offset + len` is computed before the bounds check in the `ReadAt` impl for `[u8]`, so a large offset overflows the add rather than returning `Truncated`. | `crates/pleiades-jpl/src/spk/mod.rs:92` |
| d | Untrusted `u32` segment count drives `Vec::with_capacity` before any bytes back it. A crafted body header of a few dozen bytes requests a multi-hundred-gigabyte allocation and aborts the process. The checksum gate does not help — FNV-1a is unkeyed, so a crafted payload carries a valid checksum. | `crates/pleiades-compression/src/codec.rs:504-505` |

Defects (b) and (c) manifest as panics specifically because cargo-fuzz builds
with `overflow-checks` enabled; in a release build they would wrap silently
into out-of-bounds offsets, which the `ReadAt` bounds check then rejects. They
are real regardless, and the checked arithmetic is correct in both profiles.

By contrast, the audit found no panic paths in `ingest/`: every slice index
derives from `str::find` on ASCII needles (always a char boundary), field
access is length-guarded, and all float parsing uses `map_err` rather than
`unwrap`. It is fuzzed as a regression guard, not a bug hunt.

## Architecture

### Crate layout

`fuzz/` is a **non-member crate**, cargo-fuzz's default layout. The workspace
uses an explicit `members` list, so `fuzz/` is simply never listed — no
`exclude` entry is needed, mirroring how `tools/se-*-reference` stay out of the
workspace. Consequences, all desirable:

- `cargo build --workspace`, `mise run test`, `mise run lint`, and every
  numeric gate are untouched by fuzz code.
- The nightly toolchain never contaminates the stable build.
- `libfuzzer-sys` and `arbitrary` enter no member crate's dependency graph, so
  `cargo deny`'s view of shipped dependencies is unchanged and the published
  crates gain nothing.

### Toolchain

Per the parent design: pin a **dated nightly** in `mise.toml` alongside stable
`1.97.1`, and pin `cargo:cargo-fuzz`. The `fuzz` task selects the nightly
toolchain explicitly; every other task continues to use stable. The date is
pinned, not floating, per `developer-environment`'s pinning rule — a nightly
that moves under CI would violate the repo's reproducibility stance.

`rust-version = "1.97.1"` in `[workspace.package]` is unchanged: fuzzing is a
development activity, not a consumer-facing MSRV commitment.

### Corpus and crash-artifact policy

`fuzz/corpus/<target>/` is committed so coverage accumulates across runs rather
than restarting cold each night. `fuzz/artifacts/` (crash reproducers) is
committed as well — this is what makes each fix's regression permanent, the
same fail-closed reasoning behind the proptest slice's `proptest-regressions/`
policy: a found counterexample must stay found.

`.gitignore` needs no change. Its bare `target` rule already covers
`fuzz/target/` at any depth, while `fuzz/corpus/` and `fuzz/artifacts/` are
unmatched by any existing rule. The implementation plan verifies this with
`git check-ignore` rather than assuming it.

Corpus growth is bounded by running `cargo fuzz cmin` per target before
committing, so the committed corpus stays coverage-equivalent but small.

## Targets

Four targets, all calling public API only.

### 1. `spk_kernel` — highest yield

Entry: `SpkBackendBuilder::new().add_kernel_bytes(data.to_vec(), "fuzz")`
(`crates/pleiades-jpl/src/spk/backend.rs:39`, re-exported at the crate root).
Reaches `KernelPool::add_bytes` → `DafFile::parse`, covering defects (a), (b),
and (c).

Seed corpus: truncated and byte-flipped derivatives of a real `.bsp` header.
Full kernels are too large to commit and too slow to mutate usefully; headers
plus the first summary record are where the parsing logic lives.

### 2. `compression_framing`

Entry: `CompressedArtifact::decode(data)`
(`crates/pleiades-compression/src/artifact.rs:192`) on raw fuzzer bytes.

Exercises the framing exactly as shipped: magic, version, checksum
verification, and trailing-byte rejection. Most inputs will be rejected at the
checksum — that is the point of this target, and it is why a second target
exists rather than this one being tuned.

### 3. `compression_payload`

Entry: the harness recomputes a valid FNV-1a over the mutated payload,
reassembles a well-framed artifact, then calls the same public
`CompressedArtifact::decode`.

Because the checksum is unkeyed and verified over the whole payload before any
parsing, an unaided fuzzer spends its entire budget bouncing off the hash and
never reaches the codec — including defect (d). The fixup lives **entirely in
the fuzz harness**; production code is unchanged and the target still calls
only the public entry point.

Splitting framing from payload keeps the split honest: neither target pretends
to validate what the other covers.

### 4. `ingest_corpus`

Entry: `read_public_corpus(data, &ExpectedProfile::default())`
(`crates/pleiades-jpl/src/ingest/mod.rs:35`). One call covers format detection
plus all three front-ends (Horizons vector table, Horizons API JSON, generic
CSV) plus the normalizer.

The JSON front-end is the most interesting surface here: it is a hand-rolled
scanner that extracts the `result` field and feeds it back into
`vector_table::parse`, so the two parsers nest.

Seed corpus: minimal valid examples of each of the three formats.

### Oracle

Uniform across targets, per the parent design: **no panics, no UB, no hangs.**
Parsers must return `Err`. Rejecting malformed input is success; crashing or
looping on it is failure. libFuzzer's `-timeout` catches (a); `overflow-checks`
catches (b) and (c); the OOM limit catches (d).

## Fixes

### DAF record-chain walk — defects (a), (b), (c)

One coherent change across `spk/daf.rs` and `spk/mod.rs`:

- **Cycle detection:** track visited record numbers during the chain walk and
  return a structured `SpkError` on revisit. This bounds the walk by the file's
  actual record count, with no magic constant — a well-formed DAF never
  revisits a record, so no legitimate kernel, however large, can be rejected.
- **Checked arithmetic:** convert the offset math in `record_byte` and the
  descriptor/name offset computations to `checked_mul`/`checked_add`, returning
  the existing truncation error variant on overflow.
- **Bounds check before add:** in the `ReadAt` impl for `[u8]`, use
  `checked_add` for `offset + len` so the overflow itself returns `Truncated`
  rather than panicking.

A hard record cap was rejected: it introduces a constant that could reject a
legitimate very large kernel — precisely the fail-closed-on-valid-data failure
this repo avoids. A monotonic-pointer requirement was rejected as a stricter
reading of the DAF format than NAIF guarantees.

### Compression segment count — defect (d)

Bound `segment_count` against the bytes actually remaining in the cursor
(remaining length divided by the minimum possible encoded segment size) before
`Vec::with_capacity`, returning a structured error when the header's claim
cannot be backed by the input. The equivalent alternative — dropping
`with_capacity` and letting the `Vec` grow as `decode_segment` consumes bytes —
is acceptable if it measures no slower on real artifacts; the implementation
plan picks one on evidence.

The audit confirmed the other capacity sites are safe: their counts are `u8`-
or `u16`-bounded (`codec.rs:277,283,289,346,454,459`, `artifact.rs:228`). Only
the `u32` at `codec.rs:505` is dangerous. No change to those.

### Non-regression requirement

Every fix must be invisible to valid input. Acceptance: real DE440 and asteroid
kernels still load, and all existing SPK and corpus gates stay green. This is
verified by running the gates, not by inspection.

## CI wiring

A **separate workflow**, `.github/workflows/fuzz.yml`, on its own cron
schedule with `workflow_dispatch`, its own timeout, and its own pinned
tracking issue via the existing fail-loud pattern
(`JasonEtco/create-an-issue@v2` + `.github/fuzz-failure-issue.md`, mirroring
`.github/nightly-failure-issue.md`).

Fuzzing is **not** added to `mise run ci-nightly`. The existing nightly
workflow has `timeout-minutes: 60` and already runs `test-full`,
`package-check`, `benchmark`, `deny`, and `secrets`; four targets at the parent
design's ~10 min each would overrun that envelope and fail nightly on duration,
tripping the pinned-issue alarm for a non-bug.

Separation also means fuzz runtime scales independently of the test tier, a
wedged target cannot delay or mask a test-tier regression, and per-target
budgets can grow later without renegotiating nightly's envelope.

**Budgets.** Two distinct numbers, not to be conflated:

- **Ongoing cron budget:** ~10 min per target (the parent design's figure), so
  the whole workflow lands near 40 minutes plus checkout and build. Coverage
  accumulates through the committed corpus across runs.
- **One-time merge gate:** 30 min per target (see Merge criterion). This is a
  deeper one-off campaign proving the baseline is clean, not the steady-state
  schedule.

Tasks exposed in `mise.toml`:

- `mise run fuzz` — run all targets for a short default budget (local use).
- `mise run fuzz-target <name>` — single target, explicit budget.

Both select the pinned nightly toolchain explicitly. Neither is reachable from
`ci` or `release-gate`.

### Relationship to the parent design's safety rule

The parent design requires that anything moved to nightly stay reachable from
`release-gate`. That rule governs **existing validation being relocated**;
fuzzing is net-new and gates nothing today, so `release-gate` is untouched.
The fixes themselves are protected at release time by the committed crash
reproducers, which run as ordinary unit tests in the blocking tier — so the
security property this slice establishes is enforced on every push, not only
on the fuzz cron.

## Merge criterion

The slice merges when:

1. Each of the four targets runs **30 minutes on the accumulated corpus with
   zero new crashes**, on the pinned nightly toolchain.
2. Every defect found — the four above and anything new — is fixed.
3. Every fix is pinned by a committed crash reproducer that runs as an ordinary
   test in the blocking tier.
4. All existing gates and the full test suite stay green.
5. The clean 30-minute runs are recorded in the plan's verification notes with
   their libFuzzer output, per `verification-before-completion`.

A stopping rule is required because "run until clean" is undefined — fuzzing
never proves absence of bugs, it only stops finding them. A bounded,
reproducible campaign makes the branch reviewable and lets any later nightly
finding be an ordinary bug report against a green baseline rather than an
open-ended branch blocker.

## Acceptance criteria

1. `fuzz/` builds on the pinned nightly; all four targets compile and run.
2. Defects (a)–(d) are fixed, each with a committed reproducer that fails
   before the fix and passes after.
3. 30-minute clean campaign per target, output recorded.
4. `fuzz.yml` runs on cron, fails loud via a pinned issue, and closes it on
   success.
5. `mise run ci` wall-clock is unchanged (fuzzing is not in the blocking tier).
6. Real DE440 and asteroid kernels still load; all SPK, corpus, and numeric
   gates green.
7. `docs/threat-model.md` boundary #1 updated: fuzzing moves from "planned
   (design Phase 3)" to shipped, naming the four targets.
8. AGENTS.md tool list includes cargo-fuzz and the nightly pin.

## Risks

- **Nightly toolchain drift.** A dated nightly pin can break on a future
  `cargo-fuzz` bump. Mitigated by pinning both and updating them together;
  Renovate's cadence surfaces the pair.
- **Campaign finds more than four defects.** Accepted deliberately: the branch
  fixes everything it finds. If the count becomes large enough to threaten
  reviewability, the fallback is to split the fixes into a follow-up branch
  with the harness landing first — decided on evidence, not pre-emptively.
- **Committed corpus growth.** Bounded by `cargo fuzz cmin` before each commit.
