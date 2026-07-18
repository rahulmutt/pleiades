# Devkit Phase 3 — cargo-fuzz Slice Implementation Plan

> **Errata (added post-implementation, Task 9):** This plan is a historical
> record and is left otherwise unedited; the shipped tree is correct. Two
> errors in the text below were caught and worked around during
> implementation:
>
> 1. **Task 5, Steps 2 and 4** give the FNV-1a prime as `0x1000_0000_01b3`
>    (1 extra hex digit, evaluates to 17592186044851). The correct
>    FNV-1a-64 prime — and the value `crates/pleiades-compression/src/codec.rs`
>    actually uses — is `0x100_0000_01b3` = 1099511628211. The plan's own
>    Step 4 pinning test caught this; it was implemented correctly.
> 2. **Task 6, Step 4**'s literal `vector_table.txt` example is
>    space/`=`-separated and does not parse against the current
>    `vector_table.rs`, which requires at least 5 comma-separated fields. The
>    shipped seeds instead reuse the crate's own committed fixtures
>    (`crates/pleiades-jpl/tests/fixtures/ingest/{horizons_vectors.txt,horizons_api.json,generic.csv}`).

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fuzz every untrusted-byte parser named by `docs/threat-model.md` boundary #1 (SPK/DAF kernel loading, compression artifact decode, JPL corpus ingest) and fix every robustness defect found.

**Architecture:** Four libFuzzer targets live in a non-member `fuzz/` crate driven by a pinned nightly toolchain, so no fuzz dependency enters any published crate's graph. The four known defects are fixed first, TDD-style, using crafted byte arrays as ordinary unit tests — this means the security properties are enforced in the blocking CI tier on every push, not only on the fuzz cron. The campaign then runs against a known-good baseline.

**Tech Stack:** Rust (stable 1.97.1 + pinned dated nightly), cargo-fuzz / libFuzzer, mise task runner, cargo-nextest, GitHub Actions.

**Spec:** [`docs/superpowers/specs/2026-07-18-devkit-phase3-fuzz-slice-design.md`](../specs/2026-07-18-devkit-phase3-fuzz-slice-design.md)

**Branch:** `devkit-phase3-fuzz` (already created; spec committed as `2bc9b3fb`)

## Global Constraints

- **No production `#[cfg(fuzzing)]` hooks and no widening of `pub(crate)` visibility.** Harnesses call public API only. The compression checksum fixup lives entirely in the harness.
- **Fixes must be invisible to valid input.** Real DE440 and asteroid kernels must still load; all existing SPK, corpus, and numeric gates must stay green.
- **No magic constants that could reject a legitimate kernel.** Bounds derive from the input's own size or from cycle detection.
- **`mise run ci` (blocking tier) wall-clock must not increase.** Fuzzing is never added to `ci` or `release-gate`.
- **Fuzzing is NOT added to `mise run ci-nightly`** — the existing nightly workflow has `timeout-minutes: 60` and would overrun.
- **All tool versions pinned** per `developer-environment` (dated nightly, not floating).
- **Rust edition 2021; `rust-version = "1.97.1"` in `[workspace.package]` is unchanged.**
- **Commit style:** Conventional Commits, matching the existing log (`fix(spk): …`, `test(fuzz): …`, `ci(fuzz): …`, `docs: …`).
- **Existing error types are reused, never extended:** `SpkError::new(SpkErrorKind::…, msg)` and `CompressionError::new(CompressionErrorKind::…, msg)`.

## File Structure

**Fixes (production code, Tasks 1–2):**
- Modify: `crates/pleiades-jpl/src/spk/daf.rs` — cycle detection + checked offset arithmetic in the summary-record walk.
- Modify: `crates/pleiades-jpl/src/spk/mod.rs:91-101` — `checked_add` in the sole `impl ReadAt for [u8]`.
- Modify: `crates/pleiades-compression/src/codec.rs:504-505` — bound the untrusted `u32` segment count before `Vec::with_capacity`.

**Fuzz crate (Tasks 3–6), all new, non-member:**
- `fuzz/Cargo.toml` — standalone crate with its own `[workspace]` table.
- `fuzz/.gitignore` — ignore `target/` and `Cargo.lock` is committed.
- `fuzz/fuzz_targets/spk_kernel.rs` — DAF/SPK kernel bytes.
- `fuzz/fuzz_targets/compression_framing.rs` — raw bytes through real `decode`.
- `fuzz/fuzz_targets/compression_payload.rs` — checksum-fixup harness reaching the codec.
- `fuzz/fuzz_targets/ingest_corpus.rs` — text corpus ingest.
- `fuzz/corpus/<target>/` — committed seed + minimized corpora.
- `fuzz/artifacts/<target>/` — committed crash reproducers.

**CI and docs (Tasks 7–9):**
- `.github/workflows/fuzz.yml` — own cron, own timeout, own pinned issue.
- `.github/fuzz-failure-issue.md` — issue template, mirroring `nightly-failure-issue.md`.
- Modify: `mise.toml` — nightly + cargo-fuzz pins, `fuzz` / `fuzz-target` tasks.
- Modify: `docs/threat-model.md` — boundary #1 fuzzing moves from planned to shipped.
- Modify: `AGENTS.md` — tool list and nightly pin.

**Task ordering rationale:** fixes precede the fuzz crate so that each fix carries a plain, fast unit test (fixes are independently valuable and reviewable even if the harness work stalls), and so the Task 8 campaign starts from a known-good baseline rather than rediscovering the four known defects.

---

### Task 1: Harden the DAF summary-record walk (defects a, b, c)

Fixes the unbounded record-chain walk, the `usize` overflow on a hostile record number, and the `offset + len` overflow in `ReadAt`.

**Files:**
- Modify: `crates/pleiades-jpl/src/spk/daf.rs:36-38` (`record_byte`), `:73-104` (the walk)
- Modify: `crates/pleiades-jpl/src/spk/mod.rs:91-101` (`impl ReadAt for [u8]`)
- Test: `crates/pleiades-jpl/src/spk/daf.rs` (co-located `#[cfg(test)] mod tests`, per AGENTS.md test layout)
- Test: `crates/pleiades-jpl/src/spk/mod.rs` (co-located `#[cfg(test)] mod tests`)

**Interfaces:**
- Consumes: `build_daf(&[SegmentSpec]) -> Vec<u8>` and `const_seg` from `crate::spk::test_support` (already used by `daf.rs` tests and `backend.rs:254`). `build_daf` writes FWARD = record 2, LOCFMT = `LTL-IEEE`, so the summary record starts at byte offset 1024 and its NEXT field is the little-endian `f64` at bytes `[1024..1032]`.
- Produces: `DafFile::parse` returns `Err(SpkError)` — never panics, never loops — for cyclic or out-of-range record chains. `record_byte` becomes fallible: `fn record_byte(record_number: usize) -> Result<usize, SpkError>`. Callers in `daf.rs` use `?`. `addr_to_byte` is unchanged.

- [ ] **Step 1: Write the failing tests**

Add to the existing `#[cfg(test)] mod tests` in `crates/pleiades-jpl/src/spk/daf.rs` (it already has `use super::*;` and imports `build_daf`, `SegmentSpec` — add `const_seg` to that import if absent, or construct a `SegmentSpec` the same way the neighbouring tests do):

