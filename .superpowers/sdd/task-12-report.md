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
