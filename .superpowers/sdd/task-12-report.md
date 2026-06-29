# Task 12 Report: Public docs, rustdoc example, and truthful release claims

## 1. Doctest placement and verification

The runnable example was placed in the **crate-level `//!` module rustdoc** of
`crates/pleiades-eclipse/src/lib.rs` as a ` ```rust ` fenced block (public
documented surface; rustdoc collects it).

Content:
```rust
use pleiades_data::packaged_backend;
use pleiades_eclipse::{EclipseEngine, EclipseFilter};
use pleiades_types::{Instant, JulianDay, TimeScale};

let engine = EclipseEngine::new(packaged_backend());
let after = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);
let next = engine.next_eclipse(after, EclipseFilter::All).unwrap();
assert!(next.is_some());
```

`cargo test -p pleiades-eclipse --doc` result:
```
running 1 test
test crates/pleiades-eclipse/src/lib.rs - (line 24) ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; finished in 13.97s
```

### Bug fixed: boundary overstep in `next_eclipse`

`next_eclipse` was passing `WINDOW_END_JD = 2488069.5` as the scan end to
`find_syzygies`. `find_one` in `syzygy.rs` steps one `STEP_DAYS = 0.5` past
the requested end for sign-change detection, so the last query landed at JD
2488070.0 — past the packaged ephemeris coverage — causing a `Backend(...)` error.

Fix: in `engine.rs`, `next_eclipse` now uses `WINDOW_END_JD - 0.5` as the end
instant, ensuring the scanner's extra step lands exactly at `WINDOW_END_JD`
(within coverage). The last corpus eclipse is ~93 days before `WINDOW_END_JD`,
so no eclipses are missed.

## 2. Doc surfaces updated

### `crates/pleiades-eclipse/src/lib.rs` (module doc)

Replaced the two-line stub with a full `//!` block covering:
- Window: 1900-01-01 (JD 2 415 020.5 TDB) through 2100-01-01 (JD 2 488 069.5 TDB),
  packaged-ephemeris bound; 4 NASA-canon eclipses in mid/late 2100 excluded.
- Coverage: global / geocentric only; no per-observer local circumstances.
- Outputs: type, greatest-eclipse time, magnitude, gamma, Saros series, eclipsed
  longitude (apparent tropical ecliptic of date; no ayanamsa), solar greatest-eclipse
  location (lunar: none).
- Validation: fail-closed `validate-eclipses` gate; ~908 passed, 1 allowlisted.

### `crates/pleiades-eclipse/README.md`

Replaced the two-line stub with a full description: "What it provides" (all seven
output fields), "Window and data-bound" (precise 1900-01-01…2100-01-01, 4 eclipses
excluded), "Validation" table (five tolerances, ~908 + 1 allowlisted, independent
reference), and "Quick start" code block (same content as the module doctest).

### `README.md` (root)

Added one bullet to the "Current state" enumeration after the contributor-CLI
bullet:
> global/geocentric solar and lunar eclipse data (type, greatest-eclipse time,
> magnitude, gamma, Saros series, eclipsed longitude, and solar greatest-eclipse
> location) for 1900-01-01 … 2100-01-01 via `pleiades-eclipse`, validated
> exhaustively against NASA's Five Millennium Canon by the fail-closed
> `validate-eclipses` gate; local (per-observer) circumstances are not provided.

## 3. Compatibility / gate-set listing change

`validate-eclipses` is already wired into `run_all_numeric_gates()` → `release-smoke`
(prior tasks). It was missing from the **help text** in
`crates/pleiades-validate/src/render/cli.rs`, alongside `validate-houses` and
`validate-ayanamsa`.

Added two lines to the help text string (after `ayanamsa-gate` line):
```
  validate-eclipses         Run the fail-closed eclipse gate (NASA Five Millennium
                            Canon reference, ≤60 s / ≤0.01 / exact type & Saros)
                            over the committed eclipse corpus
  eclipses-gate             Alias for validate-eclipses
```

No change to `RELEASE_CHECKLIST_REPOSITORY_MANAGED_RELEASE_GATES` (10 items) —
`validate-eclipses` is already covered by the `release-smoke` step in that list.
The "Repository-managed release gates: 10 items" test at line 313 of
`tests/release_checklist.rs` is unaffected.

## 4. Spec as-built notes

Added an "As-built notes" subsection to
`docs/superpowers/specs/2026-06-29-eclipse-subsystem-design.md` recording four
maintainer-ratified deviations:

(a) Window data-bound to 2100-01-01 (4 late-2100 eclipses excluded).
(b) Lunar shadow enlargement: 1.01 applied to (π_moon + π_sun), not "1.02 on
    Earth's radius" as the plan loosely described.
(c) Apparent eclipsed longitude via single light-time/aberration application
    (≤ 1.0″ tolerance met; no further iteration needed).