```rust
    /// Overwrites the NEXT field of the summary record at record 2.
    /// `build_daf` sets FWARD = 2, so the walk starts there and NEXT is the
    /// little-endian f64 at byte offset 1024.
    fn set_summary_next(blob: &mut [u8], next: f64) {
        blob[1024..1032].copy_from_slice(&next.to_le_bytes());
    }

    #[test]
    fn rejects_self_referential_record_chain() {
        let mut blob = build_daf(&[const_seg(10, 0, [1.0e8, 0.0, 0.0])]);
        // Record 2's NEXT points at record 2 — a cycle.
        set_summary_next(&mut blob, 2.0);

        let err = DafFile::parse(blob.as_slice())
            .expect_err("a self-referential record chain must be rejected, not looped on");
        assert_eq!(err.kind, SpkErrorKind::BadHeader);
    }

    #[test]
    fn rejects_record_number_that_overflows_offset_arithmetic() {
        let mut blob = build_daf(&[const_seg(10, 0, [1.0e8, 0.0, 0.0])]);
        // `as usize` saturates this to usize::MAX, overflowing (n - 1) * 1024.
        set_summary_next(&mut blob, 1.0e300);

        let err = DafFile::parse(blob.as_slice())
            .expect_err("an out-of-range record number must be rejected, not overflow");
        assert_eq!(err.kind, SpkErrorKind::Truncated);
    }

    #[test]
    fn valid_kernel_still_parses_after_hardening() {
        let blob = build_daf(&[
            const_seg(10, 0, [1.0e8, 0.0, 0.0]),
            const_seg(399, 3, [0.0, 0.0, 0.0]),
        ]);
        let daf = DafFile::parse(blob.as_slice()).expect("a well-formed DAF must still parse");
        assert_eq!(daf.segments.len(), 2);
    }
```

Add to the existing `#[cfg(test)] mod tests` in `crates/pleiades-jpl/src/spk/mod.rs`:

```rust
    #[test]
    fn read_at_rejects_offset_len_overflow_without_panicking() {
        let data: &[u8] = &[1, 2, 3, 4];
        let err = data
            .read_at(usize::MAX, 8)
            .expect_err("offset + len overflow must return Truncated, not panic");
        assert_eq!(err.kind, SpkErrorKind::Truncated);
    }
```

- [ ] **Step 2: Run the tests to verify they fail**

Run:
```bash
timeout 60 cargo nextest run -p pleiades-jpl \
  rejects_self_referential_record_chain \
  rejects_record_number_that_overflows_offset_arithmetic \
  read_at_rejects_offset_len_overflow_without_panicking
```

Expected, and each failure mode is different — check for all three:
- `rejects_self_referential_record_chain` — **hangs** until the 60 s `timeout` kills it (exit code 124), or is killed by nextest's slow-test timeout. The hang *is* defect (a); a hanging test here is the expected pre-fix signal, not a broken test.
- `rejects_record_number_that_overflows_offset_arithmetic` — FAILS with `attempt to multiply with overflow` (debug builds enable `overflow-checks`).
- `read_at_rejects_offset_len_overflow_without_panicking` — FAILS with `attempt to add with overflow`.
- `valid_kernel_still_parses_after_hardening` — PASSES already; it is the non-regression guard.

If the first test hangs, the run will not report the others. Re-run them individually to confirm each failure:
```bash
timeout 60 cargo nextest run -p pleiades-jpl rejects_record_number_that_overflows_offset_arithmetic
timeout 60 cargo nextest run -p pleiades-jpl read_at_rejects_offset_len_overflow_without_panicking
```

- [ ] **Step 3: Fix `ReadAt` (defect c)**

In `crates/pleiades-jpl/src/spk/mod.rs`, replace the body of `read_at` in `impl ReadAt for [u8]` (lines 91-101):

```rust
    fn read_at(&self, offset: usize, len: usize) -> Result<&[u8], SpkError> {
        let end = offset.checked_add(len).ok_or_else(|| {
            SpkError::new(
                SpkErrorKind::Truncated,
                format!("read of {len} bytes at {offset} overflowed a usize"),
            )
        })?;
        self.get(offset..end).ok_or_else(|| {
            SpkError::new(
                SpkErrorKind::Truncated,
                format!(
                    "read of {len} bytes at {offset} exceeds slice length {}",
                    <[u8]>::len(self)
                ),
            )
        })
    }
```

Note `<[u8]>::len(self)` in the message: inside this impl, bare `self.len()` resolves to the trait method. The original code relied on that resolution; keeping the fully-qualified form makes the intent explicit and avoids accidental recursion if the trait method is ever changed.

- [ ] **Step 4: Fix `record_byte` and the record walk (defects a, b)**

In `crates/pleiades-jpl/src/spk/daf.rs`, replace `record_byte` (lines 36-38) with a fallible version plus two arithmetic helpers:

```rust
fn offset_overflow() -> SpkError {
    SpkError::new(
        SpkErrorKind::Truncated,
        "DAF offset arithmetic overflowed a usize",
    )
}

fn checked_add(a: usize, b: usize) -> Result<usize, SpkError> {
    a.checked_add(b).ok_or_else(offset_overflow)
}

fn checked_mul(a: usize, b: usize) -> Result<usize, SpkError> {
    a.checked_mul(b).ok_or_else(offset_overflow)
}

fn record_byte(record_number: usize) -> Result<usize, SpkError> {
    record_number
        .checked_sub(1)
        .and_then(|zero_based| zero_based.checked_mul(RECORD_BYTES))
        .ok_or_else(|| {
            SpkError::new(
                SpkErrorKind::Truncated,
                format!("DAF record number {record_number} is out of range"),
            )
        })
}
```

Then replace the walk (lines 73-104, from `let mut segments = Vec::new();` through `Ok(DafFile { endian, segments })`) with:

```rust
        let mut segments = Vec::new();
        // A well-formed DAF never revisits a summary record. Tracking visited
        // records bounds the walk by the file's own record count, so no
        // legitimate kernel — however large — can be rejected, while a cyclic
        // or backward-pointing NEXT field terminates with an error instead of
        // looping until memory is exhausted.
        let mut visited = std::collections::HashSet::new();
        let mut rec_no = fward;
        while rec_no != 0 {
            if !visited.insert(rec_no) {
                return Err(SpkError::new(
                    SpkErrorKind::BadHeader,
                    format!("DAF summary record chain revisits record {rec_no}"),
                ));
            }
            let base = record_byte(rec_no)?;
            let next = endian.f64_at(src, base)? as usize;
            let nsum = endian.f64_at(src, checked_add(base, 16)?)? as usize; // 3rd double
            let name_base = record_byte(checked_add(rec_no, 1)?)?; // name record follows summary record
            for k in 0..nsum {
                // skip NEXT/PREV/NSUM, then k summaries
                let s = checked_add(base, checked_mul(checked_add(3, checked_mul(k, ss)?)?, 8)?)?;
                let start_et = endian.f64_at(src, s)?;
                let stop_et = endian.f64_at(src, checked_add(s, 8)?)?;
                let (target, center) = endian.packed_i32_pair_at(src, checked_add(s, 16)?)?;
                let (frame, data_type) = endian.packed_i32_pair_at(src, checked_add(s, 24)?)?;
                let (init_addr, final_addr) =
                    endian.packed_i32_pair_at(src, checked_add(s, 32)?)?;
                let nc = checked_mul(ss, 8)?;
                let raw = src.read_at(checked_add(name_base, checked_mul(k, nc)?)?, nc)?;
                let name = String::from_utf8_lossy(raw).trim_end().to_string();
                segments.push(SegmentDescriptor {
                    start_et,
                    stop_et,
                    target,
                    center,
                    frame,
                    data_type,
                    init_addr,
                    final_addr,
                    name,
                });
            }
            rec_no = next;
        }
        Ok(DafFile { endian, segments })
```

A hostile `nsum` needs no separate bound: the first read past the end of the file returns `Truncated`, so the inner loop terminates on the input's actual size.

- [ ] **Step 5: Run the tests to verify they pass**

```bash
timeout 120 cargo nextest run -p pleiades-jpl spk::
```
Expected: PASS, including all three new tests and every pre-existing `spk::` test. No hang.

- [ ] **Step 6: Verify no other `ReadAt` impl was missed**

```bash
grep -rn "impl ReadAt for" crates/
```
Expected: exactly one hit, `crates/pleiades-jpl/src/spk/mod.rs:87`. If a second impl exists (e.g. a file-backed reader added later), apply the same `checked_add` fix to it before continuing.

- [ ] **Step 7: Verify real kernels still load**

```bash
mise run test
timeout 1800 mise run gate-corpus
```
Expected: both PASS. This is the non-regression evidence required by the Global Constraints — the fixes must be invisible to valid input. If `gate-corpus` needs kernels that are not present locally (`.kernels/` is gitignored), record that it was skipped and rely on the Task 9 full-suite run plus CI.

