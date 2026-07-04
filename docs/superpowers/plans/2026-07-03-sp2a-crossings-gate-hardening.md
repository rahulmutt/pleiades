# SP-2a-FU · `validate-crossings` Gate Hardening — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn `validate-crossings` into a two-tier gate (tight self-consistency + arcsecond SE parity), expand the corpus to geo+helio Mercury–Pluto, and close spec §7 (fnv1a64 checksum-drift) — with no engine algorithm change.

**Architecture:** Add one additive public method `CrossingEngine::longitude_at`. Rewrite `crates/pleiades-validate/src/crossings_validation.rs` so each corpus row is checked by Tier 1 (recomputed crossing time vs a committed `pleiades_jd_tdb` golden column, sub-second) and Tier 2 (engine longitude evaluated at the SE crossing time vs the target, arcseconds). Extend `tools/se-crossings-reference` to emit the full planet set, add a `crossings-golden --check/--regenerate` CLI command that writes the golden column, and verify the corpus with `fnv1a64` mirroring `validate-lilith`.

**Tech Stack:** Rust (workspace crates `pleiades-events`, `pleiades-validate`, `pleiades-core`), the offline `tools/se-crossings-reference` Swiss-Ephemeris FFI binary (Moshier — no external data files), `pleiades_apparent::fnv1a64`.

## Global Constraints