(d) One allowlisted knife-edge eclipse: 1948-05-09, Saros 137.

## 5. Full workspace check results

```
cargo test -p pleiades-eclipse --doc:  1 passed, 0 failed
cargo test --workspace:               2020 passed, 0 failed (183 ignored)
cargo fmt --all -- --check:           CLEAN (no output)
cargo clippy --workspace --all-targets: CLEAN (no warnings)
```

## Files changed

- `crates/pleiades-eclipse/src/lib.rs` — module doc + doctest; engine boundary fix
- `crates/pleiades-eclipse/src/engine.rs` — `next_eclipse` WINDOW_END_JD - 0.5 fix
- `crates/pleiades-eclipse/README.md` — full accurate description
- `README.md` — eclipse "Current state" bullet
- `crates/pleiades-validate/src/render/cli.rs` — `validate-eclipses`/`eclipses-gate` in help text
- `docs/superpowers/specs/2026-06-29-eclipse-subsystem-design.md` — "As-built notes" subsection

## FIX (centralize boundary clamp)

### Change

**`crates/pleiades-eclipse/src/syzygy.rs`**: Made `STEP_DAYS` `pub(crate)` so
`engine.rs` can reference it directly instead of hardcoding `0.5`.

**`crates/pleiades-eclipse/src/engine.rs`** (`EclipseEngine::eclipses_in_range`):
After the existing `check_window` guards (kept as-is — caller bounds must still be
in-window), two clamps are applied before calling `find_syzygies`:

```rust
let scan_start = start_jd.max(WINDOW_START_JD + STEP_DAYS);
let scan_end   = end_jd.min(WINDOW_END_JD   - STEP_DAYS);
```

`next_eclipse` was simplified: it now passes `WINDOW_END_JD` as its `end`
argument (the old manual `- 0.5` was removed) because `eclipses_in_range`
applies the clamp centrally for all callers.

### Why it protects all callers

Two independent backend-overstep hazards existed at the window boundaries:

1. **END**: `find_one` loops `while jd <= end_jd + STEP_DAYS`, so the last
   sample is at `end_jd + 0.5`. Passing `WINDOW_END_JD` without clamping caused
   a query at `WINDOW_END_JD + 0.5 = 2488070.0`, past the packaged backend's
   coverage. Fix: `scan_end = end_jd.min(WINDOW_END_JD - STEP_DAYS)`.

2. **START**: `sample_sun_moon` uses light-time-retarded positions. For the Sun
   (~1 AU), the retarded epoch is ~499 s = 0.006 d before the nominal JD. When
   `scan_start = WINDOW_START_JD`, the retarded query fell before the backend's
   first segment. Fix: `scan_start = start_jd.max(WINDOW_START_JD + STEP_DAYS)`
   (STEP_DAYS = 0.5 d >> 0.006 d light-time).

Both clamps are safe: no corpus eclipse falls within 0.5 d of either window
boundary (earliest corpus eclipse is at JD 2415168, ~147 days after WINDOW_START_JD;
latest is ~93 days before WINDOW_END_JD).

### New tests

**`crates/pleiades-eclipse/tests/known_eclipses.rs`** (3 new tests):
- `next_eclipse_near_window_end_does_not_error`: starts 365 d before WINDOW_END_JD
- `previous_eclipse_at_window_end_does_not_error`: `before = WINDOW_END_JD` (scans
  full window; also exercises start-boundary clamp)
- `eclipses_in_range_ending_at_window_end_does_not_error`: start 365 d before,
  `end = WINDOW_END_JD` (direct end-boundary path)

**`crates/pleiades-validate/src/tests/validate_gates.rs`** (1 new test):
- `eclipses_listing_with_end_at_window_boundary_does_not_error`: calls
  `render_cli(&["eclipses", "--start", "2488000.0"])` (no `--end`, defaults to
  WINDOW_END_JD) and asserts `Ok` — exercises the `render_eclipses_listing`
  default-end path.

### Test commands and pass output

```
cargo test -p pleiades-eclipse
# result: 5/5 known_eclipses tests ok, 16/16 unit tests ok, 1/1 doctest ok

cargo test -p pleiades-validate eclipses_listing_with_end_at_window_boundary
# result: 1 passed; 0 failed

cargo test -p pleiades-validate
# result: 476 passed; 0 failed; 183 ignored

cargo fmt -p pleiades-eclipse -p pleiades-validate -- --check  # CLEAN
cargo clippy -p pleiades-eclipse -p pleiades-validate --all-targets  # CLEAN (no warnings)
```

### Gate and doctest unaffected

`validate_eclipse_corpus` uses `eclipses_in_range(exp_jd - 1.0, exp_jd + 1.0)` for
each corpus row (narrow windows well within the window); the clamps have no effect
on these calls. The `lib.rs` doctest (`next_eclipse` from JD 2451545.0) is equally
unaffected. Both continue to pass.
