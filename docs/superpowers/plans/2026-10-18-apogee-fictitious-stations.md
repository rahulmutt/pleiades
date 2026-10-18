# Apogee and Fictitious-Body Stations Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Gate the stations of the osculating apogee (geocentric apparent) and the 19 fictitious bodies (mean of date) station for station against Swiss Ephemeris SWIEPH, with reference stations taken as zeros of the central difference of SE's longitude.

**Architecture:** `tools/se-stations-reference` gains a pure central-difference scanner module (`cd.rs`: scan, artefact rejection, graze windows) and two modes, `--true-apogee` and `--fictitious`, that write a corpus, a windows file and an extrema file into `crates/pleiades-validate/data/stations-corpus/`. `pleiades-validate`'s `stations_validation` gains a `windows` submodule and two scopes, `Scope::TrueApogee` (packaged engine) and `Scope::Fictitious` (routing engine with `FictitiousBackend`), compared station for station outside windows, by count or parity inside, with the engine's speed sign checked at window edges.

**Tech Stack:** Rust (workspace toolchain from `mise.toml`), `libswisseph-sys` 0.1.2 in the standalone reference tool (built inside devenv), `cargo nextest`/`cargo test`.

**Spec:** `docs/superpowers/specs/2026-10-18-apogee-fictitious-stations-design.md`

## Global Constraints