- **No engine algorithm/root-finder/convention change.** Only additive API + gate/corpus/docs.
- **Pure-Rust workspace.** No new crate dependency; `fnv1a64` comes from `pleiades_apparent` (already a `pleiades-validate` dep, imported by `lilith_validation.rs`).
- **Fail closed always.** Every gate path returns `Err` immediately; never accept NaN/placeholder.
- **Neighboring gates stay green and untouched:** `validate-eclipses`, `validate-angles`, and every other entry in `run_all_numeric_gates` must pass unchanged.
- **TDB time base; hard 1900–2100 window** (`WINDOW_START_JD..=WINDOW_END_JD` in `pleiades-events`).
- **Self-consistency ceiling:** `SELF_CONSISTENCY_TOL_S = 1.0` s (a small factor above the root-finder's `REFINE_TOLERANCE_DAYS = 0.5/86400` = 0.5 s bisection tolerance).
- **Arcsecond ceilings are measured, not guessed:** set each per-body Tier-2 ceiling to ≈1.4× the measured group max (precedent: `validate-lilith` accepts an SE-vs-ours residual of ~306″).
- **Coverage-boundary honesty:** a body that cannot meet a defensible arcsec ceiling (candidate: Pluto, bounded by its backend fallback) gets a documented boundary ceiling — never silently excluded.
- **Versioning:** new public `longitude_at` → compatibility profile `0.7.5 → 0.7.6`; API-stability profile stays `0.2.1` (purely additive).

---

## Task 1: `CrossingEngine::longitude_at` public method

**Files:**
- Modify: `crates/pleiades-events/src/crossings.rs`

**Interfaces:**
- Consumes: private `CrossingEngine::longitude_deg(&self, &CelestialBody, CrossingFrame, f64) -> Result<f64, EventError>`, `check_window`, `EventError::{OutOfWindow, UnsupportedFrame}` (all already in `crossings.rs`).
- Produces: `pub fn longitude_at(&self, body: CelestialBody, frame: CrossingFrame, instant: Instant) -> Result<Longitude, EventError>` — the Tier-2 evaluator used by Task 4.

- [ ] **Step 1: Write the failing tests.** Append to the `#[cfg(test)] mod tests` block at the bottom of `crates/pleiades-events/src/crossings.rs` (the `tdb`, `wrap180`, and `LinearSunMoon` helpers are already in scope there):

```rust
    #[test]
    fn longitude_at_matches_crossing_target() {
        let engine = CrossingEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
        let after = tdb(2_451_545.0);
        let target = Longitude::from_degrees(100.0);
        let c = engine.next_sun_crossing(target, after).unwrap().unwrap();
        // At the crossing instant the engine's longitude must equal the target.
        let lon = engine
            .longitude_at(
                CelestialBody::Sun,
                CrossingFrame::GeocentricApparentOfDate,
                c.instant,
            )
            .unwrap();
        assert!(
            wrap180(lon.degrees() - 100.0).abs() < 1e-3,
            "lon at crossing {}",
            lon.degrees()
        );
    }

    #[test]
    fn longitude_at_fails_closed() {
        let engine = CrossingEngine::new(LinearSunMoon::new_moon_at(2_451_550.0));
        // Heliocentric Sun is undefined.
        let err = engine
            .longitude_at(
                CelestialBody::Sun,
                CrossingFrame::Heliocentric,
                tdb(2_451_545.0),
            )
            .unwrap_err();
        assert!(matches!(err, EventError::UnsupportedFrame { .. }));
        // Out of the packaged window.
        let err = engine
            .longitude_at(
                CelestialBody::Sun,
                CrossingFrame::GeocentricApparentOfDate,
                tdb(2_000_000.0),
            )
            .unwrap_err();
        assert!(matches!(err, EventError::OutOfWindow { .. }));
    }
```

- [ ] **Step 2: Run the tests to verify they fail.**

Run: `cargo test -p pleiades-events longitude_at`
Expected: FAIL — `no method named longitude_at found`.

- [ ] **Step 3: Implement the method.** Add it inside `impl<B: EphemerisBackend> CrossingEngine<B>` in `crates/pleiades-events/src/crossings.rs`, immediately after `next_moon_crossing` (before the closing `}` of the impl block). The doctest is required — the crate is `#![deny(missing_docs)]`:

```rust
    /// Ecliptic longitude of `body` in `frame` at `instant` (TDB).
    ///
    /// Geocentric apparent tropical of date for
    /// [`CrossingFrame::GeocentricApparentOfDate`]; heliocentric of date for
    /// [`CrossingFrame::Heliocentric`]. Fails closed outside the packaged
    /// 1900–2100 window and for heliocentric Sun/Moon, matching the crossing
    /// entry points. This is the evaluator the `validate-crossings` parity tier
    /// uses to compare the engine's longitude against a reference crossing time.
    ///
    /// ```
    /// use pleiades_data::packaged_backend;
    /// use pleiades_events::{CrossingEngine, CrossingFrame};
    /// use pleiades_types::{CelestialBody, Instant, JulianDay, TimeScale};
    ///
    /// let engine = CrossingEngine::new(packaged_backend());
    /// let t = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);
    /// let lon = engine
    ///     .longitude_at(CelestialBody::Sun, CrossingFrame::GeocentricApparentOfDate, t)
    ///     .unwrap();
    /// assert!((0.0..360.0).contains(&lon.degrees()));
    /// ```
    pub fn longitude_at(
        &self,
        body: CelestialBody,
        frame: CrossingFrame,
        instant: Instant,
    ) -> Result<Longitude, EventError> {
        let jd = instant.julian_day.days();
        self.check_window(jd)?;
        if matches!(frame, CrossingFrame::Heliocentric)
            && matches!(body, CelestialBody::Sun | CelestialBody::Moon)
        {
            return Err(EventError::UnsupportedFrame {
                detail: format!("heliocentric longitude is undefined for {:?}", body),
            });
        }
        let deg = self.longitude_deg(&body, frame, jd)?;
        Ok(Longitude::from_degrees(deg))
    }
```

- [ ] **Step 4: Run the tests and the doctest.**

Run: `cargo test -p pleiades-events`
Expected: PASS (unit tests + the `longitude_at` doctest).

- [ ] **Step 5: Commit.**

```bash
git add crates/pleiades-events/src/crossings.rs
git commit -m "feat(events): CrossingEngine::longitude_at — longitude of a body in a frame at an instant"
```

---

## Task 2: Extend the SE reference tool to the full planet set

**Files:**
- Modify: `tools/se-crossings-reference/src/main.rs`

**Interfaces:**
- Consumes: existing `geo_planet_cross_tdb(ipl, target_deg, start_tdb) -> f64`, `helio_cross_tdb(ipl, target_deg, start_tdb) -> f64`, `emit(frame, body, target, start_tdb, crossing_tdb)` (all already in `main.rs`).
- Produces: a regenerated 6-column SE reference (`frame,body,target_longitude_deg,start_jd_tdb,direction,crossing_jd_tdb`) covering geo + helio Mercury–Pluto, consumed by Task 4. This tool is out-of-workspace; it is not part of `cargo test --workspace`.

- [ ] **Step 1: Add the missing SE body-number constants.** In `tools/se-crossings-reference/src/main.rs`, next to the existing `SE_MARS`/`SE_JUPITER`/`SE_SATURN` constants (around line 33), add:

```rust
const SE_MERCURY: c_int = 2;
const SE_VENUS: c_int = 3;
const SE_URANUS: c_int = 7;
const SE_NEPTUNE: c_int = 8;
const SE_PLUTO: c_int = 9;
```

- [ ] **Step 2: Emit geocentric crossings for every planet.** In `fn main`, replace the single-body geo-Mars block (the `mars_target` / `mars_starts` loop that emits `"geo", "Mars"`) with: (a) the retained Mars retrograde triple-crossing block exactly as-is, plus (b) a full-planet geo loop. Add after the Mars block:

```rust
    // --- geo planets Mercury–Pluto: cardinal-ish targets, in-window starts. ---
    // SE has no geocentric planet-crossing function, so each is bisected on
    // swe_calc geocentric longitude (see geo_planet_cross_tdb). Starts are chosen
    // to sit comfortably inside 1900–2100 so the forward crossing is in-window.
    let geo_planets: [(c_int, &str); 7] = [
        (SE_MERCURY, "Mercury"),
        (SE_VENUS, "Venus"),
        (SE_JUPITER, "Jupiter"),
        (SE_SATURN, "Saturn"),
        (SE_URANUS, "Uranus"),
        (SE_NEPTUNE, "Neptune"),
        (SE_PLUTO, "Pluto"),
    ];
    let geo_planet_targets = [0.0_f64, 120.0, 240.0];
    let geo_planet_start = 2_440_000.5_f64; // ~1968; slow outer planets still cross within window
    for &(ipl, name) in &geo_planets {
        for &t in &geo_planet_targets {
            let c = geo_planet_cross_tdb(ipl, t, geo_planet_start);
            emit("geo", name, t, geo_planet_start, c);
        }
    }
```

- [ ] **Step 3: Emit heliocentric crossings for every planet.** Replace the existing helio block (the `[(SE_JUPITER, "Jupiter"), (SE_SATURN, "Saturn")]` loop) so it iterates the full planet set:

```rust
    // --- helio planets Mercury–Pluto via swe_helio_cross. ---
    let helio_targets = [0.0_f64, 180.0];
    let helio_starts = [2_430_000.5_f64, 2_460_000.5]; // ~1941 / ~2023
    let helio_planets: [(c_int, &str); 8] = [
        (SE_MERCURY, "Mercury"),
        (SE_VENUS, "Venus"),
        (SE_MARS, "Mars"),
        (SE_JUPITER, "Jupiter"),
        (SE_SATURN, "Saturn"),
        (SE_URANUS, "Uranus"),
        (SE_NEPTUNE, "Neptune"),
        (SE_PLUTO, "Pluto"),
    ];
    for &(ipl, name) in &helio_planets {
        for &start in &helio_starts {
            for &t in &helio_targets {
                let c = helio_cross_tdb(ipl, t, start);
                emit("helio", name, t, start, c);
            }
        }
    }
```

Remove the now-unused `let _ = SE_SUN; let _ = SE_MOON;` suppressors only if they cause an `unused` warning; leave the `# ...` header `println!`s but update the `# geo Mars: ...` comment line to `# geo planets: bisection on swe_calc geocentric longitude (no SE geo planet-cross fn); Mars block retains the retrograde triple-crossing.`

- [ ] **Step 4: Build and run the tool; verify it emits the full set without panicking.** The tool's `emit` internally asserts every row is forward and in-window, so a clean run is the test.

Run:
```bash
cd tools/se-crossings-reference && cargo run --release 2>/dev/null | grep -E '^(geo|helio),' | awk -F, '{print $1","$2}' | sort | uniq -c
```
Expected: rows for `geo,Sun`, `geo,Moon`, `geo,Mercury..Pluto`, `geo,Mars` (incl. the 3 retrograde rows), and `helio,Mercury..Pluto`; process exits 0 with no panic. Note the total data-row count (used in Task 4).

- [ ] **Step 5: Commit.**

```bash
git add tools/se-crossings-reference/src/main.rs
git commit -m "test(events): extend SE crossings reference to geo+helio Mercury-Pluto"
```

---

## Task 3: `crossings-golden` CLI command (golden-column regeneration)

**Files:**
- Modify: `crates/pleiades-validate/src/render/cli.rs`
- Test: `crates/pleiades-validate/src/render/cli.rs` (inline `#[cfg(test)]`)

**Interfaces:**
- Consumes: `pleiades_events::{CrossingEngine, CrossingFrame}`, `pleiades_data::packaged_backend`, `CrossingEngine::next_longitude_crossing`, `Task 1`'s API is not needed here.
- Produces: `pub(crate) fn append_golden_column(csv_6col: &str) -> Result<String, String>` — takes a 6-column SE CSV, returns a 7-column CSV with a `pleiades_jd_tdb` column and an updated header; plus a `crossings-golden` CLI subcommand (`--check` / `--regenerate` / `--out FILE`). Used by Task 4 to produce the committed corpus.

- [ ] **Step 1: Write the failing test.** Add to the `#[cfg(test)] mod tests` block in `crates/pleiades-validate/src/render/cli.rs`:

```rust
    #[test]
    fn append_golden_column_adds_pleiades_time() {
        // One in-window geo Sun fixture; the SE crossing_jd column value is
        // irrelevant to the golden (the golden is the engine's own recompute).
        let csv = "\
# comment line
frame,body,target_longitude_deg,start_jd_tdb,direction,crossing_jd_tdb
geo,Sun,0.000000,2416000.500000,fwd,2416195.301931810
";
        let out = super::append_golden_column(csv).expect("golden append");
        let header = out
            .lines()
            .find(|l| l.starts_with("frame,"))
            .expect("header");
        assert!(header.trim_end().ends_with(",pleiades_jd_tdb"), "header {header}");
        let row = out
            .lines()
            .find(|l| l.starts_with("geo,Sun,"))
            .expect("row");
        let fields: Vec<&str> = row.split(',').collect();
        assert_eq!(fields.len(), 7, "row should have 7 fields: {row}");
        let golden: f64 = fields[6].parse().expect("golden jd parses");
        // The engine's Sun crossing of 0° after 2416000.5 lands near the SE value.
        assert!((golden - 2_416_195.3).abs() < 1.0, "golden {golden}");
    }
```

- [ ] **Step 2: Run the test to verify it fails.**

Run: `cargo test -p pleiades-validate append_golden_column`
Expected: FAIL — `cannot find function append_golden_column`.

- [ ] **Step 3: Implement `append_golden_column`.** Add near the top of `crates/pleiades-validate/src/render/cli.rs` (module-level, after the existing `use`/helper section):

```rust
/// Recompute each SE fixture's crossing with the packaged engine and append the
/// result as a `pleiades_jd_tdb` golden column. Input is the 6-column SE CSV;
/// output is the 7-column corpus. Comment (`#`) lines pass through unchanged; the
/// `frame,` header gains a trailing `,pleiades_jd_tdb`.
pub(crate) fn append_golden_column(csv_6col: &str) -> Result<String, String> {
    use pleiades_events::{CrossingEngine, CrossingFrame};
    use pleiades_types::{CelestialBody, Instant, JulianDay, Longitude, TimeScale};

    let engine = CrossingEngine::new(pleiades_data::packaged_backend());
    let mut out = String::new();
    for line in csv_6col.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            out.push_str(line);
            out.push('\n');
            continue;
        }
        if trimmed.starts_with("frame,") {
            out.push_str(trimmed);
            out.push_str(",pleiades_jd_tdb\n");
            continue;
        }
        let f: Vec<&str> = trimmed.split(',').collect();
        if f.len() != 6 {
            return Err(format!("expected 6 fields, got {}: {trimmed}", f.len()));
        }
        let frame = match f[0] {
            "geo" => CrossingFrame::GeocentricApparentOfDate,
            "helio" => CrossingFrame::Heliocentric,
            other => return Err(format!("unknown frame {other}: {trimmed}")),
        };
        let body = parse_golden_body(f[1]).ok_or_else(|| format!("unknown body: {trimmed}"))?;
        let target: f64 = f[2].parse().map_err(|e| format!("target: {e}: {trimmed}"))?;
        let start_jd: f64 = f[3].parse().map_err(|e| format!("start: {e}: {trimmed}"))?;
        let after = Instant::new(JulianDay::from_days(start_jd), TimeScale::Tdb);
        let crossing = engine
            .next_longitude_crossing(body, Longitude::from_degrees(target), frame, after)
            .map_err(|e| format!("engine: {e}: {trimmed}"))?
            .ok_or_else(|| format!("engine found no crossing: {trimmed}"))?;
        out.push_str(trimmed);
        out.push_str(&format!(",{:.9}\n", crossing.instant.julian_day.days()));
    }
    Ok(out)
}

