# Fuzz Campaign Results — Merge Gate (Task 8)

Merge criterion (devkit-phase3-fuzz spec): each target runs 30 minutes on the accumulated
corpus with zero new crashes. All four targets ran; all four finished clean.

## Command form

```
ASAN_OPTIONS=detect_leaks=0 mise exec -- cargo "+$FUZZ_NIGHTLY" fuzz run \
  <target> <scratch-corpus-dir> /workspace/fuzz/corpus/<target> -- -max_total_time=1800
```

`$FUZZ_NIGHTLY` = `nightly-2026-07-01` (`mise.toml`).

Two deliberate deviations from the brief:

- **Scratch corpus dir first.** libFuzzer writes discoveries to the first corpus directory
  given on the command line, so putting a scratch directory first and the committed
  `fuzz/corpus/<target>` second makes the committed corpus a read-only seed source. The
  brief's Steps 3 and 6 said to minimize and commit the accumulated corpus; the maintainer
  decided instead that only the small curated seeds already in `fuzz/corpus/` (11 files) stay
  committed, and the accumulated corpus lives in the `fuzz.yml` workflow's GitHub Actions
  cache (`corpus-fuzz-*` key). This run did not touch `fuzz/corpus/` or `fuzz/artifacts/`.
- **`ASAN_OPTIONS=detect_leaks=0` is local-sandbox-only, never committed.** This dev
  container has no ptrace capability (`CapEff=0`, `ptrace_scope=2`), so LeakSanitizer aborts
  during its teardown scan and drops a bogus artifact named
  `crash-da39a3ee5e6b4b0d3255bfef95601890afd80709` — the SHA-1 of the empty string, i.e.
  content-independent, not a triggering input. GitHub Actions runners permit ptrace, so leak
  detection stays ON in `.github/workflows/fuzz.yml` and in `mise.toml`'s `fuzz`/`fuzz-target`
  tasks (neither sets `ASAN_OPTIONS`). Do not copy this flag into CI.

Campaign completed 2026-07-18T12:32:48Z. Raw logs:
`/tmp/claude-1000/-workspace/6664a6a0-3114-4703-b541-e7241273bb41/scratchpad/fuzz-logs/{spk_kernel,compression_framing,compression_payload,ingest_corpus}.log`
(`summary.txt` alongside has the per-target rc/wall lines).

## Results

| Target | Executions | Wall | Final libFuzzer summary | Cron budget |
|---|---|---|---|---|
| `spk_kernel` | 48,554,574 | 1802s | `#48554574 DONE cov: 344 ft: 1015 corp: 128/244Kb lim: 5120 exec/s: 26959 rss: 497Mb` | 600s/day vs. 1800s here |
| `compression_framing` | 333,803,323 | 1801s | `#333803323 DONE cov: 770 ft: 1375 corp: 133/18Kb lim: 4096 exec/s: 185343 rss: 519Mb` | 600s/day vs. 1800s here |
| `compression_payload` | 48,207,219 | 1801s | `#48207219 DONE cov: 1262 ft: 3622 corp: 658/88Kb lim: 4096 exec/s: 26766 rss: 208Mb` | 600s/day vs. 1800s here |
| `ingest_corpus` | 18,403,148 | 1802s | `#18403148 DONE cov: 1216 ft: 4754 corp: 1376/367Kb lim: 4096 exec/s: 10218 rss: 584Mb` | 600s/day vs. 1800s here |

All four exit rc=0. Total ~449M executions, 4 x 30 minutes.

### Reading the per-target disparities

The executions and coverage figures differ by an order of magnitude between targets. Both
disparities are by design, and neither indicates a broken harness:

- **`compression_framing` does 333M executions for only 770 edges** because it fuzzes the
  framing exactly as shipped, and the unkeyed FNV-1a checksum rejects almost every mutated
  input before any parsing happens. That is the target's purpose. `compression_payload` exists
  precisely so the mutator's bytes reach the codec: the harness recomputes the checksum, which
  is why it gets 1262 edges from 7x fewer executions.
- **`spk_kernel` saturates earliest and covers the least.** Its last new edge appeared at
  execution 35.6M of 48.5M (73% through the run), versus 81% for `compression_framing`, 95%
  for `ingest_corpus`, and 98% for `compression_payload`; it gained only 3 edges (341 -> 344)
  across the whole campaign and its corpus holds 128 entries. `DafFile::parse` gates on an
  exact 8-byte magic, an enum-valued `LOCFMT`, and `ND == 2 && NI == 6` before reaching any
  interesting logic, so the mutator plateaus once it learns to preserve those gates.