- Span: JD 2415025.5–2488064.5 (the tool's existing `FULL_SPAN`), TT on the SE side, read as TDB by the engine (under 2 ms apart).
- Apogee: SE id 13 (`SE_OSCU_APOG`), flags `SEFLG_SWIEPH` (default apparent), group `geo`, grid 0.01 day, h = 0.001 day, second scan 1.3 h.
- Fictitious: SE ids 40–58 with the names `Cupido, Hades, Zeus, Kronos, Apollon, Admetos, Vulkanus, Poseidon, Transpluto, Nibiru, Harrington, NeptuneLeverrier, NeptuneAdams, PlutoLowell, PlutoPickering, Vulcan, WhiteMoon, Proserpina, Waldemath`, flags `SEFLG_SWIEPH|SEFLG_TRUEPOS|SEFLG_NOABERR|SEFLG_NOGDEFL|SEFLG_NONUT`, group `mean`, grid 0.25 day, h = 0.01 day, second scan 1.3 h.
- Grid offset: the scan grid starts `GRID_OFFSET_DAYS = 0.00137` after the span start.
- Station bisection tolerance 1e-7 day. Artefact match tolerance 1 s. Artefact pair width: less than 3 × h.
- Window edges: first grid instant on each side with `|cd| >= 2 × margin`. A window is strict when `miss >= margin / 2`.
- Seorbel pin: `seorbel.txt`, SHA-256 `97b454ff78f4f4716b5cc987a93ca8f33e44ef4b524a165a155a8a4885fd2e18`.
- Ceilings: measured maximum × 1.4, rounded up to two significant figures; `lon_arcsec` never below 2.3.
- Margin: 3 × the largest engine-minus-SE speed difference at the SE extrema below 0.1°/day, rounded up to one significant figure.
- Pure Rust in the workspace; the C dependency stays in the standalone tool.
- Code is `cargo fmt`-clean and passes `cargo clippy --workspace --all-targets --all-features -- -D warnings`; rustdoc passes `mise run docs`.
- Run `pleiades-validate` tests with `cargo test -p pleiades-validate --lib <filter>` (add `-- --include-ignored` only where a step says so); long runs (corpus generation, the full gate, `mise run test-full`) are run by the controller, not by subagents.
- The tool builds with: `env PATH=/nix/store/irfrbndi76zhkvqsfhmsn4a99iafck29-nix-2.35.2/bin:/nix/store/9d33009br89y4kg1z8jrmaaq2vcrrjz7-devenv-2.1.2/bin:/home/dev/.cargo/bin:/usr/bin:/bin /nix/store/9d33009br89y4kg1z8jrmaaq2vcrrjz7-devenv-2.1.2/bin/devenv shell -- env CFLAGS=-std=gnu17 cargo <build|test|clippy> --release --offline --manifest-path tools/se-stations-reference/Cargo.toml`.

## Review Focus

1. **An SE jump stepped over by the central difference** gives a station pair 2h apart in one scan only; it must be dropped, never written to the corpus (Task 1 `an_isolated_jump_gives_a_dropped_pair`).
2. **A real station very close to another real one** (the apogee has pairs about an hour apart) must survive the artefact filter, because both scans find it (Task 1 `close_real_stations_survive_both_scans`).
3. **A window whose recorded sides contradict its count** must be rejected by both the tool and the gate parser (Task 1 `a_window_whose_count_contradicts_its_sides_aborts`, Task 3 `windows_parse_and_reject_bad_rows`).
4. **A margin too small for the engine–SE speed difference** would leave an ephemeris-dependent station pair in the strict comparison; a blocking test checks the margin against every recorded extremum (Task 3 `margins_cover_the_engine_speed_difference_at_every_extremum`).
5. **The fictitious bodies must be scanned over the full span in the `mean` group,** although the planet corpus's `mean` series use the short span (Task 4 `new_scopes_scan_the_full_span`).

---

### Task 1: Central-difference scanner, artefact filter, windows and modes in the reference tool

**Files:**
- Create: `tools/se-stations-reference/src/cd.rs`
- Modify: `tools/se-stations-reference/src/main.rs` (module docs, `mod cd;`, two modes, shared writer)
- Modify: `tools/se-stations-reference/src/pins.rs` (seorbel pin and `verify_fictitious_files`)

**Interfaces:**
- Produces (tool-internal): `cd::Kind`, `cd::Station`, `cd::Scan`, `cd::Window`, `cd::central_difference`, `cd::scan`, `cd::reject_artefacts`, `cd::graze_windows`.
- Produces (files, Task 2): `true-apogee.csv`, `true-apogee-manifest.txt`, `true-apogee-windows.csv`, `true-apogee-windows-manifest.txt`, `true-apogee-extrema.csv`, and the same five with the `fictitious` stem.
- Corpus CSV: `group,body,jd_tt,lon_deg,kind` (as `stations.csv`).
- Windows CSV: `group,body,turn_jd_tt,miss_deg_per_day,lo_jd_tt,hi_jd_tt,se_stations,lo_side,hi_side,margin_deg_per_day`, numbers `{:.7}` for instants and `{:.3e}` for speeds, sides `1`/`-1`.
- Extrema CSV: `group,body,jd_tt,cd_deg_per_day` — every sampled extremum of `cd` that stays on one side of zero with `|cd| < 0.1`.
- Manifests: `slice stations-<stem> file=<stem>.csv role=stations rows=<n> checksum=<fnv1a64>` and `slice stations-<stem>-windows file=<stem>-windows.csv role=windows rows=<n> checksum=<fnv1a64>`.

- [ ] **Step 1: Write the failing tests in `cd.rs`**

Create `tools/se-stations-reference/src/cd.rs` with the types and stub functions below (each stub `todo!()`), then the tests:

```rust
//! Stations as zeros of the central difference of a longitude series
//! (issue #167 (d)), with rejection of the artefact pairs an isolated jump in
//! the series creates, and graze windows around near-zero speed extrema.
//!
//! Pure: every function takes the longitude as a closure, so the tests drive
//! it with synthetic series and `main.rs` with `swe_calc`.

/// Stations bisected to this width (days).
pub const BISECT_TOLERANCE_DAYS: f64 = 1e-7;
/// The scan grid starts this far after the span start, off the regular
/// instants where SE's apogee jumps were seen (spike 2026-10-18).
pub const GRID_OFFSET_DAYS: f64 = 0.001_37;
/// The second scan's step, relative to the first.
pub const CHECK_STEP_RATIO: f64 = 1.3;
/// Two scans agree on a station when they find it this close (days).
pub const MATCH_TOLERANCE_DAYS: f64 = 1.0 / 86_400.0;
/// Only extrema below this |speed| are written to the extrema file (deg/day).
pub const EXTREMUM_RECORD_DEG_PER_DAY: f64 = 0.1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// Turns retrograde (speed goes from positive to negative).
    R,
    /// Turns direct.
    D,
}

impl Kind {
    pub fn letter(self) -> &'static str {
        match self {
            Self::R => "R",
            Self::D => "D",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Station {
    pub jd: f64,
    pub kind: Kind,
}

/// One scan: its stations and the sampled extrema of the speed that stay on
/// one side of zero, as `(jd, speed)`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Scan {
    pub stations: Vec<Station>,
    pub extrema: Vec<(f64, f64)>,
}

/// One graze window.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Window {
    pub turn_jd: f64,
    pub miss_deg_per_day: f64,
    pub lo: f64,
    pub hi: f64,
    pub se_stations: usize,
    pub lo_side: i8,
    pub hi_side: i8,
}

pub fn wrap180(d: f64) -> f64 {
    (d + 180.0).rem_euclid(360.0) - 180.0
}

/// `wrap180(lon(jd + h) − lon(jd − h)) / 2h`, degrees per day.
pub fn central_difference(lon: &dyn Fn(f64) -> f64, jd: f64, h: f64) -> f64 {
    wrap180(lon(jd + h) - lon(jd - h)) / (2.0 * h)
}

pub fn scan(lon: &dyn Fn(f64) -> f64, span: (f64, f64), grid: f64, h: f64) -> Scan {
    todo!()
}

pub fn reject_artefacts(
    primary: &[Station],
    check: &[Station],
    h: f64,
) -> Result<(Vec<Station>, Vec<[Station; 2]>), String> {
    todo!()
}

pub fn graze_windows(
    speed: &dyn Fn(f64) -> f64,
    extrema: &[(f64, f64)],
    stations: &[Station],
    span: (f64, f64),
    grid: f64,
    margin: f64,
) -> Result<Vec<Window>, String> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A longitude whose speed is `cos(2π t / 10)`: stations every 5 days,
    /// R at t = 2.5 + 10k, D at 7.5 + 10k (speed crosses zero falling, then rising).
    fn wave(t: f64) -> f64 {
        10.0 / (2.0 * std::f64::consts::PI) * (2.0 * std::f64::consts::PI * t / 10.0).sin()
    }

    #[test]
    fn scan_finds_alternating_stations_to_the_tolerance() {
        let s = scan(&wave, (0.0, 30.0), 0.25, 0.01);
        let jds: Vec<f64> = s.stations.iter().map(|st| st.jd).collect();
        let want = [2.5, 7.5, 12.5, 17.5, 22.5, 27.5];
        assert_eq!(jds.len(), want.len(), "{jds:?}");
        for (got, want) in jds.iter().zip(want) {
            assert!((got - want).abs() < 1e-5, "{got} vs {want}");
        }
        assert_eq!(s.stations[0].kind, Kind::R);
        assert_eq!(s.stations[1].kind, Kind::D);
        // The one-sided extrema are the wave's peaks, at speed +-1.
        assert!(!s.extrema.is_empty());
        assert!(s.extrema.iter().all(|(_, v)| v.abs() > 0.99), "{:?}", s.extrema);
    }

    /// The wave plus an isolated 0.7" jump in longitude at `jump` only.
    fn jumpy(jump: f64) -> impl Fn(f64) -> f64 {
        move |t: f64| {
            // A constant speed of -0.001 deg/day near zero, so a jump of the
            // central difference flips its sign.
            let base = -0.001 * t;
            if (t - jump).abs() < 1e-9 { base + 0.7 / 3600.0 } else { base }
        }
    }

    #[test]
    fn an_isolated_jump_gives_a_dropped_pair() {
        let h = 0.001;
        // Put the jump where the h scan's grid steps across t ± h.
        let span = (0.0, 2.0);
        let jump = span.0 + GRID_OFFSET_DAYS + 50.0 * 0.01 + h;
        let lon = jumpy(jump);
        let a = scan(&lon, span, 0.01, h);
        let b = scan(&lon, span, 0.01, CHECK_STEP_RATIO * h);
        assert!(!a.stations.is_empty(), "the jump makes a pair in the h scan");
        let (kept, dropped) = reject_artefacts(&a.stations, &b.stations, h).unwrap();
        assert!(kept.is_empty(), "{kept:?}");
        assert_eq!(dropped.len(), a.stations.len() / 2);
        for [first, second] in dropped {
            assert_ne!(first.kind, second.kind);
            assert!(second.jd - first.jd < 3.0 * h);
        }
    }

    #[test]
    fn close_real_stations_survive_both_scans() {
        // Two real stations 0.04 day (about an hour) apart: the speed is
        // (t - 1)(t - 1.04), so it turns retrograde at 1.0 and direct at 1.04.
        let lon = |t: f64| t.powi(3) / 3.0 - 1.02 * t * t + 1.04 * t;
        let a = scan(&lon, (0.0, 2.0), 0.01, 0.001);
        let b = scan(&lon, (0.0, 2.0), 0.01, 0.0013);
        let (kept, dropped) = reject_artefacts(&a.stations, &b.stations, 0.001).unwrap();
        assert!(dropped.is_empty());
        assert_eq!(kept, a.stations);
        assert_eq!(kept.iter().map(|s| s.kind).collect::<Vec<_>>(), [Kind::R, Kind::D]);
        assert!((kept[1].jd - kept[0].jd - 0.04).abs() < 1e-4);
    }

    #[test]
    fn a_lone_unmatched_station_aborts() {
        let a = [Station { jd: 10.0, kind: Kind::R }];
        let b: [Station; 0] = [];
        let err = reject_artefacts(&a, &b, 0.001).unwrap_err();
        assert!(err.contains("10.0"), "{err}");
    }

    #[test]
    fn a_mismatched_kind_is_not_a_match() {
        let a = [Station { jd: 10.0, kind: Kind::R }];
        let b = [Station { jd: 10.0, kind: Kind::D }];
        assert!(reject_artefacts(&a, &b, 0.001).is_err());
    }

    /// A speed `v(t) = (t − 5)^2 / 10 + floor`: one extremum at t = 5.
    fn dip(floor: f64) -> impl Fn(f64) -> f64 {
        move |t: f64| (t - 5.0).powi(2) / 10.0 + floor
    }

    #[test]
    fn a_shallow_dip_becomes_a_window_with_both_sides_above() {
        let speed = dip(0.001);
        let w = graze_windows(&speed, &[(5.0, 0.001)], &[], (0.0, 10.0), 0.01, 0.002).unwrap();
        assert_eq!(w.len(), 1);
        let w = w[0];
        assert_eq!((w.se_stations, w.lo_side, w.hi_side), (0, 1, 1));
        assert!(speed(w.lo) >= 0.004 && speed(w.hi) >= 0.004);
        assert!(speed(w.lo + 0.01) < 0.004 && speed(w.hi - 0.01) < 0.004);
        assert_eq!(w.miss_deg_per_day, 0.001);
    }

    #[test]
    fn a_deep_extremum_is_not_a_window() {
        let w = graze_windows(&dip(0.01), &[(5.0, 0.01)], &[], (0.0, 10.0), 0.01, 0.002).unwrap();
        assert!(w.is_empty());
    }

    #[test]
    fn a_window_whose_count_contradicts_its_sides_aborts() {
        // Both sides above zero, but one station recorded inside.
        let inside = [Station { jd: 5.0, kind: Kind::R }];
        let err = graze_windows(&dip(0.001), &[(5.0, 0.001)], &inside, (0.0, 10.0), 0.01, 0.002)
            .unwrap_err();
        assert!(err.contains("parity") || err.contains("count"), "{err}");
    }

    #[test]
    fn a_window_reaching_the_span_edge_aborts() {
        assert!(graze_windows(&dip(0.001), &[(5.0, 0.001)], &[], (4.9, 5.1), 0.01, 0.002).is_err());
    }

    #[test]
    fn overlapping_windows_merge_and_keep_the_closer_miss() {
        let speed = |t: f64| ((t - 5.0).powi(2) - 0.0001).abs() / 10.0 + 0.0005;
        let extrema = [(4.99, 0.0008), (5.01, 0.0005)];
        let w = graze_windows(&speed, &extrema, &[], (0.0, 10.0), 0.01, 0.002).unwrap();
        assert_eq!(w.len(), 1);
        assert_eq!(w[0].miss_deg_per_day, 0.0005);
        assert_eq!(w[0].turn_jd, 5.01);
    }
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Add `mod cd;` to `main.rs` (after `mod pins;`). Run the tool's test command (Global Constraints) with `test`.
Expected: the new tests FAIL with `not yet implemented`; the existing pins tests pass.

- [ ] **Step 3: Implement `scan`, `reject_artefacts` and `graze_windows`**

```rust
pub fn scan(lon: &dyn Fn(f64) -> f64, span: (f64, f64), grid: f64, h: f64) -> Scan {
    let speed = |jd: f64| central_difference(lon, jd, h);
    let lo = span.0 + GRID_OFFSET_DAYS;
    let steps = ((span.1 - lo) / grid).floor() as u64;
    let mut out = Scan::default();
    let (mut p2, mut p1, mut prev_jd) = (f64::NAN, speed(lo), lo);
    for k in 1..=steps {
        let jd = lo + k as f64 * grid;
        let cur = speed(jd);
        if (p1 <= 0.0) != (cur <= 0.0) {
            let (mut a, mut b, mut f_a) = (prev_jd, jd, p1);
            while b - a > BISECT_TOLERANCE_DAYS {
                let mid = 0.5 * (a + b);
                let f_mid = speed(mid);
                if (f_a <= 0.0) == (f_mid <= 0.0) {
                    a = mid;
                    f_a = f_mid;
                } else {
                    b = mid;
                }
            }
            let kind = if cur > 0.0 { Kind::D } else { Kind::R };
            out.stations.push(Station { jd: 0.5 * (a + b), kind });
        } else if p2.is_finite()
            && (p1 - p2) * (cur - p1) < 0.0
            && (p2 <= 0.0) == (p1 <= 0.0)
        {
            out.extrema.push((prev_jd, p1));
        }
        p2 = p1;
        p1 = cur;
        prev_jd = jd;
    }
    out
}

/// Keeps the stations of `primary` that `check` (the 1.3 h scan) also finds,
/// with the same kind within [`MATCH_TOLERANCE_DAYS`]. Every station found by
/// one scan only must be one of an R/D pair less than 3 h apart in that scan,
/// the signature of an isolated jump stepped over at t ± h; such pairs are
/// returned as dropped (those of `primary`). Anything else is an error.
pub fn reject_artefacts(
    primary: &[Station],
    check: &[Station],
    h: f64,
) -> Result<(Vec<Station>, Vec<[Station; 2]>), String> {
    let matched = |s: &Station, other: &[Station]| {
        other
            .iter()
            .any(|o| o.kind == s.kind && (o.jd - s.jd).abs() <= MATCH_TOLERANCE_DAYS)
    };
    let pairs_of = |list: &[Station], other: &[Station], width: f64| -> Result<Vec<[Station; 2]>, String> {
        let lone: Vec<Station> = list.iter().filter(|s| !matched(s, other)).copied().collect();
        let mut pairs = Vec::new();
        let mut i = 0;
        while i < lone.len() {
            let pair = lone.get(i + 1).filter(|next| {
                next.kind != lone[i].kind && next.jd - lone[i].jd < width
            });
            match pair {
                Some(next) => {
                    pairs.push([lone[i], *next]);
                    i += 2;
                }
                None => {
                    return Err(format!(
                        "station at jd_tt={:.7} ({}) is found by one scan only and is not one of an artefact pair",
                        lone[i].jd,
                        lone[i].kind.letter()
                    ))
                }
            }
        }
        Ok(pairs)
    };
    let dropped = pairs_of(primary, check, 3.0 * h)?;
    pairs_of(check, primary, 3.0 * CHECK_STEP_RATIO * h)?;
    let kept = primary.iter().filter(|s| matched(s, check)).copied().collect();
    Ok((kept, dropped))
}

/// One window per extremum of `extrema` within `margin` of zero: from the
/// first grid instant on each side (on the scan's grid, from the extremum's
/// sample) at which `|speed| >= 2 × margin`. Overlapping windows merge and
/// keep the closer miss. Each window counts the `stations` inside it and
/// records the sign of `speed` at both edges; a count of the wrong parity,
/// or a window reaching the span's edge, is an error.
pub fn graze_windows(
    speed: &dyn Fn(f64) -> f64,
    extrema: &[(f64, f64)],
    stations: &[Station],
    span: (f64, f64),
    grid: f64,
    margin: f64,
) -> Result<Vec<Window>, String> {
    let far = |jd: f64| speed(jd).abs() >= 2.0 * margin;
    let mut windows: Vec<Window> = Vec::new();
    for &(turn_jd, value) in extrema {
        if value.abs() >= margin {
            continue;
        }
        let (mut lo, mut hi) = (turn_jd, turn_jd);
        while !far(lo) {
            lo -= grid;
            if lo < span.0 {
                return Err(format!("graze window at jd_tt={turn_jd:.7} reaches the span's start"));
            }
        }
        while !far(hi) {
            hi += grid;
            if hi > span.1 {
                return Err(format!("graze window at jd_tt={turn_jd:.7} reaches the span's end"));
            }
        }
        let miss = value.abs();
        match windows.last_mut() {
            Some(last) if lo <= last.hi => {
                last.hi = last.hi.max(hi);
                if miss < last.miss_deg_per_day {
                    last.miss_deg_per_day = miss;
                    last.turn_jd = turn_jd;
                }
            }
            _ => windows.push(Window {
                turn_jd,
                miss_deg_per_day: miss,
                lo,
                hi,
                se_stations: 0,
                lo_side: 0,
                hi_side: 0,
            }),
        }
    }
    let side = |jd: f64| if speed(jd) > 0.0 { 1 } else { -1 };
    for w in &mut windows {
        w.se_stations = stations.iter().filter(|s| w.lo <= s.jd && s.jd <= w.hi).count();
        w.lo_side = side(w.lo);
        w.hi_side = side(w.hi);
        if (w.lo_side != w.hi_side) != (w.se_stations % 2 == 1) {
            return Err(format!(
                "graze window [{:.7}, {:.7}] holds {} stations, but its edge sides {},{} imply the other parity (count)",
                w.lo, w.hi, w.se_stations, w.lo_side, w.hi_side
            ));
        }
    }
    Ok(windows)
}
```

The `extrema` passed in must be in ascending order (the scan produces them that way).

- [ ] **Step 4: Run the tests to verify they pass**

Run the tool's test command. Expected: all `cd` tests and the pins tests PASS. If `an_isolated_jump_gives_a_dropped_pair` finds no pair because the jump misses the h scan's samples, move `jump` onto a sample of `jd ± h` for a grid instant `jd`, keeping the test's intent: the h scan finds a pair, the 1.3 h scan does not.

- [ ] **Step 5: Pin `seorbel.txt` and add `verify_fictitious_files`**

Copy from `tools/se-aspects-reference/src/pins.rs` into `tools/se-stations-reference/src/pins.rs`: `PINNED_SEORBEL` (`seorbel.txt`, `97b454ff78f4f4716b5cc987a93ca8f33e44ef4b524a165a155a8a4885fd2e18`), `verify_fictitious_files`, and its tests (`fictitious_verification_rejects_a_seorbel_with_the_wrong_content` and any sibling test of that function there). Keep the message pointing at `tools/se-fictitious-reference/data/seorbel.txt`.

- [ ] **Step 6: Add the two modes to `main.rs`**

Add the constants:

```rust
const SE_OSCU_APOG: c_int = 13;
const TRUE_APOGEE_GRID_DAYS: f64 = 0.01;
const TRUE_APOGEE_STEP_DAYS: f64 = 0.001;
const FICTITIOUS_GRID_DAYS: f64 = 0.25;
const FICTITIOUS_STEP_DAYS: f64 = 0.01;
/// Graze margins of the central-difference speed (deg/day): 3 times the
/// largest engine-minus-SE speed difference at the SE extrema below 0.1
/// deg/day. Provisional values; Task 2 of the plan measures and replaces them.
const TRUE_APOGEE_SPEED_MARGIN_DEG_PER_DAY: f64 = 2e-3;
const FICTITIOUS_SPEED_MARGIN_DEG_PER_DAY: f64 = 1e-4;
const FICTITIOUS: [(c_int, &str); 19] = [
    (40, "Cupido"), (41, "Hades"), (42, "Zeus"), (43, "Kronos"), (44, "Apollon"),
    (45, "Admetos"), (46, "Vulkanus"), (47, "Poseidon"), (48, "Transpluto"),
    (49, "Nibiru"), (50, "Harrington"), (51, "NeptuneLeverrier"), (52, "NeptuneAdams"),
    (53, "PlutoLowell"), (54, "PlutoPickering"), (55, "Vulcan"), (56, "WhiteMoon"),
    (57, "Proserpina"), (58, "Waldemath"),
];
```

Add a longitude-only reader (no `SEFLG_SPEED`; the existing `state` asserts on a finite speed only where it is used, so add):

```rust
/// Longitude in [0, 360) of `swe_calc(jd_tt, ipl, iflag)`, failing closed on
/// an error, a SWIEPH fallback or a non-finite value.
fn longitude(jd_tt: f64, ipl: c_int, iflag: c_int) -> f64 {
    let (lon, _) = state(jd_tt, ipl, iflag);
    lon
}
```

(`state` already panics on an error, a fallback and a non-finite longitude, and returns the speed unchecked.)

Add one function that scans one body and appends its rows, used by both modes:

```rust
/// The central-difference stations of one body: both scans, artefact
/// rejection, windows and extrema. Appends corpus, windows and extrema rows.
#[allow(clippy::too_many_arguments)]
fn cd_body(
    out: &mut Outputs,
    group: &str,
    name: &str,
    ipl: c_int,
    iflag: c_int,
    grid: f64,
    h: f64,
    margin: f64,
) {
    let lon = |jd: f64| longitude(jd, ipl, iflag);
    let primary = cd::scan(&lon, FULL_SPAN, grid, h);
    let check = cd::scan(&lon, FULL_SPAN, grid, cd::CHECK_STEP_RATIO * h);
    let (kept, dropped) = cd::reject_artefacts(&primary.stations, &check.stations, h)
        .unwrap_or_else(|e| panic!("{group},{name}: {e}"));
    for [a, b] in &dropped {
        eprintln!("{group},{name}: dropped artefact pair {:.7} {} / {:.7} {}", a.jd, a.kind.letter(), b.jd, b.kind.letter());
    }
    let speed = |jd: f64| cd::central_difference(&lon, jd, h);
    let windows = cd::graze_windows(&speed, &primary.extrema, &kept, FULL_SPAN, grid, margin)
        .unwrap_or_else(|e| panic!("{group},{name}: {e}"));
    for s in &kept {
        out.corpus(format!("{group},{name},{:.7},{:.9},{}", s.jd, lon(s.jd), s.kind.letter()));
    }
    for w in &windows {
        out.window(format!(
            "{group},{name},{:.7},{:.3e},{:.7},{:.7},{},{},{},{margin:.3e}",
            w.turn_jd, w.miss_deg_per_day, w.lo, w.hi, w.se_stations, w.lo_side, w.hi_side
        ));
    }
    for &(jd, v) in primary.extrema.iter().filter(|(_, v)| v.abs() < cd::EXTREMUM_RECORD_DEG_PER_DAY) {
        out.extremum(format!("{group},{name},{jd:.7},{v:+.6e}"));
    }
    out.dropped.push((name.to_string(), dropped.len()));
    eprintln!("{group},{name}: {} stations, {} windows, {} artefact pairs dropped", kept.len(), windows.len(), dropped.len());
}
```

`Outputs` collects the three texts and their row counts plus the dropped counts, and a `write(out_dir, stem, header)` method writes the five files (corpus with header, its manifest, windows with header, its manifest, extrema with header) with the manifest lines in the Interfaces block. The corpus header is:

```
# Source: Swiss Ephemeris 2.10.03 (libswisseph-sys 0.1.2), SEFLG_SWIEPH with sepl_18/semo_18 (SHA-256 pinned in tools/se-stations-reference/src/pins.rs)[ and seorbel.txt].
# A row is a sign change of the central difference wrap180(lon(t + h) - lon(t - h)) / 2h of swe_calc(jd_tt, body, iflag) longitudes,
# on a <grid>-day grid offset 0.00137 day from JD 2415025.5, h = <h> day, bisected to 1e-7 day. jd_tt is TT. Not SE's reported speed:
# SE's osculating-apogee speed jumps at isolated instants and its Proserpina speed is inconsistent with its own positions (issue #167 (d)).
# Each body is also scanned with 1.3 h; a station found by one scan only must be one of an R/D pair under 3 h apart (an isolated jump), and is dropped.
# Dropped artefact pairs: <Name> <n>, ... (bodies with none omitted; "none" if no body has any).
# <group>: <frame description>; JD 2415025.5-2488064.5.
# kind: R = turns retrograde, D = turns direct. lon_deg is the longitude at the station.
group,body,jd_tt,lon_deg,kind
```

The windows header states the window rule (edges at `|cd| >= 2 x margin`, sides, parity, strict when `miss >= margin / 2`) and the margin's basis, and carries one machine-readable line, `# margin_deg_per_day=<value as {:e}>`, before the column line, so the gate can read the margin even when the file has no rows. The extrema header states the 0.1 deg/day cut.

Dispatch in `main`:

```rust
if args.iter().any(|a| a == "--true-apogee") {
    true_apogee_main(&args);
    return;
}
if args.iter().any(|a| a == "--fictitious") {
    fictitious_main(&args);
    return;
}
```

`true_apogee_main` reads `--out` and `--ephe` exactly as `asteroids_main` does (factor that block into `fn ephe_setup(args, mode, verify) -> String`, used by all three modes), verifies with `pins::verify_swieph_files`, and runs `cd_body(&mut out, "geo", "TrueApogee", SE_OSCU_APOG, SEFLG_SWIEPH, TRUE_APOGEE_GRID_DAYS, TRUE_APOGEE_STEP_DAYS, TRUE_APOGEE_SPEED_MARGIN_DEG_PER_DAY)`, then writes stem `true-apogee`. `fictitious_main` verifies with `pins::verify_fictitious_files` and runs `cd_body` for each of `FICTITIOUS` with group `mean`, iflag `SEFLG_SWIEPH | MEAN_OF_DATE`, `FICTITIOUS_GRID_DAYS`, `FICTITIOUS_STEP_DAYS`, `FICTITIOUS_SPEED_MARGIN_DEG_PER_DAY`, then writes stem `fictitious`.

Update the module doc: a "True-apogee mode" and a "Fictitious mode" paragraph, each with its flags, grid, h, the central-difference definition, the artefact rule, the windows and extrema files, and the run command (`tools/se-stations-reference/target/release/se-stations-reference --true-apogee --out crates/pleiades-validate/data/stations-corpus`).

- [ ] **Step 7: Build, test and lint the tool**

Run the tool's `build`, `test` and `clippy -- -D warnings` commands (Global Constraints). Expected: build OK, all tests PASS, no warnings.

- [ ] **Step 8: Commit**

```bash
git add tools/se-stations-reference/src
git commit -m "feat(se-stations-reference): central-difference stations with artefact rejection and graze windows for the apogee and fictitious bodies (#167)"
```

---

### Task 2 (controller): Generate the corpora and fix the margins

**Files:**
- Create: `crates/pleiades-validate/data/stations-corpus/{true-apogee,fictitious}{.csv,-manifest.txt,-windows.csv,-windows-manifest.txt,-extrema.csv}`
- Modify: `tools/se-stations-reference/src/main.rs` (the two margin constants and their comment)

This task runs long SE scans and an engine measurement; the controller runs it.

- [ ] **Step 1: Generate with the provisional margins**

```bash
tools/se-stations-reference/target/release/se-stations-reference --true-apogee --ephe <ephe dir with sepl_18, semo_18, seas_18> --out <scratch>/gen1
tools/se-stations-reference/target/release/se-stations-reference --fictitious --ephe <ephe dir with sepl_18, semo_18, seorbel.txt> --out <scratch>/gen1
```

Expected from the spike: about 4786 apogee stations less the artefact pairs (8 dropped pairs or fewer), 7461 Vulcan stations, 398–400 for each of 16 other fictitious bodies, none for WhiteMoon and Waldemath. Record the counts and dropped pairs in the ledger.

- [ ] **Step 2: Measure the engine-minus-SE speed difference at the recorded extrema**

For every row of `<scratch>/gen1/true-apogee-extrema.csv` and `fictitious-extrema.csv`, compute the engine's speed by the same central difference (`EventEngine::longitude_at`, same h, packaged engine for the apogee in `GeocentricApparentOfDate`, routing engine for the fictitious bodies in `GeocentricMeanOfDate`) and take the largest |engine − SE|, with a throwaway scratch probe crate (path dependencies on the worktree crates, empty `[workspace]`, `--offline`, release). Record both maxima in the ledger.

- [ ] **Step 3: Set the margins and regenerate**

Set each constant to 3 × its measured maximum rounded up to one significant figure, and its comment to the measured value, date and rule. Rebuild the tool and rerun Step 1 into `<scratch>/gen2`. Copy the ten files from `gen2` into `crates/pleiades-validate/data/stations-corpus/`. Check that `stations.csv` and `asteroids.csv` are untouched (`git status`).

- [ ] **Step 4: Commit**

```bash
git add tools/se-stations-reference/src/main.rs crates/pleiades-validate/data/stations-corpus
git commit -m "data(stations): SWIEPH central-difference station corpora for the osculating apogee and the fictitious bodies (#167)"
```

---

### Task 3: Windows module, manifest checks and the margin basis in the gate

**Files:**
- Create: `crates/pleiades-validate/src/stations_validation/windows.rs`
- Modify: `crates/pleiades-validate/src/stations_validation.rs` (file constants, body names, errors, `validate_subset`)
- Modify: `crates/pleiades-validate/src/stations_validation/tests.rs`

**Interfaces:**
- Consumes: the Task 2 files; `Series`, `Found`, `Group`, `StationKind`, `check_manifest`, `parse_manifest`, `fnv1a64`.
- Produces:
  - `pub(super) struct SpeedWindow { group: Group, body_name: &'static str, turn_jd: f64, miss: f64, lo: f64, hi: f64, se_stations: usize, lo_side: i8, hi_side: i8, margin: f64 }` with `fn contains(&self, jd: f64) -> bool` and `fn strict(&self) -> bool { self.miss >= 0.5 * self.margin }`.
  - `pub(super) fn parse_windows(csv: &str) -> Result<Vec<SpeedWindow>, StationsError>`
  - `pub(super) fn check_windows(csv: &str, manifest: &str, series: &[Series]) -> Result<Vec<SpeedWindow>, StationsError>` (checksum, row count, parse, `se_stations` against the corpus)
  - `pub(super) fn split_windows(label: &str, windows: &[SpeedWindow], engine: &[Found], corpus: &[Found]) -> Result<(Vec<Found>, Vec<Found>, WindowStats), StationsError>`
  - `pub(super) fn check_window_edges(label: &str, windows: &[SpeedWindow], speed_at: impl Fn(f64) -> Result<f64, StationsError>) -> Result<(), StationsError>`
  - `pub(super) struct WindowStats { windows: usize, strict: usize, differing: usize }` with `absorb`.
  - New `StationsError` variants: `MalformedWindows(String)`, `WindowStationCount { series: String, lo: f64, hi: f64, recorded: usize, corpus: usize }`, `WindowParity { series: String, lo: f64, hi: f64, side: &'static str, count: usize, se_stations: usize }`, `WindowCount { series: String, lo: f64, hi: f64, engine: usize, corpus: usize }`, `WindowEdgeSide { series: String, jd_tt: f64, se_side: i8, engine_speed: f64 }`, each with a `Display` arm naming the series and window.
  - `body_from_name` accepts `TrueApogee` and the 19 fictitious names (Global Constraints), mapping to the `CelestialBody` variant of the same name.
  - File constants `TRUE_APOGEE_CSV`, `TRUE_APOGEE_MANIFEST`, `TRUE_APOGEE_WINDOWS_CSV`, `TRUE_APOGEE_WINDOWS_MANIFEST`, `TRUE_APOGEE_EXTREMA_CSV` and the five `FICTITIOUS_*` twins (`include_str!` of the Task 2 files).
  - `struct SubsetFiles<'a>` (fields `asteroid_csv`, `asteroid_manifest`, `true_apogee_csv`, `true_apogee_manifest`, `true_apogee_windows_csv`, `true_apogee_windows_manifest`, `fictitious_csv`, `fictitious_manifest`, `fictitious_windows_csv`, `fictitious_windows_manifest`) with `const fn committed()`, as in `aspects_validation` (#274); `validate_subset(files: &SubsetFiles)` checks all five pairs before the mean/sid scan.

- [ ] **Step 1: Write the failing tests**

In `stations_validation/tests.rs` add (and update the two existing asteroid tamper tests to `validate_subset(&SubsetFiles { asteroid_csv: &tampered, ..SubsetFiles::committed() })` and `SubsetFiles { asteroid_manifest: &drifted, .. }`):

```rust
use super::windows::{parse_windows, split_windows, SpeedWindow};

fn window(lo: f64, hi: f64, se_stations: usize, sides: (i8, i8), miss: f64) -> SpeedWindow {
    SpeedWindow {
        group: Group::Geo,
        body_name: "TrueApogee",
        turn_jd: 0.5 * (lo + hi),
        miss,
        lo,
        hi,
        se_stations,
        lo_side: sides.0,
        hi_side: sides.1,
        margin: 0.002,
    }
}

#[test]
fn committed_new_corpora_match_their_manifests_and_windows() {
    for (csv, manifest, windows, windows_manifest) in [
        (TRUE_APOGEE_CSV, TRUE_APOGEE_MANIFEST, TRUE_APOGEE_WINDOWS_CSV, TRUE_APOGEE_WINDOWS_MANIFEST),
        (FICTITIOUS_CSV, FICTITIOUS_MANIFEST, FICTITIOUS_WINDOWS_CSV, FICTITIOUS_WINDOWS_MANIFEST),
    ] {
        let series = check_manifest(csv, manifest).expect("manifest matches");
        super::windows::check_windows(windows, windows_manifest, &series).expect("windows match");
    }
}

#[test]
fn new_corpora_hold_the_expected_bodies() {
    let apogee = check_manifest(TRUE_APOGEE_CSV, TRUE_APOGEE_MANIFEST).unwrap();
    assert_eq!(apogee.len(), 1);
    assert_eq!((apogee[0].group, apogee[0].body_name), (Group::Geo, "TrueApogee"));
    let fict = check_manifest(FICTITIOUS_CSV, FICTITIOUS_MANIFEST).unwrap();
    // WhiteMoon and Waldemath never station, so they have no series.
    assert_eq!(fict.len(), 17);
    assert!(fict.iter().all(|s| s.group == Group::Mean));
}

#[test]
fn subset_fails_tampered_new_corpora_and_windows() {
    let apogee = TRUE_APOGEE_CSV.replacen(",R\n", ",D\n", 1);
    assert!(matches!(
        validate_subset(&SubsetFiles { true_apogee_csv: &apogee, ..SubsetFiles::committed() }),
        Err(StationsError::ChecksumMismatch { .. })
    ));
    let fict = FICTITIOUS_CSV.replacen(",R\n", ",D\n", 1);
    assert!(matches!(
        validate_subset(&SubsetFiles { fictitious_csv: &fict, ..SubsetFiles::committed() }),
        Err(StationsError::ChecksumMismatch { .. })
    ));
    let windows = format!("{TRUE_APOGEE_WINDOWS_CSV}# tampered\n");
    assert!(matches!(
        validate_subset(&SubsetFiles { true_apogee_windows_csv: &windows, ..SubsetFiles::committed() }),
        Err(StationsError::ChecksumMismatch { .. })
    ));
    let (rows, checksum) = parse_manifest(FICTITIOUS_WINDOWS_MANIFEST).unwrap();
    let drifted = format!(
        "slice stations-fictitious-windows file=fictitious-windows.csv role=windows rows={} checksum={checksum}\n",
        rows + 1
    );
    assert!(matches!(
        validate_subset(&SubsetFiles { fictitious_windows_manifest: &drifted, ..SubsetFiles::committed() }),
        Err(StationsError::ManifestDrift { .. })
    ));
}

const WINDOWS_HEADER: &str = "group,body,turn_jd_tt,miss_deg_per_day,lo_jd_tt,hi_jd_tt,se_stations,lo_side,hi_side,margin_deg_per_day\n";

#[test]
fn windows_parse_and_reject_bad_rows() {
    let ok = format!("{WINDOWS_HEADER}geo,TrueApogee,2485441.4310000,5.950e-5,2485441.3000000,2485441.5600000,0,1,1,2.000e-3\n");
    let w = parse_windows(&ok).unwrap();
    assert_eq!(w.len(), 1);
    assert!(!w[0].strict());
    for bad in [
        // sides contradict the count's parity
        "geo,TrueApogee,2485441.431,5.95e-5,2485441.3,2485441.56,1,1,1,2e-3",
        "geo,TrueApogee,2485441.431,5.95e-5,2485441.3,2485441.56,0,1,-1,2e-3",
        // turn outside the window, lo >= hi
        "geo,TrueApogee,2485442.0,5.95e-5,2485441.3,2485441.56,0,1,1,2e-3",
        "geo,TrueApogee,2485441.431,5.95e-5,2485441.56,2485441.3,0,1,1,2e-3",
        // miss not within the margin, a non-positive margin, a bad side
        "geo,TrueApogee,2485441.431,2e-3,2485441.3,2485441.56,0,1,1,2e-3",
        "geo,TrueApogee,2485441.431,0,2485441.3,2485441.56,0,1,1,0",
        "geo,TrueApogee,2485441.431,5.95e-5,2485441.3,2485441.56,0,+1,1,2e-3",
        // unknown body, wrong field count
        "geo,Moon,2485441.431,5.95e-5,2485441.3,2485441.56,0,1,1,2e-3",
        "geo,TrueApogee,2485441.431,5.95e-5,2485441.3,2485441.56,0,1,1",
    ] {
        assert!(
            matches!(parse_windows(&format!("{WINDOWS_HEADER}{bad}\n")), Err(StationsError::MalformedWindows(_))),
            "{bad}"
        );
    }
}

#[test]
fn a_window_count_that_disagrees_with_the_corpus_fails() {
    let series = check_manifest(TRUE_APOGEE_CSV, TRUE_APOGEE_MANIFEST).unwrap();
    let first = series[0].stations[0].jd;
    // A window around the first station that claims to hold none.
    let csv = format!(
        "{WINDOWS_HEADER}geo,TrueApogee,{first:.7},1.000e-4,{:.7},{:.7},0,1,1,2.000e-3\n",
        first - 0.01,
        first + 0.01
    );
    let manifest = format!(
        "slice stations-true-apogee-windows file=true-apogee-windows.csv role=windows rows=1 checksum={}\n",
        fnv1a64(&csv)
    );
    assert!(matches!(
        super::windows::check_windows(&csv, &manifest, &series),
        Err(StationsError::WindowStationCount { recorded: 0, corpus: 1, .. })
    ));
}

#[test]
fn near_tangent_windows_accept_any_split_of_the_right_parity() {
    let w = [window(10.0, 12.0, 2, (1, 1), 0.0005)];
    let pair = |n: usize| -> Vec<Found> {
        (0..n).map(|i| found(10.5 + 0.1 * i as f64, 1.0, if i % 2 == 0 { R } else { D })).collect()
    };
    for (got, want) in [(0, 2), (2, 0), (2, 2), (4, 2)] {
        let (_, _, stats) = split_windows("t", &w, &pair(got), &pair(want)).unwrap();
        assert_eq!(stats.differing, usize::from(got != want));
    }
    for (got, want) in [(1, 2), (2, 1)] {
        assert!(matches!(
            split_windows("t", &w, &pair(got), &pair(want)),
            Err(StationsError::WindowParity { .. })
        ));
    }
}

#[test]
fn strict_windows_require_equal_counts() {
    let w = [window(10.0, 12.0, 2, (1, 1), 0.001)];
    assert!(w[0].strict());
    let pair = vec![found(10.5, 1.0, R), found(10.6, 1.0, D)];
    assert!(matches!(
        split_windows("t", &w, &[], &pair),
        Err(StationsError::WindowCount { engine: 0, corpus: 2, .. })
    ));
    let (eng, cor, stats) = split_windows("t", &w, &pair, &pair).unwrap();
    assert!(eng.is_empty() && cor.is_empty());
    assert_eq!((stats.windows, stats.strict, stats.differing), (1, 1, 0));
}

#[test]
fn stations_outside_windows_stay_strict() {
    let w = [window(10.0, 12.0, 0, (1, 1), 0.0005)];
    let outside = vec![found(12.000_001, 1.0, R)];
    let (eng, cor, _) = split_windows("t", &w, &[], &outside).unwrap();
    assert!(eng.is_empty());
    assert_eq!(cor.len(), 1);
}

#[test]
fn an_engine_speed_on_the_other_side_at_an_edge_fails() {
    use super::windows::check_window_edges;
    let w = [window(10.0, 12.0, 1, (1, -1), 0.0005)];
    let falling = |jd: f64| Ok(if jd == 10.0 { 0.01 } else { -0.01 });
    assert!(check_window_edges("t", &w, falling).is_ok());
    assert!(matches!(
        check_window_edges("t", &w, |_| Ok(0.01)),
        Err(StationsError::WindowEdgeSide { jd_tt, se_side: -1, .. }) if jd_tt == 12.0
    ));
    // Exactly zero counts as negative, as in the reference tool.
    assert!(check_window_edges("t", &w, |jd| Ok(if jd == 10.0 { 0.0 } else { -0.01 })).is_err());
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p pleiades-validate --lib stations_validation::tests::`
Expected: compile errors (missing `windows` module, `SubsetFiles`, constants).

- [ ] **Step 3: Implement**

`windows.rs`, modelled on `aspects_validation/grazes.rs` (read it first):
- `parse_windows`: skip blank, `#` and `group,` lines; exactly 10 fields; group via `Group::from_name`; body via `body_from_name`; finite numbers; sides exactly `"1"` or `"-1"`; reject unless `lo < hi && lo <= turn && turn <= hi`, `margin > 0`, `0 <= miss < margin`, and `(lo_side != hi_side) == (se_stations % 2 == 1)`; windows of one series ascend without overlap. Every failure is `MalformedWindows` naming the line.
- `check_windows`: manifest checksum (`ChecksumMismatch`), parse, row count (`ManifestDrift`), and for each window `se_stations` equals the stations of its series inside `[lo, hi]` (`WindowStationCount`, series label `"<group> <body>"`). Returns the windows.
- `split_windows`: for each window, engine and corpus stations inside; each count's parity must be `se_stations % 2` (`WindowParity`, `side` `"engine"` or `"corpus"`); a strict window with unequal counts is `WindowCount`; counts tallied in `WindowStats`; returns the stations outside every window.
- `check_window_edges`: at `lo` and `hi`, the sign of `speed_at(jd)?` (`> 0.0` is `1`, else `-1`) must equal the recorded side (`WindowEdgeSide`).

`stations_validation.rs`: add `mod windows;`, the constants, the variants with `Display` arms, the names in `body_from_name`, `SubsetFiles`, and the new `validate_subset`:

```rust
fn validate_subset(files: &SubsetFiles) -> Result<StationsReport, StationsError> {
    check_manifest(files.asteroid_csv, files.asteroid_manifest)?;
    for (csv, manifest, windows, windows_manifest) in [
        (files.true_apogee_csv, files.true_apogee_manifest, files.true_apogee_windows_csv, files.true_apogee_windows_manifest),
        (files.fictitious_csv, files.fictitious_manifest, files.fictitious_windows_csv, files.fictitious_windows_manifest),
    ] {
        let series = check_manifest(csv, manifest)?;
        windows::check_windows(windows, windows_manifest, &series)?;
    }
    validate_scoped(CORPUS_CSV, MANIFEST, Scope::MeanSidSubset)
}
```

Update `validate_stations_corpus_subset`'s doc to list the new corpora.

- [ ] **Step 4: Add the margin-basis test**

```rust
/// Each corpus's margin is at least 3 times the engine-minus-SE speed
/// difference at every recorded SE extremum below 0.1 deg/day, and every
/// extremum below the margin lies inside a window (issue #167 (d)).
#[test]
fn margins_cover_the_engine_speed_difference_at_every_extremum() {
    // (extrema csv, windows csv, h in days, frame, engine)
    let apogee_engine = EventEngine::new(packaged_backend());
    let fict_engine = fictitious_engine();
    check_margin_basis(TRUE_APOGEE_EXTREMA_CSV, TRUE_APOGEE_WINDOWS_CSV, TRUE_APOGEE_STEP_DAYS, &apogee_engine, CrossingFrame::GeocentricApparentOfDate);
    check_margin_basis(FICTITIOUS_EXTREMA_CSV, FICTITIOUS_WINDOWS_CSV, FICTITIOUS_STEP_DAYS, &fict_engine, CrossingFrame::GeocentricMeanOfDate);
}
```

with `TRUE_APOGEE_STEP_DAYS = 0.001` and `FICTITIOUS_STEP_DAYS = 0.01` as `const`s in `stations_validation.rs` (doc: "the reference tool's central-difference step"), `fn fictitious_engine() -> EventEngine<RoutingBackend>` in `stations_validation.rs` (`RoutingBackend::new(vec![Box::new(PackagedDataBackend::new()), Box::new(FictitiousBackend::new(PackagedDataBackend::new()))])`), a `pub(super) fn engine_speed<B: EphemerisBackend>(engine, body, reference, jd, h) -> Result<f64, StationsError>` (the central difference of `longitude_at`, wrapped to (−180, 180]) in `stations_validation.rs`, and the test helper `check_margin_basis` that parses the extrema CSV (`group,body,jd_tt,cd_deg_per_day`), reads the margin from the windows file's `# margin_deg_per_day=<value>` header line (Task 1), and asserts for every row: `|engine_speed − cd| * 3 <= margin`, and if `|cd| < margin` a window of that series contains `jd_tt`. Print the maximum difference with `eprintln!`. Also check that every window row's `margin_deg_per_day` equals the header's value.

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test -p pleiades-validate --lib stations_validation::tests::`
Expected: PASS (the margin-basis test may take a few seconds).

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-validate/src/stations_validation.rs crates/pleiades-validate/src/stations_validation
git commit -m "feat(validate): parse and check the apogee and fictitious station corpora and their windows in the blocking tier (#167)"
```

---

### Task 4: Scopes, generic scan, comparison and ceilings

**Files:**
- Modify: `crates/pleiades-validate/src/stations_validation.rs`
- Modify: `crates/pleiades-validate/src/stations_thresholds.rs`
- Modify: `crates/pleiades-validate/src/stations_validation/tests.rs`

**Interfaces:**
- Consumes: Task 3's `windows` module, constants, `fictitious_engine`, `engine_speed`.
- Produces: `Scope::TrueApogee`, `Scope::Fictitious`; `validate_stations_corpus` runs both after the asteroids; `MIN_ROWS_VALIDATED_TRUE_APOGEE`, `MIN_ROWS_VALIDATED_FICTITIOUS`; ceilings for `TrueApogee` and the 17 stationing fictitious bodies.

- [ ] **Step 1: Write the failing tests**

```rust
#[test]
fn new_scopes_scan_the_full_span() {
    let apogee = check_manifest(TRUE_APOGEE_CSV, TRUE_APOGEE_MANIFEST).unwrap();
    let fict = check_manifest(FICTITIOUS_CSV, FICTITIOUS_MANIFEST).unwrap();
    for series in apogee.iter().chain(&fict) {
        assert_eq!(span(series), FULL_SPAN, "{}", series.label());
    }
    // The planet corpus's mean series keep the short span.
    let planets = check_manifest(CORPUS_CSV, MANIFEST).unwrap();
    let mean_mars = planets.iter().find(|s| s.label() == "mean Mars").unwrap();
    assert_eq!(span(mean_mars), SHORT_SPAN);
}

#[test]
fn every_new_body_has_ceilings_and_floors_match_the_corpora() {
    let apogee = check_manifest(TRUE_APOGEE_CSV, TRUE_APOGEE_MANIFEST).unwrap();
    let fict = check_manifest(FICTITIOUS_CSV, FICTITIOUS_MANIFEST).unwrap();
    for series in apogee.iter().chain(&fict) {
        let c = ceilings_for(series.body_name).unwrap_or_else(|| panic!("{}", series.body_name));
        assert!(c.lon_arcsec >= 2.3);
    }
    let in_windows = |csv: &str| -> usize {
        super::windows::parse_windows(csv).unwrap().iter().map(|w| w.se_stations).sum()
    };
    let rows = |s: &[Series]| -> usize { s.iter().map(|x| x.stations.len()).sum() };
    assert_eq!(rows(&apogee) - in_windows(TRUE_APOGEE_WINDOWS_CSV), MIN_ROWS_VALIDATED_TRUE_APOGEE);
    assert_eq!(rows(&fict) - in_windows(FICTITIOUS_WINDOWS_CSV), MIN_ROWS_VALIDATED_FICTITIOUS);
}

#[test]
fn generic_chunked_scan_matches_a_single_scan_on_the_routing_engine() {
    // One fictitious series over two chunks reproduces the single scan.
    let fict = check_manifest(FICTITIOUS_CSV, FICTITIOUS_MANIFEST).unwrap();
    let zeus = fict.iter().find(|s| s.body_name == "Zeus").unwrap();
    let engine = fictitious_engine();
    let short = Series { stations: Vec::new(), ..zeus.clone() };
    let (start, end) = (FULL_SPAN.0, FULL_SPAN.0 + 2.0 * CHUNK_DAYS);
    let whole = scan_chunk(&engine, &short, start, end).unwrap();
    let chunks = vec![
        scan_chunk(&engine, &short, start, start + CHUNK_DAYS).unwrap(),
        scan_chunk(&engine, &short, start + CHUNK_DAYS, end).unwrap(),
    ];
    assert_eq!(join_chunks(chunks), whole);
}

#[test]
#[ignore = "slow: full apogee and fictitious station scans"]
fn the_full_gate_reports_the_new_scopes() {
    let report = validate_stations_corpus().expect("gate passes");
    assert!(report.summary_line.contains("Apogee stations"));
    assert!(report.summary_line.contains("Fictitious-body stations"));
    assert!(report.series_lines.iter().any(|l| l.starts_with("geo TrueApogee:")));
    assert!(report.series_lines.iter().any(|l| l.starts_with("mean Proserpina:")));
}
```

- [ ] **Step 2: Run them to verify they fail**

Run: `cargo test -p pleiades-validate --lib stations_validation::tests::`
Expected: compile errors (scopes, floors, generic `scan_chunk`).

- [ ] **Step 3: Implement**

- `Scope::TrueApogee` and `Scope::Fictitious`: `includes` true, `floor` the new constants, `title` `"Apogee stations"` and `"Fictitious-body stations"`.
- `span(series)`: `FULL_SPAN` when `series.group == Group::Geo && series.body != CelestialBody::TrueNode`, or when the series' body is a fictitious body (`is_fictitious(&series.body)`, a `matches!` over the 19 variants); `SHORT_SPAN` otherwise.
- `scan_chunk` and `scan_series_chunked` take `engine: &EventEngine<B>` with `B: EphemerisBackend`.
- `validate_scoped(csv, manifest, scope)` becomes `validate_scoped_with(engine, csv, manifest, windows: Option<(&str, &str)>, scope)`, generic over `B`, and the existing callers pass `&EventEngine::new(packaged_backend())` and `None`. When `windows` is `Some`, it runs `windows::check_windows`, and for each series, before `compare_series`, runs `check_window_edges` with `|jd| engine_speed(engine, &series.body, series.group.reference(), jd, step)` (step `TRUE_APOGEE_STEP_DAYS` for `Scope::TrueApogee`, `FICTITIOUS_STEP_DAYS` for `Scope::Fictitious`), then `split_windows` over the series' windows, and compares what is outside with `compare_exact`. The series line gains `, windows <n> (<strict> strict, <differing> differing)` for the two new scopes.
- The summary line for the two scopes names the reference: `"vs Swiss Ephemeris SWIEPH central-difference station corpus (station-for-station outside speed-graze windows)"`, with max time and lon.
- `validate_stations_corpus` merges `validate_scoped_with(&EventEngine::new(packaged_backend()), TRUE_APOGEE_CSV, TRUE_APOGEE_MANIFEST, Some((TRUE_APOGEE_WINDOWS_CSV, TRUE_APOGEE_WINDOWS_MANIFEST)), Scope::TrueApogee)` and the fictitious one with `&fictitious_engine()`, after the asteroids.
- `stations_thresholds.rs`: the two floors (doc: corpus rows outside windows, with the generation date) and provisional ceilings from the spike, measured × 1.4 rounded up to two significant figures, `lon_arcsec` at least 2.3:

| Body | time_s | lon_arcsec |
|---|---|---|
| TrueApogee | 930 (from 663 s) | 2.3 |
| Cupido | 32 | 2.3 |
| Hades | 27 | 2.3 |
| Zeus | 30 | 2.3 |
| Kronos | 25 | 2.3 |
| Apollon | 30 | 2.3 |
| Admetos | 27 | 2.3 |
| Vulkanus | 24 | 2.3 |
| Poseidon | 31 | 2.3 |
| Transpluto | 19 | 2.3 |
| Nibiru | 57 | 2.3 |
| Harrington | 19 | 2.3 |
| NeptuneLeverrier | 37 | 2.3 |
| NeptuneAdams | 37 | 2.3 |
| PlutoLowell | 28 | 2.3 |
| PlutoPickering | 23 | 2.3 |
| Vulcan | 140 | 2.3 |
| Proserpina | 27 | 2.3 |

Each entry carries a comment with the spike's measured maximum (spec table) and "provisional, re-measured by the full gate run (plan Task 5)".

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p pleiades-validate --lib stations_validation::tests::`
Expected: PASS (the `#[ignore]` full-gate test is not run here).

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-validate/src
git commit -m "feat(validate): gate osculating-apogee and fictitious-body stations against SWIEPH central-difference corpora (#167)"
```

---

### Task 5 (controller): Full gate run and measured ceilings

**Files:**
- Modify: `crates/pleiades-validate/src/stations_thresholds.rs`

- [ ] **Step 1: Run the full gate**

Run: `cargo run -q -p pleiades-validate -- validate-stations` (in the background, no edits while it runs).
Expected: PASS. If a ceiling fails, the error names the series and residual; raise that one ceiling to residual × 1.4 and rerun.

- [ ] **Step 2: Set the measured ceilings**

From the series lines (max time, max lon per body), set each new ceiling to measured × 1.4 rounded up to two significant figures with the 2.3″ floor, and its comment to the measured values and date. Rerun Step 1.

- [ ] **Step 3: Commit**

```bash
git add crates/pleiades-validate/src/stations_thresholds.rs
git commit -m "test(validate): measured station ceilings for the apogee and fictitious bodies (#167)"
```

---

### Task 6: Docs

**Files:**
- Modify: `crates/pleiades-validate/src/stations_validation.rs` (module doc)
- Modify: `crates/pleiades-validate/src/stations_thresholds.rs` (module doc: the new corpora's reference convention; replace the "not explained ... open observation (issue #167, item (f))" Pluto sentence with the explanation: SE's reported apparent speed omits part of the rate of nutation, about 40 s at Pluto's stations, measured 2026-10-18)
- Modify: `docs/status.md`, `crates/pleiades-events/README.md`, `AGENTS.md` (the stations-gate sentence of the CI paragraph)

- [ ] **Step 1: Write the docs**

Each place states: what is gated (TrueApogee geocentric apparent, the 17 stationing fictitious bodies in the mean of date, 1900–2100, SWIEPH), that these corpora take stations as zeros of the central difference of SE's longitude and why (SE's apogee speed jumps at isolated instants; SE's Proserpina speed is inconsistent with its own positions by about 4.4 h at stations), the window rule, and the measured maxima from Task 5.

- [ ] **Step 2: Check**

Run: `cargo fmt --all --check && cargo clippy --workspace --all-targets --all-features -- -D warnings && mise run docs`
Expected: clean.

- [ ] **Step 3: Commit**

```bash
git add AGENTS.md docs crates/pleiades-events/README.md crates/pleiades-validate/src
git commit -m "docs: apogee and fictitious-body station gating, and the Pluto speed convention (#167)"
```

---

### Finish (controller)

- `mise run test-full`; a nightly dispatched on the branch (`gh workflow run nightly.yml --ref <branch>`). If the `stations-gate` step is 15 minutes or more, move the two scopes to their own job and mise task as #272 did, and re-measure.
- PR, then a comment on #167 ticking (d).