- [ ] **Step 8: Commit**

```bash
git add crates/pleiades-jpl/src/spk/daf.rs crates/pleiades-jpl/src/spk/mod.rs
git commit -m "fix(spk): bound DAF record chain and check offset arithmetic

A crafted kernel could loop forever on a self-referential summary-record
NEXT pointer, or overflow usize offset arithmetic via a saturating
f64-to-usize record number. Cycle detection bounds the walk by the file's
own record count (no magic constant, so no legitimate kernel is rejected),
and checked arithmetic returns Truncated instead of panicking.

Threat model boundary #1: parsers must return structured errors, never panic."
```

---

### Task 2: Bound the untrusted segment count in artifact decode (defect d)

A crafted body header claims a `u32` segment count that no input backs, and `Vec::with_capacity` tries to allocate for it — a few dozen bytes request hundreds of gigabytes and abort the process. The unkeyed FNV-1a checksum offers no protection, since an attacker computes it over their own payload.

**Files:**
- Modify: `crates/pleiades-compression/src/codec.rs:504-505` (`decode_body`)
- Test: `crates/pleiades-compression/src/codec.rs` (co-located `#[cfg(test)] mod tests`)

**Interfaces:**
- Consumes: `Cursor::remaining(&self) -> &'a [u8]` (`codec.rs:57`), `Cursor::new` (`codec.rs:53`), `decode_body(cursor: &mut Cursor<'_>) -> Result<BodyArtifact, CompressionError>` (`codec.rs:499`). All are `pub(crate)`, so this test is white-box and must live inside the crate.
- Produces: `decode_body` returns `Err(CompressionError)` with kind `InvalidFormat` when the declared segment count exceeds what the remaining bytes could possibly encode. Behaviour on valid input is unchanged.

**Minimum encoded segment size — derivation (do not change without re-deriving):** `decode_segment` (`codec.rs:450`) reads two instants plus two count bytes before any channel data. `decode_instant` (`codec.rs:310`) reads one `f64` (8 bytes) plus one `u8` time scale = 9 bytes. So the floor is `9 + 9 + 1 + 1 = 20` bytes. `validate_segment` may reject segments this small, which only makes the real minimum larger — 20 is a safe lower bound, which is all the check needs.

- [ ] **Step 1: Write the failing test**

Add to the `#[cfg(test)] mod tests` in `crates/pleiades-compression/src/codec.rs`:

```rust
    #[test]
    fn decode_body_rejects_segment_count_larger_than_remaining_bytes() {
        // A body header whose declared segment count cannot possibly be backed
        // by the bytes that follow. Without a bound, this reaches
        // Vec::with_capacity(u32::MAX as usize) and aborts the process.
        let mut bytes = Vec::new();
        encode_celestial_body(&mut bytes, &CelestialBody::Sun).unwrap();
        write_u8(&mut bytes, encode_stored_frame(StoredFrame::Geocentric));
        write_u32(&mut bytes, u32::MAX);
        // No segment bytes follow at all.

        let mut cursor = Cursor::new(&bytes);
        let err = decode_body(&mut cursor)
            .expect_err("an unbacked segment count must be rejected before allocating");
        assert_eq!(err.kind, CompressionErrorKind::InvalidFormat);
    }
```

If `CelestialBody`, `StoredFrame`, `encode_celestial_body`, `write_u8`, `write_u32`, or `encode_stored_frame` are not already in scope in that test module, add them — the module's existing `use super::*;` covers the codec's own items; `CelestialBody` and `StoredFrame` come from `crate::channels`.

- [ ] **Step 2: Run the test to verify it fails**

```bash
timeout 120 cargo nextest run -p pleiades-compression decode_body_rejects_segment_count_larger_than_remaining_bytes
```
Expected: the test process **aborts** (memory allocation failure / SIGABRT), or fails with an allocation error. It will not fail with a clean assertion — that abort is defect (d).

On a machine with generous overcommit the allocation may succeed and the test may instead fail on the `expect_err`. Either outcome confirms the defect; both are fixed by Step 3.

- [ ] **Step 3: Implement the bound**

In `crates/pleiades-compression/src/codec.rs`, add near the top of the file alongside the other module constants:

```rust
/// Smallest number of bytes any encoded segment can occupy: two instants
/// (`f64` + time-scale byte each) plus the channel and residual-channel count
/// bytes. Used to reject a declared segment count that the remaining input
/// could not possibly back, before it reaches `Vec::with_capacity`.
const MIN_ENCODED_SEGMENT_BYTES: usize = 20;
```

Replace lines 504-505 in `decode_body`:

```rust
    let segment_count = cursor.read_u32()? as usize;
    let max_possible_segments = cursor.remaining().len() / MIN_ENCODED_SEGMENT_BYTES;
    if segment_count > max_possible_segments {
        return Err(CompressionError::new(
            CompressionErrorKind::InvalidFormat,
            format!(
                "compressed artifact declared {segment_count} segments but only \
                 {} bytes remain (at most {max_possible_segments} possible)",
                cursor.remaining().len()
            ),
        ));
    }
    let mut segments = Vec::with_capacity(segment_count);
```

This preserves the `with_capacity` performance intent while making the capacity provably backed by real input. It introduces no constant that could reject valid data: a genuine artifact's segment count is always at most `remaining / 20`.

- [ ] **Step 4: Run the test to verify it passes**

```bash
timeout 120 cargo nextest run -p pleiades-compression
```
Expected: PASS, including the new test and every pre-existing compression test (round-trip, codec byte-stability, and the proptest suites from the Phase 3 first slice).

- [ ] **Step 5: Confirm the other capacity sites need no change**

```bash
grep -n "with_capacity" crates/pleiades-compression/src/codec.rs crates/pleiades-compression/src/artifact.rs
```
Expected hits and their bounds — verify each count is `u8`- or `u16`-derived and therefore already bounded: `codec.rs:277,283,289,346` (`u8`, ≤255), `codec.rs:454,459` (`u8`), `artifact.rs:228` (`u16`, ≤65535). Only the `u32` in `decode_body` was dangerous. Make no changes to these; if a new `u32`-driven site has appeared, bound it the same way.

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-compression/src/codec.rs
git commit -m "fix(compression): bound declared segment count before allocating

A crafted body header could declare u32::MAX segments, driving
Vec::with_capacity to a multi-hundred-gigabyte allocation and aborting the
process from a few dozen input bytes. The unkeyed FNV-1a checksum is no
defence — an attacker computes it over their own payload. The count is now
checked against the bytes actually remaining.

Threat model boundary #1: parsers must return structured errors, never panic."
```

---

### Task 3: Scaffold the fuzz crate and pin the nightly toolchain

**Files:**
- Create: `fuzz/Cargo.toml`, `fuzz/.gitignore`
- Modify: `mise.toml`
- Verify: `.gitignore` (expected: no change needed)

**Interfaces:**
- Consumes: nothing from earlier tasks.
- Produces: a buildable non-member `fuzz` crate whose targets are added in Tasks 4–6; `mise run fuzz` and `mise run fuzz-target <name>` task names used by Task 7's workflow. The crate is named `pleiades-fuzz`, `publish = false`, version `0.0.0`.

- [ ] **Step 1: Pin the toolchain and cargo-fuzz in `mise.toml`**

`mise.toml` currently pins `rust = { version = "1.97.1", components = "rustfmt,clippy" }` on line 2. Add the nightly and cargo-fuzz pins alongside it in the `[tools]` section:

```toml
# Fuzzing requires a nightly toolchain (libFuzzer's -Z sanitizer flags). Pinned
# to a DATED nightly, never a floating one — a nightly that moved under CI
# would break the repo's reproducibility stance. Stable 1.97.1 above remains
# the toolchain for every other task; nothing in the blocking tier or
# release-gate uses this.
"rust-nightly" = "nightly-2026-07-01"
"cargo:cargo-fuzz" = "0.12.0"
```

Verify both resolve before continuing:
```bash
mise install
mise exec -- cargo fuzz --version
```
Expected: prints a `cargo-fuzz 0.12.x` version string.

If mise's rust backend does not accept a second `rust` entry under a distinct key, install the dated nightly via rustup instead and record the exact pin in the same comment block:
```bash
rustup toolchain install nightly-2026-07-01
```
Then have the `fuzz` tasks invoke `cargo +nightly-2026-07-01`. Pick whichever works on this machine and keep the pinned date identical either way — do not leave both paths in the file.

- [ ] **Step 2: Add the mise tasks**

Append to `mise.toml`, after the existing `[tasks.benchmark]` block:

```toml
[tasks.fuzz]
description = "Run all fuzz targets for a short budget (local smoke; NOT part of ci or ci-nightly)."
run = """
set -euo pipefail
for target in spk_kernel compression_framing compression_payload ingest_corpus; do
  echo "── fuzzing $target ──"
  cargo +nightly-2026-07-01 fuzz run "$target" -- -max_total_time=60
done
"""