fn parse_golden_body(name: &str) -> Option<pleiades_types::CelestialBody> {
    use pleiades_types::CelestialBody;
    Some(match name {
        "Sun" => CelestialBody::Sun,
        "Moon" => CelestialBody::Moon,
        "Mercury" => CelestialBody::Mercury,
        "Venus" => CelestialBody::Venus,
        "Mars" => CelestialBody::Mars,
        "Jupiter" => CelestialBody::Jupiter,
        "Saturn" => CelestialBody::Saturn,
        "Uranus" => CelestialBody::Uranus,
        "Neptune" => CelestialBody::Neptune,
        "Pluto" => CelestialBody::Pluto,
        _ => return None,
    })
}
```

- [ ] **Step 4: Run the test to verify it passes.**

Run: `cargo test -p pleiades-validate append_golden_column`
Expected: PASS.

- [ ] **Step 5: Add the `crossings-golden` CLI subcommand.** In the command `match` in `render/cli.rs`, add an arm next to `generate-packaged-artifact` (mirror its `--check`/`--out FILE` argument handling at `cli.rs:265`). The corpus path is `concat!(env!("CARGO_MANIFEST_DIR"), "/data/crossings-corpus/crossings.csv")`:

```rust
        Some("crossings-golden") => {
            let path = concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/data/crossings-corpus/crossings.csv"
            );
            let current = std::fs::read_to_string(path)
                .map_err(|e| format!("read {path}: {e}"))?;
            // Strip any existing 7th column back to the 6-column SE form so the
            // command is idempotent (regenerate from the SE reference columns).
            let six = strip_golden_column(&current);
            let regenerated = append_golden_column(&six)?;
            if args.iter().any(|a| a == "--check") {
                if regenerated != current {
                    return Err(
                        "crossings golden column is stale; run `crossings-golden --regenerate`"
                            .to_string(),
                    );
                }
                return Ok("crossings-golden: committed golden column is current".to_string());
            }
            // --regenerate (default): write it back (or to --out FILE).
            let out_path = arg_value(&args, "--out").unwrap_or_else(|| path.to_string());
            std::fs::write(&out_path, &regenerated)
                .map_err(|e| format!("write {out_path}: {e}"))?;
            let checksum = pleiades_apparent::fnv1a64(&regenerated);
            Ok(format!(
                "crossings-golden: wrote {out_path}; manifest checksum= {checksum}"
            ))
        }