The merge criterion (30 minutes, zero new crashes) is met for all four. But `spk_kernel`'s
clean result rests on a narrower exploration than the other three, so it should not be read as
equally exhaustive evidence. If that target's depth matters later, the lever is a richer seed
corpus — structurally valid DAFs varying `ND`/`NI`, record counts, and endianness — rather
than a longer budget, which the plateau suggests would buy little.

Every figure above was read directly from the DONE line of each log and cross-checked
against `summary.txt`; all match the campaign's reported evidence exactly.

Crash-indicator grep across all four logs:

```
$ grep -n "SUMMARY: libFuzzer: deadly signal\|ERROR: AddressSanitizer\|panicked at\|detected memory leaks" \
  spk_kernel.log compression_framing.log compression_payload.log ingest_corpus.log
(no output)
```

`find /workspace/fuzz/artifacts -type f`: only the four `.gitkeep` placeholders, no crash
reproducers.

## Defects found: none

Expected result, not a gap in coverage. The four known defects this slice set out to guard
against were already found and fixed in Tasks 1 and 2, each pinned by a unit test in the
blocking tier:

- Unbounded DAF summary-record chain walk (self-referential `NEXT` pointer) — fixed in
  `3808e9640` (`fix(spk): bound DAF record chain and check offset arithmetic`).
- `usize` record-number overflow from a saturating f64-to-usize cast — same commit,
  `3808e9640`.
- `ReadAt::read_at`'s `offset + len` overflow — same commit, `3808e9640`.
- Unbounded `Vec::with_capacity` driven by a declared segment count — fixed in `c55165b29`
  (`fix(compression): bound declared segment count before allocating`).

This campaign establishes the clean baseline the merge criterion requires against those
fixes; it is not expected to (and did not) find anything new. No new reproducers to record.

That said, "zero defects" is evidence of a clean baseline, not proof of absence — and it is
not uniformly strong across the four targets. See the per-target disparities above:
`compression_payload` and `ingest_corpus` were still finding new coverage at 95-98% through
their runs, while `spk_kernel` had effectively saturated by 73%.

## Full suite (Step 5)

Run by the controller after the campaign finished, so nothing contended with it for CPU.

The first run **failed**, and the failure was a real regression this branch had introduced —
not a flake, and not something the fuzzing found:

```
[audit] - /workspace/mise.toml [tool-manifest.rust-entry-invalid]:
           mise.toml rust tool entry must use an inline table
[audit] - /workspace/mise.toml [tool-manifest.rust-components-missing]:
           mise.toml rust tool entry should include both rustfmt and clippy components
CI_EXIT=1
TESTFULL_EXIT=100   # 2752 tests run: 5 failed
```

`crates/pleiades-validate/src/release/workspace_audit.rs` hand-parses `mise.toml` and requires
the `[tools]` `rust` value to start with `{` and end with `}`. An earlier commit on this branch
had changed it to an array (`rust = [{ ... }, "{{env.FUZZ_NIGHTLY}}"]`) so that `mise install`
would provision both toolchains. That array starts with `[`, so the rule fired; the parser then
skips the entry, so the components rule fired too. Both failed `[tasks.audit]`, a dependency of
`ci`, and cascaded into five `pleiades-cli` release/workspace-provenance test failures.

Fixed in `774ef3d98` by reverting `mise.toml`'s `[tools] rust` to be byte-identical to `main`
and pinning the dated nightly in `fuzz/rust-toolchain.toml` instead — rustup's own mechanism,
and `fuzz/` is already a standalone workspace root. No production code changed. Tradeoff:
`mise install` no longer pre-provisions the nightly; rustup installs it lazily on first use,
still automatically. The date now lives in two places (`mise.toml`'s `FUZZ_NIGHTLY`, which the
mise tasks consume, and `fuzz/rust-toolchain.toml`, which rustup reads directly) with a comment
on each pointing at the other — mise's templating cannot reach a file rustup parses itself.

Re-run after the fix, both green:

```
mise run ci         → CI_EXIT=0        real 2m35.132s
                       [test] Summary [91.368s] 1595 tests run: 1595 passed (2 slow), 84 skipped
mise run test-full  → TESTFULL_EXIT=0  real 55m35.062s
                       Summary [3314.521s] 2752 tests run: 2752 passed (193 slow), 0 skipped
```

Blocking-tier wall-clock is 2m35s, consistent across runs and not increased by this slice —
fuzzing is in neither `ci` nor `ci-nightly`, and `fuzz/` is a non-member crate, so the blocking
tier never builds it.

Worth recording as a process lesson: the array pin had been verified functionally (mise really
did install both toolchains, and a cold-start teardown test confirmed it) but nobody ran
`mise run ci`. When a repo's own policy linter constrains the *shape* of a config file, "the
tool accepts it" is not sufficient evidence.