[tasks.fuzz-target]
description = "Run one fuzz target for an explicit budget: mise run fuzz-target <name> <seconds>."
run = "cargo +nightly-2026-07-01 fuzz run {{arg(name=\"target\")}} -- -max_total_time={{arg(name=\"seconds\")}}"
```

Neither task is added to `ci`, `ci-nightly`, or `release-gate` — confirm in the next step.

- [ ] **Step 3: Verify the blocking and nightly tiers are untouched**

```bash
grep -n -A3 "^\[tasks.ci\]\|^\[tasks.ci-nightly\]\|^\[tasks.release-gate\]" mise.toml
```
Expected: `ci` depends on `fmt, lint, docs, test, doctest, secrets, deny, claims-audit` (or whatever it listed before this task — the point is that **no `fuzz` entry appears**), `ci-nightly` depends on `test-full, package-check, benchmark, deny, secrets` with no `fuzz`, and `release-gate` is unchanged.

- [ ] **Step 4: Create the fuzz crate manifest**

Create `fuzz/Cargo.toml`:

```toml
# Standalone, non-member crate. The root workspace uses an explicit `members`
# list and this path is deliberately absent from it, so libfuzzer-sys and
# arbitrary never enter any published crate's dependency graph and cargo deny's
# view of shipped dependencies is unchanged. The empty [workspace] table below
# makes this its own workspace root rather than an orphan inside the parent.
[package]
name = "pleiades-fuzz"
version = "0.0.0"
publish = false
edition = "2021"

[package.metadata]
cargo-fuzz = true

[workspace]

[dependencies]
libfuzzer-sys = "0.4"
pleiades-jpl = { path = "../crates/pleiades-jpl" }
pleiades-compression = { path = "../crates/pleiades-compression" }

[profile.release]
debug = 1

[[bin]]
name = "spk_kernel"
path = "fuzz_targets/spk_kernel.rs"
test = false
doc = false
bench = false

[[bin]]
name = "compression_framing"
path = "fuzz_targets/compression_framing.rs"
test = false
doc = false
bench = false

[[bin]]
name = "compression_payload"
path = "fuzz_targets/compression_payload.rs"
test = false
doc = false
bench = false

[[bin]]
name = "ingest_corpus"
path = "fuzz_targets/ingest_corpus.rs"
test = false
doc = false
bench = false
```

Create `fuzz/.gitignore`:

```
target
```

- [ ] **Step 5: Verify the root workspace does not absorb the fuzz crate**

```bash
grep -n "fuzz" Cargo.toml
cargo metadata --no-deps --format-version 1 | grep -o '"name":"pleiades-fuzz"' || echo "correctly excluded"
```
Expected: the first command prints nothing (no `fuzz` in `members` or `exclude` — the explicit `members` list is what keeps it out). The second prints `correctly excluded`.

- [ ] **Step 6: Verify corpus and artifact directories are committable**

```bash
mkdir -p fuzz/corpus/spk_kernel fuzz/artifacts/spk_kernel
git check-ignore -v fuzz/corpus/spk_kernel fuzz/artifacts/spk_kernel || echo "corpus and artifacts are committable"
git check-ignore -v fuzz/target || echo "UNEXPECTED: fuzz/target is not ignored"
```
Expected: `corpus and artifacts are committable`, and `fuzz/target` **is** matched by the root `.gitignore`'s bare `target` rule (so the second command prints the match, not the UNEXPECTED line). If `fuzz/target` is not ignored, add `fuzz/target` to `fuzz/.gitignore` — do not weaken the root rule.

- [ ] **Step 7: Verify the workspace build is unaffected**

```bash
cargo build --workspace 2>&1 | tail -5
mise run test
```
Expected: both succeed on **stable**, with no reference to `pleiades-fuzz`, `libfuzzer-sys`, or the nightly toolchain in the output.

- [ ] **Step 8: Commit**

```bash
git add fuzz/Cargo.toml fuzz/.gitignore mise.toml
git commit -m "build(fuzz): scaffold non-member fuzz crate on pinned nightly

cargo-fuzz and a dated nightly are pinned in mise; the fuzz crate is a
standalone non-member so libfuzzer never enters a published crate's graph.
Not wired into ci, ci-nightly, or release-gate."
```

The crate has no targets yet, so `cargo fuzz build` is expected to fail until Task 4. That is fine — this task's deliverable is the scaffold and the proof that the stable build and both CI tiers are unaffected.

---

### Task 4: `spk_kernel` fuzz target

The highest-yield target: it decodes attacker-controlled record pointers and counts.

**Files:**
- Create: `fuzz/fuzz_targets/spk_kernel.rs`
- Create: `fuzz/corpus/spk_kernel/` (seed inputs)
- Create: `fuzz/artifacts/spk_kernel/.gitkeep`

**Interfaces:**
- Consumes: `pleiades_jpl::SpkBackend` and `SpkBackendBuilder::add_kernel_bytes(bytes: Vec<u8>, label: impl Into<String>) -> Result<Self, SpkError>` — both re-exported at the crate root (`crates/pleiades-jpl/src/lib.rs:58-61`, confirmed public). Reaches `KernelPool::add_bytes` → `DafFile::parse`, i.e. the code hardened in Task 1.
- Produces: a runnable `spk_kernel` target and a committed seed corpus for Task 8's campaign.

- [ ] **Step 1: Write the target**

Create `fuzz/fuzz_targets/spk_kernel.rs`:

```rust
#![no_main]

use libfuzzer_sys::fuzz_target;
use pleiades_jpl::SpkBackend;

// Oracle: no panic, no UB, no hang. Rejecting malformed kernel bytes with an
// SpkError is success; crashing or looping on them is the failure this target
// exists to catch. Threat model boundary #1 (kernel/corpus loading).
fuzz_target!(|data: &[u8]| {
    let _ = SpkBackend::builder().add_kernel_bytes(data.to_vec(), "fuzz");
});
```

- [ ] **Step 2: Build the target to verify it compiles**

```bash
cd fuzz && cargo +nightly-2026-07-01 fuzz build spk_kernel 2>&1 | tail -20
```
Expected: builds successfully. If `SpkBackend::builder()` is not found, check the exact re-export list at `crates/pleiades-jpl/src/lib.rs:58-61` and adjust — do not add a new `pub use` to the library to make the harness compile, as that would violate the no-visibility-widening constraint.

- [ ] **Step 3: Generate the seed corpus**

Full kernels are too large to commit and too slow to mutate usefully; the parsing logic lives in the file record and the first summary record. Write a throwaway generator to `/tmp/claude-1000/-workspace/ac3c66ee-5c4e-4b60-91ef-a8a194ff6b11/scratchpad/gen_seeds.rs` — or add a temporary `#[test]` to `pleiades-jpl` that writes the files, then delete it — producing seeds from `build_daf`:

```rust
// Temporary helper — writes seeds, then delete it. Uses the same test_support
// builder the DAF unit tests use, so seeds are structurally valid DAFs.
use crate::spk::test_support::{build_daf, const_seg};

#[test]
fn write_fuzz_seeds() {
    let dir = std::path::Path::new("../../fuzz/corpus/spk_kernel");
    std::fs::create_dir_all(dir).unwrap();

    let full = build_daf(&[
        const_seg(10, 0, [1.0e8, 0.0, 0.0]),
        const_seg(399, 3, [0.0, 0.0, 0.0]),
    ]);
    // A valid two-segment kernel.
    std::fs::write(dir.join("valid_two_segment.bsp"), &full).unwrap();
    // Header plus the first summary record — where the parsing logic lives.
    std::fs::write(dir.join("header_and_summary.bsp"), &full[..full.len().min(2048)]).unwrap();
    // Truncated mid-file-record.
    std::fs::write(dir.join("truncated_file_record.bsp"), &full[..512]).unwrap();
    // Big-endian marker variant.
    let mut big = full.clone();
    big[88..96].copy_from_slice(b"BIG-IEEE");
    std::fs::write(dir.join("big_endian_marker.bsp"), &big).unwrap();
}
```

Run it, confirm the four files exist, then remove the temporary test:
```bash
ls -la fuzz/corpus/spk_kernel/
```
Expected: four `.bsp` files.

- [ ] **Step 4: Smoke-run the target**

```bash
cd fuzz && cargo +nightly-2026-07-01 fuzz run spk_kernel -- -max_total_time=60
```
Expected: runs for 60 s and exits cleanly with **no crashes** — Task 1 fixed the three known DAF defects, so a short run should be clean.

If it *does* crash, that is a genuine new finding. Per the spec's merge criterion the branch fixes everything found: reproduce it with `cargo fuzz run spk_kernel fuzz/artifacts/spk_kernel/<crash-file>`, add a unit test in `daf.rs` that encodes the same input, fix it, and commit the reproducer alongside the fix before moving on.

- [ ] **Step 5: Minimize and stage the corpus**

```bash
cd fuzz && cargo +nightly-2026-07-01 fuzz cmin spk_kernel
mkdir -p artifacts/spk_kernel && touch artifacts/spk_kernel/.gitkeep
```

- [ ] **Step 6: Commit**

```bash
git add fuzz/fuzz_targets/spk_kernel.rs fuzz/corpus/spk_kernel fuzz/artifacts/spk_kernel/.gitkeep
git commit -m "test(fuzz): add spk_kernel target for DAF/SPK kernel bytes

Fuzzes the kernel-loading half of threat model boundary #1 via the public
SpkBackendBuilder::add_kernel_bytes. Seed corpus is committed so coverage
accumulates across runs."
```

---

### Task 5: `compression_framing` and `compression_payload` fuzz targets

Two targets, because the unkeyed FNV-1a checksum is verified over the whole payload before any parsing. An unaided fuzzer spends its entire budget bouncing off the hash and never reaches the codec. Splitting keeps the coverage claim honest: neither target pretends to validate what the other covers.

**Files:**
- Create: `fuzz/fuzz_targets/compression_framing.rs`, `fuzz/fuzz_targets/compression_payload.rs`
- Create: `fuzz/corpus/compression_framing/`, `fuzz/corpus/compression_payload/`
- Create: `fuzz/artifacts/compression_framing/.gitkeep`, `fuzz/artifacts/compression_payload/.gitkeep`

**Interfaces:**
- Consumes: `pleiades_compression::CompressedArtifact::decode(bytes: &[u8]) -> Result<Self, CompressionError>` (`artifact.rs:192`, public, re-exported at `lib.rs:52`) and `pleiades_compression::ARTIFACT_VERSION: u16` (`lib.rs:68`, public, currently `7`).
- Produces: two runnable targets. Note `ARTIFACT_MAGIC` (`lib.rs:70`) and `fnv1a64` (`codec.rs:128`) are **`pub(crate)`** — the payload harness therefore hardcodes the magic bytes and reimplements FNV-1a locally. This is required by the Global Constraint against widening visibility, and is exactly what the spec means by the fixup living entirely in the harness.

- [ ] **Step 1: Write the framing target**

Create `fuzz/fuzz_targets/compression_framing.rs`:

```rust
#![no_main]

use libfuzzer_sys::fuzz_target;
use pleiades_compression::CompressedArtifact;

// Fuzzes the framing exactly as shipped: magic, version, checksum verification
// and trailing-byte rejection. Most inputs are rejected at the checksum — that
// is the point of this target, and why compression_payload exists separately
// rather than this one being tuned to get past the hash.
fuzz_target!(|data: &[u8]| {
    let _ = CompressedArtifact::decode(data);
});
```

- [ ] **Step 2: Write the payload target**

Create `fuzz/fuzz_targets/compression_payload.rs`:

```rust
#![no_main]

use libfuzzer_sys::fuzz_target;
use pleiades_compression::{CompressedArtifact, ARTIFACT_VERSION};

// ARTIFACT_MAGIC and fnv1a64 are pub(crate) in pleiades-compression. Rather
// than widen production visibility or add a #[cfg(fuzzing)] hook, the harness
// carries its own copies: the magic is a fixed 8-byte literal and FNV-1a is a
// five-line standard algorithm. If either ever changes in the library, this
// target stops reaching the codec and the framing target still covers the
// public path — a silent-coverage-loss risk accepted deliberately, and pinned
// by the assertion in the smoke-test step of this task.
const ARTIFACT_MAGIC: [u8; 8] = *b"PLDEPHEM";

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in bytes {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x1000_0000_01b3);
    }
    hash
}

// The fuzzer's bytes are treated as the artifact PAYLOAD. The harness wraps
// them in valid framing with a correct checksum, so the mutator's work lands
// inside the codec instead of dying at the hash. decode() itself is unmodified
// and still the real public entry point.
fuzz_target!(|payload: &[u8]| {
    let mut framed = Vec::with_capacity(payload.len() + 18);
    framed.extend_from_slice(&ARTIFACT_MAGIC);
    framed.extend_from_slice(&ARTIFACT_VERSION.to_le_bytes());
    framed.extend_from_slice(&fnv1a64(payload).to_le_bytes());
    framed.extend_from_slice(payload);

    let _ = CompressedArtifact::decode(&framed);
});
```

- [ ] **Step 3: Build both targets**

```bash
cd fuzz && cargo +nightly-2026-07-01 fuzz build compression_framing compression_payload 2>&1 | tail -20
```
Expected: both build.

- [ ] **Step 4: Verify the harness's FNV-1a matches the library's**

This is the assertion the payload target's comment relies on. Add a temporary `#[test]` inside `pleiades-compression` (where `fnv1a64` is reachable) confirming the harness copy agrees:

```rust
    #[test]
    fn harness_fnv1a64_matches_library() {
        fn harness_fnv1a64(bytes: &[u8]) -> u64 {
            let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
            for byte in bytes {
                hash ^= *byte as u64;
                hash = hash.wrapping_mul(0x1000_0000_01b3);
            }
            hash
        }
        for case in [b"".as_slice(), b"a", b"PLDEPHEM", &[0xff; 64]] {
            assert_eq!(harness_fnv1a64(case), fnv1a64(case), "case {case:?}");
        }
    }
```

```bash
timeout 120 cargo nextest run -p pleiades-compression harness_fnv1a64_matches_library
```
Expected: PASS. **Keep this test** — do not delete it. It is what stops the payload target from silently losing coverage if the library's checksum ever changes. Commit it with this task.

- [ ] **Step 5: Generate seed corpora**

Seed the payload target with real `encode()` output minus its 18-byte framing header, and the framing target with complete artifacts. Add a temporary `#[test]` in `pleiades-compression` that writes them, using the crate's existing artifact builder (`try_build_test_artifact` in `artifact.rs`, or whichever helper the existing round-trip tests use — check `crates/pleiades-compression/src/tests.rs` for the established pattern and reuse it):

```rust
#[test]
fn write_fuzz_seeds() {
    let artifact = /* build via the same helper the round-trip tests use */;
    let encoded = artifact.encode().unwrap();

    let framing_dir = std::path::Path::new("../../fuzz/corpus/compression_framing");
    std::fs::create_dir_all(framing_dir).unwrap();
    std::fs::write(framing_dir.join("valid_artifact.bin"), &encoded).unwrap();
    std::fs::write(framing_dir.join("truncated.bin"), &encoded[..encoded.len() / 2]).unwrap();

    let payload_dir = std::path::Path::new("../../fuzz/corpus/compression_payload");
    std::fs::create_dir_all(payload_dir).unwrap();
    // Strip the 8-byte magic + 2-byte version + 8-byte checksum framing.
    std::fs::write(payload_dir.join("valid_payload.bin"), &encoded[18..]).unwrap();
}
```