```

Add the `strip_golden_column` helper next to `append_golden_column` (drops a trailing 7th field from data rows and the `,pleiades_jd_tdb` suffix from the header, leaving comments intact), and reuse the existing `arg_value(&args, "--out")` helper if present in `cli.rs` (if not, parse `args.iter().position(|a| a == "--out").and_then(|i| args.get(i + 1)).cloned()`):

```rust
fn strip_golden_column(csv: &str) -> String {
    let mut out = String::new();
    for line in csv.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') {
            out.push_str(line);
        } else if t.starts_with("frame,") {
            out.push_str(t.trim_end().trim_end_matches(",pleiades_jd_tdb"));
        } else {
            let mut f: Vec<&str> = t.split(',').collect();
            if f.len() == 7 {
                f.truncate(6);
            }
            out.push_str(&f.join(","));
        }
        out.push('\n');
    }
    out
}
```

- [ ] **Step 6: Add the command to the help banner + run to confirm it builds.** Add two lines to the giant help string (near the `crossings-gate` entry): `  crossings-golden          Regenerate or (--check) verify the crossings corpus pleiades_jd_tdb golden column`.

Run: `cargo build -p pleiades-validate`
Expected: builds clean.

- [ ] **Step 7: Commit.**

```bash
git add crates/pleiades-validate/src/render/cli.rs
git commit -m "feat(validate): crossings-golden command — regenerate/verify the corpus golden column"
```

---

## Task 4: Two-tier gate + regenerated corpus + fnv1a64 checksum

This is the coupled task: the parser moves from 6 to 7 fields, so the committed corpus must gain its golden column in the same task. Steps regenerate the data, rewrite the gate, measure the arcsecond ceilings, and re-green the corpus test — one commit at the end.

**Files:**
- Modify: `crates/pleiades-validate/src/crossings_validation.rs`
- Modify: `crates/pleiades-validate/data/crossings-corpus/crossings.csv` (regenerated)
- Modify: `crates/pleiades-validate/data/crossings-corpus/manifest.txt`

**Interfaces:**
- Consumes: Task 1 `CrossingEngine::longitude_at`; Task 2 (extended SE tool); Task 3 (`crossings-golden`); `pleiades_apparent::fnv1a64`; `root::wrap180` logic (reimplemented locally as `wrap180_deg`).
- Produces: two-tier `validate_crossings_corpus() -> Result<CrossingsCorpusReport, CrossingsCorpusError>` with the same public name (already re-exported and wired into `run_all_numeric_gates`); no CLI wiring change needed.

- [ ] **Step 1: Regenerate the SE reference data.** From the extended tool (Task 2), write the 6-column SE CSV over the corpus file (preserving the `#` header comment block the tool prints):

```bash
cd tools/se-crossings-reference && cargo run --release > /workspace/crates/pleiades-validate/data/crossings-corpus/crossings.csv
cd /workspace
```

- [ ] **Step 2: Add the golden column.** Run the Task-3 command to append `pleiades_jd_tdb` and print the new checksum:

Run: `cargo run -q -p pleiades-validate -- crossings-golden --regenerate`
Expected: prints `crossings-golden: wrote …/crossings.csv; manifest checksum= <N>`. Record `<N>` and the row count (`grep -cE '^(geo|helio),' crates/pleiades-validate/data/crossings-corpus/crossings.csv`).

- [ ] **Step 3: Update the manifest.** Rewrite `crates/pleiades-validate/data/crossings-corpus/manifest.txt` to (a) update `rows:` to the new count, (b) replace the `sha256(crossings.csv): …` line with a `checksum=<N>` line (decimal fnv1a64 from Step 2), and (c) update the generator line:

```
corpus: crossings
source: Swiss Ephemeris 2.10.03 (Moshier, SEFLG_MOSEPH)
generator: tools/se-crossings-reference (SE reference) + crossings-golden --regenerate (engine golden)
rows: <N>
checksum=<N-checksum>
frames: geo (apparent tropical of date), helio (SEFLG_HELCTR)
window: 1900-2100 CE (JD 2415020.5..=2488069.5), times in TDB
```

- [ ] **Step 4: Write the failing gate tests.** Replace the entire `#[cfg(test)] mod tests` block at the bottom of `crossings_validation.rs` (including the "intentionally omitted" comment) with:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_crossings_passes_over_committed_corpus() {
        let report = validate_crossings_corpus().expect("gate should pass");
        // Pin the fixture count so a corpus that silently loses rows fails.
        assert_eq!(report.checked, EXPECTED_ROWS, "unexpected fixture count");
    }

    #[test]
    fn manifest_checksum_matches_corpus() {
        // Closes spec §7: the manifest's fnv1a64 must equal the live CSV digest.
        let (_rows, want) = parse_manifest().expect("manifest parses");
        assert_eq!(fnv1a64(CORPUS_CSV), want, "manifest checksum drifted from crossings.csv");
    }

    #[test]
    fn tier1_catches_golden_drift() {
        // A row whose pleiades_jd_tdb golden is perturbed beyond the sub-second
        // self-consistency ceiling must fail closed.
        let csv = "\
frame,body,target_longitude_deg,start_jd_tdb,direction,crossing_jd_tdb,pleiades_jd_tdb
geo,Sun,0.000000,2416000.500000,fwd,2416195.301931810,2416199.301931810
";
        let err = validate_crossings_csv(csv).unwrap_err();
        assert!(matches!(err, CrossingsCorpusError::SelfConsistencyExceeded { .. }), "{err:?}");
    }

    #[test]
    fn tier2_catches_longitude_drift() {
        // A target offset far from where the engine actually is at the SE time
        // must fail the arcsecond parity tier.
        let csv = "\
frame,body,target_longitude_deg,start_jd_tdb,direction,crossing_jd_tdb,pleiades_jd_tdb
geo,Sun,10.000000,2416000.500000,fwd,2416195.301931810,PLEIADES
";
        // Fill the golden with the engine's real recompute so Tier 1 passes and
        // only Tier 2 can fire.
        let csv = fill_golden_for_test(csv);
        let err = validate_crossings_csv(&csv).unwrap_err();
        assert!(matches!(err, CrossingsCorpusError::ParityExceeded { .. }), "{err:?}");
    }

    #[test]
    fn non_forward_and_bad_arity_are_schema_errors() {
        let bad = "geo,Sun,0.0,2416000.5,bwd,2416195.3,2416195.3\n";
        assert!(matches!(validate_crossings_csv(bad).unwrap_err(), CrossingsCorpusError::Schema { .. }));
        let short = "geo,Sun,0.0,2416000.5,fwd,2416195.3\n";
        assert!(matches!(validate_crossings_csv(short).unwrap_err(), CrossingsCorpusError::Schema { .. }));
    }

    // Test helper: replace a literal `PLEIADES` golden placeholder with the
    // engine's real next-crossing time so a crafted row exercises Tier 2 alone.
    fn fill_golden_for_test(csv: &str) -> String {
        use pleiades_events::{CrossingEngine, CrossingFrame};
        let engine = CrossingEngine::new(packaged_backend());
        let mut out = String::new();
        for line in csv.lines() {
            if let Some(idx) = line.find(",PLEIADES") {
                let f: Vec<&str> = line[..idx].split(',').collect();
                let frame = if f[0] == "geo" {
                    CrossingFrame::GeocentricApparentOfDate
                } else {
                    CrossingFrame::Heliocentric
                };
                let body = parse_body(f[1]).unwrap();
                let target = Longitude::from_degrees(f[2].parse::<f64>().unwrap());
                let after = Instant::new(JulianDay::from_days(f[3].parse::<f64>().unwrap()), TimeScale::Tdb);
                let c = engine
                    .next_longitude_crossing(body, target, frame, after)
                    .unwrap()
                    .unwrap();
                out.push_str(&format!("{},{:.9}\n", &line[..idx], c.instant.julian_day.days()));
            } else {
                out.push_str(line);
                out.push('\n');
            }
        }
        out
    }
}
```

- [ ] **Step 5: Run the tests to verify they fail.**

Run: `cargo test -p pleiades-validate crossings`
Expected: FAIL — `validate_crossings_csv`, `parse_manifest`, `EXPECTED_ROWS`, `SelfConsistencyExceeded`, `ParityExceeded` not defined.

- [ ] **Step 6: Rewrite the gate.** Replace the header comment + constants + error enum + `validate_crossings_corpus` + `ceiling_for` in `crossings_validation.rs` with the two-tier implementation. Keep `parse_body`, `CrossingsGateOutcome`, and `run_crossings_gate` as-is. New top-of-file (through the end of `validate_crossings_csv`):

```rust
//! Fail-closed two-tier gate over the committed SE crossing corpus.
//!
//! Tier 1 (self-consistency): every row's crossing is recomputed by the packaged
//! engine and compared to the committed `pleiades_jd_tdb` golden within
//! `SELF_CONSISTENCY_TOL_S` — tight teeth against any drift in engine output.
//! Tier 2 (SE parity): the engine's longitude at the SE crossing time is compared
//! to the target within a per-body arcsecond ceiling — honest, unamplified
//! agreement with Swiss Ephemeris across the Moshier-vs-VSOP87/ELP theory floor.
//! A sibling `manifest.txt` records an fnv1a64 digest of the CSV (drift guard).

use pleiades_apparent::fnv1a64;
use pleiades_data::packaged_backend;
use pleiades_events::{CrossingEngine, CrossingFrame};
use pleiades_types::{CelestialBody, Instant, JulianDay, Longitude, TimeScale};

const CORPUS_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/crossings-corpus/crossings.csv"
));
const MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/crossings-corpus/manifest.txt"
));

/// Fixture count pinned by the corpus test. Update when the corpus is regenerated.
const EXPECTED_ROWS: usize = /* set to the Step-2 row count */ 0;

/// Tier-1 self-consistency ceiling: the engine is deterministic, so a recompute
/// matches the committed golden to the bit unless engine output changed. Set a
/// small factor above the root-finder's 0.5 s bisection tolerance.
const SELF_CONSISTENCY_TOL_S: f64 = 1.0;