Run it, confirm the files exist, then delete this temporary test (unlike Step 4's, this one is scaffolding):
```bash
ls -la fuzz/corpus/compression_framing/ fuzz/corpus/compression_payload/
```

- [ ] **Step 6: Smoke-run both targets**

```bash
cd fuzz && cargo +nightly-2026-07-01 fuzz run compression_framing -- -max_total_time=60
cd fuzz && cargo +nightly-2026-07-01 fuzz run compression_payload -- -max_total_time=60
```
Expected: both clean — Task 2 fixed the known allocation defect.

Confirm the payload target is actually reaching the codec rather than dying early: its reported `cov:` figure should be meaningfully higher than the framing target's. If they are similar, the framing wrap is wrong (check the version bytes and the 18-byte offset) — a payload target that never gets past the hash is worthless, so do not proceed until the coverage numbers differ.

Any crash is a genuine new finding: reproduce, add a unit test in `codec.rs` or `artifact.rs`, fix, and commit the reproducer with the fix.

- [ ] **Step 7: Minimize and commit**

```bash
cd fuzz && cargo +nightly-2026-07-01 fuzz cmin compression_framing
cd fuzz && cargo +nightly-2026-07-01 fuzz cmin compression_payload
mkdir -p artifacts/compression_framing artifacts/compression_payload
touch artifacts/compression_framing/.gitkeep artifacts/compression_payload/.gitkeep
```

```bash
git add fuzz/fuzz_targets/compression_framing.rs fuzz/fuzz_targets/compression_payload.rs \
        fuzz/corpus/compression_framing fuzz/corpus/compression_payload \
        fuzz/artifacts/compression_framing/.gitkeep fuzz/artifacts/compression_payload/.gitkeep \
        crates/pleiades-compression/src/tests.rs
git commit -m "test(fuzz): add compression framing and payload targets

Two targets because the unkeyed FNV-1a is verified before any parsing: one
fuzzes the framing as shipped, one recomputes the checksum in the harness so
the mutator reaches the codec. No production code is modified; a unit test
pins the harness's FNV-1a copy against the library's."
```

Adjust the final `git add` path if the seed-writing test lived in a different file than `src/tests.rs`.

---

### Task 6: `ingest_corpus` fuzz target

A regression guard rather than a bug hunt: the pre-design audit found no panic paths in `ingest/` — every slice index derives from `str::find` on ASCII needles, field access is length-guarded, and all float parsing uses `map_err`. This target locks that property in.

**Files:**
- Create: `fuzz/fuzz_targets/ingest_corpus.rs`
- Create: `fuzz/corpus/ingest_corpus/`
- Create: `fuzz/artifacts/ingest_corpus/.gitkeep`

**Interfaces:**
- Consumes: `pleiades_jpl::ingest::read_public_corpus(bytes: &[u8], expected: &ExpectedProfile) -> Result<PublicCorpus, IngestError>` (`ingest/mod.rs:35`) and `ExpectedProfile`, which derives `Default` (`ingest/profile.rs:26`). One call covers format detection plus all three front-ends plus the normalizer.
- Produces: a runnable `ingest_corpus` target.

- [ ] **Step 1: Confirm the ingest module is publicly reachable**

```bash
grep -n "pub mod ingest\|pub use ingest" crates/pleiades-jpl/src/lib.rs
```
Expected: `ingest` is a public module. If it is not, adjust the target's import path to whatever the crate actually exposes — do not add a new `pub` to the library.

- [ ] **Step 2: Write the target**

Create `fuzz/fuzz_targets/ingest_corpus.rs`:

```rust
#![no_main]

use libfuzzer_sys::fuzz_target;
use pleiades_jpl::ingest::{read_public_corpus, ExpectedProfile};

// One call covers format detection plus all three front-ends (Horizons vector
// table, Horizons API JSON, generic CSV) plus the normalizer. The JSON path is
// the most interesting surface: a hand-rolled scanner extracts the `result`
// field and feeds it back into vector_table::parse, so the two parsers nest.
//
// A pre-design audit found no panic paths here, so this is a regression guard
// on threat model boundary #1 rather than a bug hunt.
fuzz_target!(|data: &[u8]| {
    let _ = read_public_corpus(data, &ExpectedProfile::default());
});
```

- [ ] **Step 3: Build the target**

```bash
cd fuzz && cargo +nightly-2026-07-01 fuzz build ingest_corpus 2>&1 | tail -20
```
Expected: builds.

- [ ] **Step 4: Write the seed corpus**

Detection is first-match-wins over three sniffs, so seed one minimal input per format. Create these files directly:

`fuzz/corpus/ingest_corpus/vector_table.txt` — detection requires both `$$SOE` and `$$EOE`:
```
$$SOE
2451545.000000000 = A.D. 2000-Jan-01 12:00:00.0000 TDB
 X = 1.0E-01 Y = 2.0E-01 Z = 3.0E-01
 VX= 1.0E-03 VY= 2.0E-03 VZ= 3.0E-03
$$EOE
```

`fuzz/corpus/ingest_corpus/horizons_api.json` — detection requires a leading `{` plus both `"result"` and `"signature"`:
```json
{"signature":{"source":"NASA/JPL Horizons API","version":"1.2"},"result":"$$SOE\n2451545.000000000 = A.D. 2000-Jan-01 12:00:00.0000 TDB\n X = 1.0E-01 Y = 2.0E-01 Z = 3.0E-01\n VX= 1.0E-03 VY= 2.0E-03 VZ= 3.0E-03\n$$EOE\n"}
```

`fuzz/corpus/ingest_corpus/generic.csv` — detection needs a `#` line or a comma:
```
# frame=ecliptic
jd_tdb,x_au,y_au,z_au
2451545.0,0.1,0.2,0.3
```

`fuzz/corpus/ingest_corpus/unrecognized.txt`:
```
not any known format
```

Check these against the real parsers rather than trusting the shapes above — read `crates/pleiades-jpl/src/ingest/format/vector_table.rs` and `generic_csv.rs` for the exact column expectations and adjust the seeds so at least the vector-table and CSV seeds parse successfully. A seed that fails detection still exercises the detector, but a seed that reaches the normalizer is worth far more.

- [ ] **Step 5: Smoke-run**

```bash
cd fuzz && cargo +nightly-2026-07-01 fuzz run ingest_corpus -- -max_total_time=60
```
Expected: clean, consistent with the audit's finding. Any crash is a genuine new finding — reproduce, add a unit test in the relevant `ingest/format/*.rs`, fix, commit the reproducer with the fix.

- [ ] **Step 6: Minimize and commit**

```bash
cd fuzz && cargo +nightly-2026-07-01 fuzz cmin ingest_corpus
mkdir -p artifacts/ingest_corpus && touch artifacts/ingest_corpus/.gitkeep
```

```bash
git add fuzz/fuzz_targets/ingest_corpus.rs fuzz/corpus/ingest_corpus fuzz/artifacts/ingest_corpus/.gitkeep
git commit -m "test(fuzz): add ingest_corpus target for JPL text ingestion

Covers detection plus all three front-ends and the normalizer. The audit found
no panic paths here, so this locks that property in as a regression guard."
```

---

### Task 7: Fuzz CI workflow

A separate workflow, not an addition to `ci-nightly`: the existing nightly job has `timeout-minutes: 60` and already runs `test-full`, `package-check`, `benchmark`, `deny`, and `secrets`. Four targets at ~10 min each would overrun it and fail nightly on duration, tripping the pinned-issue alarm for a non-bug.

**Files:**
- Create: `.github/workflows/fuzz.yml`
- Create: `.github/fuzz-failure-issue.md`

**Interfaces:**
- Consumes: the `mise` setup, cache, and fail-loud issue pattern from `.github/workflows/nightly.yml`; the pinned nightly from Task 3.
- Produces: a cron-scheduled fuzz run with its own pinned tracking issue, independent of the nightly tier.

**Budgets — two distinct numbers, do not conflate:** ongoing cron budget is ~10 min per target (this task); the one-time merge gate is 30 min per target (Task 8).

- [ ] **Step 1: Read the existing nightly workflow to mirror its conventions**

```bash
cat .github/workflows/nightly.yml
cat .github/nightly-failure-issue.md
```
Match its `actions/checkout@v4`, `jdx/mise-action@v2`, `actions/cache@v4`, and `JasonEtco/create-an-issue@v2` versions exactly rather than the versions written below, in case they have moved.

- [ ] **Step 2: Create the issue template**

Create `.github/fuzz-failure-issue.md`:

```markdown
---
title: "Fuzz campaign failure"
labels: bug, fuzzing, security
---

The scheduled fuzz campaign found a crash, hang, or OOM.

Run: {{ env.RUN_URL }}

The crashing input is attached to that run as the `fuzz-artifacts` artifact.
To reproduce locally:

```bash
cargo +nightly-2026-07-01 fuzz run <target> fuzz/artifacts/<target>/<crash-file>
```

Per `docs/superpowers/specs/2026-07-18-devkit-phase3-fuzz-slice-design.md`,
a finding is fixed with a committed reproducer that runs as an ordinary unit
test in the blocking tier — not only as a corpus entry.

This issue is updated rather than duplicated on repeat failures, and closed
automatically when a campaign next passes.
```

- [ ] **Step 3: Create the workflow**

Create `.github/workflows/fuzz.yml`:

```yaml
name: Fuzz

# Separate from nightly.yml on purpose: nightly has a 60-minute budget already
# consumed by test-full, package-check, benchmark, deny and secrets. Keeping
# fuzz here lets its runtime scale independently, stops a wedged target from
# delaying or masking a test-tier regression, and lets per-target budgets grow
# without renegotiating nightly's envelope.
on:
  schedule:
    - cron: "0 3 * * *" # 03:00 UTC daily — offset from nightly's 06:00
  workflow_dispatch:

permissions:
  contents: read
  issues: write

jobs:
  fuzz:
    runs-on: ubuntu-latest
    timeout-minutes: 90 # ~10 min x 4 targets, plus toolchain install and build
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Set up mise
        uses: jdx/mise-action@v2
        with:
          install: false

      - name: Cache Rust toolchain, Cargo, and fuzz build artifacts
        uses: actions/cache@v4
        with:
          path: |
            ~/.rustup
            ~/.cargo
            ~/.local/share/mise
            fuzz/target
          key: ${{ runner.os }}-fuzz-${{ hashFiles('mise.toml', 'fuzz/Cargo.toml') }}
          restore-keys: |
            ${{ runner.os }}-fuzz-

      - name: Install mise-managed tools
        run: mise install

      - name: Run fuzz campaign
        run: |
          set -euo pipefail
          for target in spk_kernel compression_framing compression_payload ingest_corpus; do
            echo "── fuzzing $target ──"
            (cd fuzz && cargo +nightly-2026-07-01 fuzz run "$target" -- -max_total_time=600)
          done

      - name: Upload crash artifacts
        if: failure()
        uses: actions/upload-artifact@v4
        with:
          name: fuzz-artifacts
          path: fuzz/artifacts/
          if-no-files-found: ignore

      - name: Open/update tracking issue on failure
        if: failure()
        uses: JasonEtco/create-an-issue@v2
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          RUN_URL: ${{ github.server_url }}/${{ github.repository }}/actions/runs/${{ github.run_id }}
        with:
          filename: .github/fuzz-failure-issue.md
          update_existing: true
          search_existing: open
```

Copy the "Close tracking issue on success" step verbatim from the tail of `.github/workflows/nightly.yml`, changing only the issue-title search string so it closes the fuzz issue rather than the nightly one.

- [ ] **Step 4: Validate the workflow syntax**

```bash
mise run lint || true
python3 -c "import yaml,sys; yaml.safe_load(open('.github/workflows/fuzz.yml')); print('fuzz.yml parses')"
python3 -c "import yaml,sys; yaml.safe_load(open('.github/workflows/nightly.yml')); print('nightly.yml still parses')"
```
Expected: both print their confirmation line.

- [ ] **Step 5: Confirm nightly is untouched**

```bash
git diff --stat .github/workflows/nightly.yml mise.toml
grep -n "fuzz" .github/workflows/nightly.yml || echo "nightly correctly has no fuzz step"
```
Expected: no diff to `nightly.yml`, and `nightly correctly has no fuzz step`.

- [ ] **Step 6: Commit**

```bash
git add .github/workflows/fuzz.yml .github/fuzz-failure-issue.md
git commit -m "ci(fuzz): add standalone fuzz workflow with fail-loud issue

Separate from nightly.yml: nightly's 60-minute budget is already spent, and
fuzz runtime should scale independently of the test tier. ~10 min per target
on its own cron, crash inputs uploaded as artifacts, pinned tracking issue."
```

---

### Task 8: Merge-gate campaign

The one-time deep campaign that establishes the clean baseline. This is the task that satisfies the spec's merge criterion.

**Files:**
- Modify: `fuzz/corpus/*/` (accumulated, minimized)
- Modify: `fuzz/artifacts/*/` (any crash reproducers found)
- Create: `docs/superpowers/specs/notes/2026-07-18-fuzz-campaign-results.md` (the repo's established location for measured-evidence records — see the residuals and readiness notes already there)
- Possibly modify: production source, if the campaign finds anything

**Interfaces:**
- Consumes: all four targets from Tasks 4–6.
- Produces: recorded evidence of 30 min/target clean runs, a minimized committed corpus, and a unit-test reproducer for every defect found.

**Merge criterion (from the spec):** each target runs **30 minutes on the accumulated corpus with zero new crashes**; every defect found is fixed; every fix is pinned by a committed reproducer that runs as an ordinary test in the blocking tier.

- [ ] **Step 1: Run the full campaign**

Run each target for 30 minutes. These are long; run them one at a time and capture the output:

```bash
mkdir -p /tmp/claude-1000/-workspace/ac3c66ee-5c4e-4b60-91ef-a8a194ff6b11/scratchpad/fuzz-logs
cd fuzz
for target in spk_kernel compression_framing compression_payload ingest_corpus; do
  cargo +nightly-2026-07-01 fuzz run "$target" -- -max_total_time=1800 \
    2>&1 | tee "/tmp/claude-1000/-workspace/ac3c66ee-5c4e-4b60-91ef-a8a194ff6b11/scratchpad/fuzz-logs/$target.log"
done
```

Expected per target: a final libFuzzer summary line reporting the executions performed and `Done` with no `ERROR:` / `crash-` / `SUMMARY: libFuzzer: deadly signal` lines.

- [ ] **Step 2: Triage and fix every finding**

For each crash, hang, or OOM:

1. Reproduce it: `cd fuzz && cargo +nightly-2026-07-01 fuzz run <target> artifacts/<target>/<crash-file>`
2. Write a **failing unit test** in the crate that owns the parser, encoding the same input — co-located per AGENTS.md, following the patterns from Tasks 1 and 2. This is what puts the fix in the blocking tier.
3. Run it and confirm it fails for the expected reason.
4. Fix the defect using the same discipline as Tasks 1–2: structured `Err`, no magic constants that could reject valid data.
5. Confirm the test passes and the crash no longer reproduces.
6. Commit the fix, the test, and the crash artifact together.
7. Restart that target's 30-minute clock — a fix invalidates the clean run that preceded it.

If findings accumulate to the point where the branch stops being reviewable, invoke the spec's documented fallback: split the remaining fixes into a follow-up branch and land the harness first. That is a judgment call to raise with the maintainer, not to make silently.

- [ ] **Step 3: Minimize all corpora**

```bash
cd fuzz
for target in spk_kernel compression_framing compression_payload ingest_corpus; do
  cargo +nightly-2026-07-01 fuzz cmin "$target"
done
du -sh corpus/*
```
Expected: each corpus stays small (single-digit MB at most). If one has grown large, re-run `cmin` and confirm it is coverage-equivalent before committing.

- [ ] **Step 4: Record the results**

Create `docs/superpowers/specs/notes/2026-07-18-fuzz-campaign-results.md` with, for each target: the exact command, wall-clock duration, total executions, final coverage figure, and the crash count. Paste the final libFuzzer summary line for each. List every defect found with its fix commit. This is the evidence the spec's merge criterion requires — `verification-before-completion` means recording actual output, not asserting success.

- [ ] **Step 5: Run the full suite**

```bash
mise run ci
timeout 3600 mise run test-full
```
Expected: both PASS. Record the `mise run ci` wall-clock and confirm it has not increased relative to `main` — a Global Constraint.

- [ ] **Step 6: Commit**

```bash
git add fuzz/corpus fuzz/artifacts docs/superpowers/specs/notes/2026-07-18-fuzz-campaign-results.md
git commit -m "test(fuzz): commit minimized corpora and campaign results

30 minutes per target with zero crashes on the accumulated corpus, per the
slice's merge criterion. Corpora are committed so nightly coverage accumulates
rather than restarting cold."
```

---

### Task 9: Documentation

**Files:**
- Modify: `docs/threat-model.md:31` (boundary #1 row)
- Modify: `AGENTS.md` (tool list, and the tiering/security pointers)

**Interfaces:**
- Consumes: the shipped state of Tasks 1–8.
- Produces: documentation matching what actually shipped. No new documents — `docs/threat-model.md` and `AGENTS.md` are the single sources.

- [ ] **Step 1: Update the threat model**

In `docs/threat-model.md`, the boundary #1 row currently ends with `fuzzing planned (design Phase 3)`. Replace that clause with the shipped state:

```
fuzzed continuously by four cargo-fuzz targets (`spk_kernel`, `compression_framing`, `compression_payload`, `ingest_corpus`) on a daily cron; findings are fixed with regression tests in the blocking tier
```

Leave the rest of the row — the structured-errors rule and the checksum-pinned corpora — unchanged.

- [ ] **Step 2: Verify the claim is accurate**

```bash
ls fuzz/fuzz_targets/
grep -n "cron" .github/workflows/fuzz.yml
```
Expected: exactly the four named targets exist, and the workflow has a daily cron. The threat model must not claim coverage that does not exist — if a target was dropped during implementation, name only what shipped.

- [ ] **Step 3: Update AGENTS.md**

Add `cargo-fuzz` and the pinned nightly to the mise-installed tool list, matching what `mise.toml` now contains. In the "Security and reliability" section, note that the untrusted-byte parsers are fuzzed on a daily cron and that fuzz findings are fixed with blocking-tier regression tests. Add a line to the CI tiering description recording that fuzzing is a **third** tier — separate from blocking and nightly — with its own workflow and budget, so a future contributor does not try to fold it into `ci-nightly`.

Match the file's existing heading structure and terse style; read the surrounding sections before writing.

- [ ] **Step 4: Verify docs consistency**

```bash
mise run docs
mise run claims-audit
```
Expected: both PASS. `claims-audit` parses release claims — if the AGENTS.md or threat-model edits disturbed an anchor it reads, fix the anchor rather than loosening the audit.

- [ ] **Step 5: Commit**

```bash
git add docs/threat-model.md AGENTS.md
git commit -m "docs: record shipped fuzz coverage in threat model and AGENTS.md

Boundary #1 fuzzing moves from planned to shipped, naming the four targets.
AGENTS.md gains the cargo-fuzz and nightly pins and notes fuzzing as a third
CI tier distinct from blocking and nightly."
```

---

### Task 10: Branch completion

- [ ] **Step 1: Verify the full acceptance criteria**

Check each of the spec's eight acceptance criteria against actual evidence, not memory:

```bash
cd fuzz && cargo +nightly-2026-07-01 fuzz build 2>&1 | tail -3   # 1. all targets build
cd /workspace && mise run ci                                      # 5. blocking tier green and not slower
timeout 3600 mise run test-full                                   # 6. gates green
grep -n "fuzz" docs/threat-model.md AGENTS.md                     # 7, 8. docs updated
```

Criteria 2 (defects fixed with reproducers), 3 (30-min clean campaign), and 4 (workflow fails loud) are evidenced by the Task 8 results note, the committed reproducer tests, and `.github/workflows/fuzz.yml` respectively.

- [ ] **Step 2: Review the complete diff**

```bash
git diff main...HEAD --stat
git diff main...HEAD -- crates/
```
The `crates/` diff is the part that ships to consumers — it should be small and consist only of the hardening fixes plus their tests. If it contains anything else, that is scope creep to remove.

- [ ] **Step 3: Trigger the workflow once manually**

Once the branch is pushed, run the fuzz workflow via `workflow_dispatch` against it to confirm it executes end-to-end in CI — the local runs prove the targets work, not that the workflow does. Confirm it completes within its 90-minute timeout.

- [ ] **Step 4: Finish the branch**

Use the `superpowers:finishing-a-development-branch` skill. Per the maintainer's standing preference, open a PR and let it auto-merge once CI is green, then delete the feature branch.

---

## Self-Review

**Spec coverage** — every spec section maps to a task:

| Spec section | Task |
| --- | --- |
| Scope correction (SPK/DAF in scope) | 1, 4 |
| Crate layout (non-member) | 3 |
| Toolchain (dated nightly, cargo-fuzz) | 3 |
| Corpus and crash-artifact policy | 3 (verify), 4–6 (seed), 8 (minimize/commit) |
| Target 1 `spk_kernel` | 4 |
| Target 2 `compression_framing` | 5 |
| Target 3 `compression_payload` | 5 |
| Target 4 `ingest_corpus` | 6 |
| Oracle (no panic/UB/hang) | 4–6 (in each target's comment) |
| Fix: DAF walk (a, b, c) | 1 |
| Fix: segment count (d) | 2 |
| Non-regression requirement | 1 (Step 7), 8 (Step 5), 10 (Step 1) |
| CI wiring (separate workflow) | 7 |
| Budgets (10 min cron / 30 min merge) | 7 (Step header), 8 |
| Merge criterion | 8, 10 |
| Acceptance criteria 1–8 | 10 (Step 1) |
| Risks: toolchain drift | 3 (pinned, commented) |
| Risks: campaign finds more | 8 (Step 2, incl. documented fallback) |
| Risks: corpus growth | 8 (Step 3) |

**Type consistency** — verified against the source, not assumed:
- `record_byte` changes from `fn(usize) -> usize` to `fn(usize) -> Result<usize, SpkError>`; both call sites in `daf.rs` use `?`. `addr_to_byte` is untouched.
- `SpkErrorKind::{Truncated, BadHeader}` and `CompressionErrorKind::InvalidFormat` all exist (`spk/mod.rs:33-51`, and the compression error enum used throughout `codec.rs`).
- `SpkBackend`, `SpkBackendBuilder`, `SpkError`, `SpkErrorKind` are public at the `pleiades-jpl` root (`lib.rs:58-61`) — confirmed by reading the re-export list.
- `ARTIFACT_VERSION` is `pub` (`lib.rs:68`); `ARTIFACT_MAGIC` (`lib.rs:70`) and `fnv1a64` (`codec.rs:128`) are `pub(crate)`, which is why Task 5 duplicates them in the harness and Step 4 pins the copy with a test.
- `ExpectedProfile` derives `Default` (`profile.rs:26`), so `ExpectedProfile::default()` in Task 6 is valid.
- `Cursor::remaining() -> &'a [u8]` (`codec.rs:57`) supports the `.len()` call in Task 2's bound.
- `MIN_ENCODED_SEGMENT_BYTES = 20` is derived in Task 2 from `decode_segment` + `decode_instant`, not guessed.

**Known open items deliberately left to the implementer**, each with an explicit decision rule in-step rather than a placeholder: the exact nightly-pin mechanism (mise rust backend vs rustup, Task 3 Step 1), the compression seed-artifact builder name (Task 5 Step 5 — reuse whatever the existing round-trip tests use), and the ingest seed shapes (Task 6 Step 4 — validate against the real parsers).