// Tier-2 per-body arcsecond ceilings — MEASURED from the committed corpus (Step 8),
// set to ~1.4x each body-class group max. These are cross-theory (SE Moshier vs
// engine VSOP87/ELP) floors, not engine error; cf. validate-lilith ~306".
// Initial values below are refined by the Step-8 measurement.
const GEO_SUN_ARCSEC: f64 = 3.0;
const GEO_MOON_ARCSEC: f64 = 40.0;
const GEO_PLANET_ARCSEC: f64 = 60.0;
const HELIO_ARCSEC: f64 = 60.0;
// Pluto is bounded by its documented backend fallback (VSOP87 excludes Pluto);
// its ceiling is a documented coverage boundary, wider than the other planets.
const PLUTO_ARCSEC: f64 = 300.0;

#[derive(Debug)]
pub enum CrossingsCorpusError {
    /// Tier-1: a recomputed crossing drifted from the committed golden.
    SelfConsistencyExceeded { row: String, residual_s: f64, ceiling_s: f64 },
    /// Tier-2: engine longitude at the SE time exceeded the arcsecond ceiling.
    ParityExceeded { row: String, residual_arcsec: f64, ceiling_arcsec: f64 },
    /// The engine found no crossing for a fixture SE reports one for.
    Missing { row: String },
    /// Malformed corpus row.
    Schema { row: String },
    /// Malformed or missing manifest fields.
    Manifest(String),
    /// The committed CSV digest disagrees with the manifest.
    ChecksumMismatch { got: u64, want: u64 },
    /// Engine error.
    Engine(String),
}

impl std::fmt::Display for CrossingsCorpusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for CrossingsCorpusError {}

#[derive(Debug)]
pub struct CrossingsCorpusReport {
    pub checked: usize,
    pub max_self_consistency_s: f64,
    pub max_parity_arcsec: f64,
}

impl CrossingsCorpusReport {
    pub fn summary_line(&self) -> String {
        format!(
            "validate-crossings: {} SE crossing fixtures — Tier 1 self-consistency \
             max {:.3} s (ceiling {:.1} s), Tier 2 SE-parity max {:.1}\" (per-body arcsec ceilings)",
            self.checked, self.max_self_consistency_s, SELF_CONSISTENCY_TOL_S, self.max_parity_arcsec
        )
    }
}

fn wrap180_deg(mut d: f64) -> f64 {
    d = ((d + 180.0).rem_euclid(360.0)) - 180.0;
    d
}

fn parse_manifest() -> Result<(usize, u64), CrossingsCorpusError> {
    let mut rows = None;
    let mut checksum = None;
    for line in MANIFEST.lines() {
        let line = line.trim();
        if let Some(v) = line.strip_prefix("rows:") {
            rows = Some(
                v.trim()
                    .parse::<usize>()
                    .map_err(|e| CrossingsCorpusError::Manifest(format!("rows: {e}")))?,
            );
        }
        for tok in line.split_whitespace() {
            if let Some(v) = tok.strip_prefix("checksum=") {
                checksum = Some(
                    v.parse::<u64>()
                        .map_err(|e| CrossingsCorpusError::Manifest(format!("checksum: {e}")))?,
                );
            }
        }
    }
    Ok((
        rows.ok_or_else(|| CrossingsCorpusError::Manifest("rows: missing".into()))?,
        checksum.ok_or_else(|| CrossingsCorpusError::Manifest("checksum= missing".into()))?,
    ))
}

fn arcsec_ceiling_for(frame: CrossingFrame, body: &CelestialBody) -> f64 {
    match frame {
        CrossingFrame::Heliocentric => match body {
            CelestialBody::Pluto => PLUTO_ARCSEC,
            _ => HELIO_ARCSEC,
        },
        CrossingFrame::GeocentricApparentOfDate => match body {
            CelestialBody::Sun => GEO_SUN_ARCSEC,
            CelestialBody::Moon => GEO_MOON_ARCSEC,
            CelestialBody::Pluto => PLUTO_ARCSEC,
            _ => GEO_PLANET_ARCSEC,
        },
        _ => GEO_PLANET_ARCSEC,
    }
}

/// Validate a 7-column crossings CSV string. `validate_crossings_corpus` calls
/// this with the committed `CORPUS_CSV`; tests call it with crafted rows.
pub(crate) fn validate_crossings_csv(csv: &str) -> Result<CrossingsCorpusReport, CrossingsCorpusError> {
    let engine = CrossingEngine::new(packaged_backend());
    let mut checked = 0usize;
    let mut max_self = 0.0_f64;
    let mut max_parity = 0.0_f64;
    for line in csv.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("frame,") {
            continue;
        }
        let f: Vec<&str> = line.split(',').collect();
        if f.len() != 7 {
            return Err(CrossingsCorpusError::Schema { row: line.to_string() });
        }
        let frame = match f[0] {
            "geo" => CrossingFrame::GeocentricApparentOfDate,
            "helio" => CrossingFrame::Heliocentric,
            _ => return Err(CrossingsCorpusError::Schema { row: line.to_string() }),
        };
        let body = parse_body(f[1]).ok_or_else(|| CrossingsCorpusError::Schema { row: line.to_string() })?;
        if f[4].trim() != "fwd" {
            return Err(CrossingsCorpusError::Schema { row: line.to_string() });
        }
        let target = f[2].parse::<f64>().map_err(|_| CrossingsCorpusError::Schema { row: line.to_string() })?;
        let start_jd = f[3].parse::<f64>().map_err(|_| CrossingsCorpusError::Schema { row: line.to_string() })?;
        let se_jd = f[5].parse::<f64>().map_err(|_| CrossingsCorpusError::Schema { row: line.to_string() })?;
        let golden_jd = f[6].parse::<f64>().map_err(|_| CrossingsCorpusError::Schema { row: line.to_string() })?;
        let after = Instant::new(JulianDay::from_days(start_jd), TimeScale::Tdb);

        // Tier 1: recompute vs committed golden.
        let got = engine
            .next_longitude_crossing(body.clone(), Longitude::from_degrees(target), frame, after)
            .map_err(|e| CrossingsCorpusError::Engine(e.to_string()))?
            .ok_or_else(|| CrossingsCorpusError::Missing { row: line.to_string() })?;
        let residual_s = (got.instant.julian_day.days() - golden_jd).abs() * 86_400.0;
        if residual_s > SELF_CONSISTENCY_TOL_S {
            return Err(CrossingsCorpusError::SelfConsistencyExceeded {
                row: line.to_string(),
                residual_s,
                ceiling_s: SELF_CONSISTENCY_TOL_S,
            });
        }
        max_self = max_self.max(residual_s);

        // Tier 2: engine longitude at the SE time vs target, in arcseconds.
        let se_instant = Instant::new(JulianDay::from_days(se_jd), TimeScale::Tdb);
        let lambda = engine
            .longitude_at(body.clone(), frame, se_instant)
            .map_err(|e| CrossingsCorpusError::Engine(e.to_string()))?;
        let residual_arcsec = wrap180_deg(lambda.degrees() - target).abs() * 3600.0;
        let ceiling_arcsec = arcsec_ceiling_for(frame, &body);
        if residual_arcsec > ceiling_arcsec {
            return Err(CrossingsCorpusError::ParityExceeded {
                row: line.to_string(),
                residual_arcsec,
                ceiling_arcsec,
            });
        }
        max_parity = max_parity.max(residual_arcsec);
        checked += 1;
    }
    Ok(CrossingsCorpusReport { checked, max_self_consistency_s: max_self, max_parity_arcsec: max_parity })
}

pub fn validate_crossings_corpus() -> Result<CrossingsCorpusReport, CrossingsCorpusError> {
    let (manifest_rows, manifest_checksum) = parse_manifest()?;
    let got = fnv1a64(CORPUS_CSV);
    if got != manifest_checksum {
        return Err(CrossingsCorpusError::ChecksumMismatch { got, want: manifest_checksum });
    }
    let report = validate_crossings_csv(CORPUS_CSV)?;
    if report.checked != manifest_rows {
        return Err(CrossingsCorpusError::Manifest(format!(
            "manifest rows={manifest_rows} but corpus has {} data rows",
            report.checked
        )));
    }
    Ok(report)
}
```

Set `EXPECTED_ROWS` to the Step-2 count. Keep the existing `parse_body`, `CrossingsGateOutcome`, `run_crossings_gate` below.

- [ ] **Step 7: Run the crafted-input tests.**

Run: `cargo test -p pleiades-validate crossings`
Expected: `tier1_catches_golden_drift`, `tier2_catches_longitude_drift`, `non_forward_and_bad_arity_are_schema_errors`, `manifest_checksum_matches_corpus` PASS. `validate_crossings_passes_over_committed_corpus` may FAIL on Tier 2 if an initial arcsec ceiling is too tight — that is the measurement signal for Step 8.

- [ ] **Step 8: Measure and set the arcsecond ceilings.** Temporarily raise all `*_ARCSEC` constants to `1.0e9`, run the corpus gate printing per-body residuals via a throwaway `#[ignore]` test, then set each constant to `ceil(1.4 × measured group max)` (and `PLUTO_ARCSEC` to its own boundary value). Add, run, then delete this scratch test:

```rust
    #[test]
    #[ignore] // measurement only; run manually with --ignored
    fn measure_parity_residuals() {
        let r = validate_crossings_corpus().unwrap();
        eprintln!("max parity arcsec = {:.2}", r.max_parity_arcsec);
        // Per-row breakdown: temporarily eprintln inside validate_crossings_csv.
    }
```

Run: `cargo test -p pleiades-validate measure_parity_residuals -- --ignored --nocapture`, read the per-body maxima, set the real ceilings, remove the scratch test and the temporary `1.0e9` values.

- [ ] **Step 9: Run the full crossings gate + regenerate the golden-check.** Confirm the committed golden is current and the gate passes end-to-end.

Run:
```bash
cargo run -q -p pleiades-validate -- crossings-golden --check
cargo test -p pleiades-validate crossings
cargo run -q -p pleiades-validate -- validate-crossings
```
Expected: golden `--check` says current; all crossings tests PASS; `validate-crossings` prints the two-tier summary line.

- [ ] **Step 10: Confirm neighboring gates are unperturbed.**

Run: `cargo run -q -p pleiades-validate -- validate-eclipses && cargo run -q -p pleiades-validate -- validate-angles`
Expected: both PASS.

- [ ] **Step 11: Commit.**

```bash
git add crates/pleiades-validate/src/crossings_validation.rs \
        crates/pleiades-validate/data/crossings-corpus/crossings.csv \
        crates/pleiades-validate/data/crossings-corpus/manifest.txt
git commit -m "feat(validate): two-tier validate-crossings (self-consistency + arcsec SE parity) + fnv1a64 checksum-drift; corpus expanded to geo+helio Mercury-Pluto"
```

---

## Task 5: Compatibility profile, README, PLAN, and spec bookkeeping

**Files:**
- Modify: `crates/pleiades-core/src/compatibility/mod.rs`
- Modify: `README.md`
- Modify: `PLAN.md`
- Modify: `docs/superpowers/specs/2026-07-03-sp2-longitude-crossings-design.md`

**Interfaces:**
- Consumes: the resolved two-tier gate + coverage from Task 4.
- Produces: profile id `0.7.6`, an updated SP-2a capability entry, and consistent README/PLAN/spec status. The overclaim audit (`compat-claims-audit`) must stay green.

- [ ] **Step 1: Bump the profile id.** In `crates/pleiades-core/src/compatibility/mod.rs:26`:

```rust
pub const CURRENT_COMPATIBILITY_PROFILE_ID: &str = "pleiades-compatibility-profile/0.7.6";
```

- [ ] **Step 2: Rewrite the SP-2a capability entry.** In the same file (the `"SP-2a (longitude crossings): …"` string near line 83) and the mirrored sentence inside `CURRENT_COMPATIBILITY_PROFILE_SUMMARY` (near line 41), replace the "per-body time ceilings … are cross-theory-calibrated placeholders pending maintainer review" wording with the resolved description:

```
SP-2a (longitude crossings): a new pleiades-events crate ships a longitude-crossing engine — CrossingEngine with next_sun_crossing/next_moon_crossing (Swiss-Ephemeris solcross/mooncross analogues), general geocentric-apparent-of-date body crossings, heliocentric helio_cross crossings, and a CrossingEngine::longitude_at evaluator — over the 1900-2100 TDB window, exposed via the validate-crossings CLI (aliases crossings / crossings-gate) and not re-exported from pleiades-core. The fail-closed validate-crossings gate is two-tier over a committed corpus covering geocentric and heliocentric bodies Mercury-Pluto (plus Sun/Moon geocentric): Tier 1 recomputes each crossing and holds it to a sub-second self-consistency ceiling vs a committed engine golden column; Tier 2 evaluates the engine's longitude at the Swiss-Ephemeris crossing time and holds it to per-body arcsecond ceilings (documented cross-theory floors vs SE Moshier; Pluto's wider ceiling is a declared coverage boundary bounded by its backend fallback; precedent: validate-lilith accepts an SE-vs-ours floor of ~306"). The corpus is checksum-guarded (fnv1a64) and pinned by row count.
```

Keep the honesty framing but drop "placeholders pending maintainer review".

- [ ] **Step 3: Run the compatibility + overclaim tests.**

Run:
```bash
cargo test -p pleiades-core compatibility
cargo run -q -p pleiades-validate -- compatibility-profile
cargo run -q -p pleiades-validate -- claims-audit
```
Expected: PASS; profile prints `0.7.6`; overclaim audit OK.

- [ ] **Step 4: Update `README.md` "current state".** Replace the crossings sentence with: the longitude-crossing engine (`solcross`/`mooncross`/general/`helio_cross`, plus `longitude_at`) via `pleiades-events`, gated by a **two-tier** `validate-crossings` (sub-second self-consistency + per-body arcsecond SE parity) over a corpus covering geo+helio Mercury–Pluto. Match surrounding prose; the overclaim audit checks README ↔ profile agreement.

- [ ] **Step 5: Update `PLAN.md`.** In the status line, replace the SP-2a sentence (the one describing "per-body time ceilings … placeholders pending maintainer review") with: "SP-2a-FU validate-crossings hardening done (2026-07-03) — two-tier gate (sub-second self-consistency golden + per-body arcsecond SE parity), corpus expanded to geo+helio Mercury–Pluto, fnv1a64 checksum-drift closes spec §7, CrossingEngine::longitude_at added, compatibility profile 0.7.6. SP-2b (rise/set/transit), SP-2c (local eclipse circumstances), and SP-3 remain."

- [ ] **Step 6: Mark the SP-2a design Open item resolved.** In `docs/superpowers/specs/2026-07-03-sp2-longitude-crossings-design.md`, under "Open items", annotate item 1 (per-body time ceilings): append "— **Resolved by SP-2a-FU (2026-07-03):** replaced by a two-tier gate (sub-second self-consistency + per-body arcsecond SE parity); see `2026-07-03-sp2a-crossings-gate-hardening-design.md`."

- [ ] **Step 7: Run the full workspace test + release gate.**

Run:
```bash
cargo test --workspace
cargo run -q -p pleiades-validate -- release-gate
```
Expected: all PASS.

- [ ] **Step 8: Commit.**

```bash
git add crates/pleiades-core/src/compatibility/mod.rs README.md PLAN.md \
        docs/superpowers/specs/2026-07-03-sp2-longitude-crossings-design.md
git commit -m "docs(events): resolve crossings gate ceilings via two-tier gate; compatibility profile 0.7.6; status refresh"
```

---

## Self-Review

**1. Spec coverage:**
- Two-tier gate (Tier 1 self-consistency + Tier 2 arcsec parity), old `*_TOL_S` deleted → Task 4.
- `pleiades_jd_tdb` golden column + `crossings-golden --check/--regenerate` → Tasks 3, 4.
- Corpus expansion to geo+helio Mercury–Pluto via extended SE tool → Tasks 2, 4.
- Pluto coverage-boundary ceiling documented, not dropped → Task 4 (`PLUTO_ARCSEC`, `arcsec_ceiling_for`).
- `CrossingEngine::longitude_at` public method → Task 1.
- fnv1a64 manifest checksum-drift, spec §7 → Tasks 3–4 (`parse_manifest`, `ChecksumMismatch`, `manifest_checksum_matches_corpus`).
- No engine algorithm change → held (only additive `longitude_at`, gate/corpus/docs).
- Compat profile 0.7.5→0.7.6, README, PLAN, design-doc Open item → Task 5.
- Regression: `validate-eclipses`/`validate-angles` green → Task 4 Step 10, Task 5 Step 7.

**2. Placeholder scan:** The only deferred value is `EXPECTED_ROWS` and the five `*_ARCSEC` constants — each has an explicit derivation step (Task 4 Step 2 for the count; Step 8 for the ceilings) with a concrete measurement procedure and initial values, not a "TODO". No "add error handling"/"write tests" placeholders; every logic step ships real code.

**3. Type consistency:** `validate_crossings_csv`, `validate_crossings_corpus`, `CrossingsCorpusReport { checked, max_self_consistency_s, max_parity_arcsec }`, `CrossingsCorpusError::{SelfConsistencyExceeded, ParityExceeded, Missing, Schema, Manifest, ChecksumMismatch, Engine}`, `parse_manifest`, `arcsec_ceiling_for`, `wrap180_deg`, `append_golden_column`, `strip_golden_column`, `parse_golden_body`, and `CrossingEngine::longitude_at(body, frame, instant) -> Result<Longitude, EventError>` are used identically across Tasks 1, 3, 4, 5. The corpus schema `frame,body,target_longitude_deg,start_jd_tdb,direction,crossing_jd_tdb,pleiades_jd_tdb` matches between the tool output (6 cols, Task 2), `append_golden_column` (adds the 7th, Task 3), and the gate parser (7 cols, Task 4).
