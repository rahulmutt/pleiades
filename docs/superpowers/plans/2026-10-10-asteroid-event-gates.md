# Asteroid Event Gates Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Gate the event finders' asteroid output (stations, exact aspects, osculating nodes/apsides of Ceres, Pallas, Juno and Vesta) against Swiss Ephemeris reference rows.

**Architecture:** Each Swiss Ephemeris reference tool gains an `--asteroids` mode (SWIEPH with SHA-256-pinned `seas_18`/`sepl_18`/`semo_18`) that writes `asteroids.csv` plus a manifest beside the existing planet corpus. Each existing gate verifies and compares that second file under a new `Scope::Asteroids` with its own ceilings and row floor. Planet corpora, their manifests and the event engine are untouched.

**Tech Stack:** Rust; `libswisseph-sys 0.1.2` (Swiss Ephemeris 2.10.03) in the standalone `tools/se-*-reference` crates; `pleiades-validate` gates; mise tasks; devenv for the SE tools' C build.

**Spec:** `docs/superpowers/specs/2026-10-10-asteroid-event-gates-design.md`

## Global Constraints

- Bodies: Ceres, Pallas, Juno, Vesta only (SE ids 17, 18, 19, 20). Eros stays ungated for events.
- SE data files: `seas_18.se1`, `sepl_18.se1`, `semo_18.se1` from `https://raw.githubusercontent.com/aloistr/swisseph/master/ephe/`, SHA-256-pinned in each tool, verified fail-closed before any asteroid row is written, gitignored under `<tool>/data/`.
- Existing pins (reuse verbatim): `SEPL_18_SHA256 = "ca1393ceab3a44fbc895887cf789c68819ae6a1cbc9b22225872dbe4ccd99a66"`, `SEMO_18_SHA256 = "1ca07bd67c24374d77226180c20a4f9996cba013697894810518e7eb582ca4f7"` (from `tools/se-nodaps-reference/src/main.rs:87-88`).
- Spike stop condition: SE vs sb441 position disagreement > 5″ for any body stops the work; report to the maintainer.
- Ceilings: measured maximum × 1.4, rounded up to two significant figures, never below the spike floor for angle ceilings; each ceiling commented with its measured maximum.
- Planet corpora (`stations.csv`, `aspects.csv`, `nod-aps.csv`) and their `manifest.txt` files must be byte-unchanged.
- Tiers: stations and aspects asteroid rows only in the full gates (`gate-stations`, `gate-aspects`, nightly jobs, `release-gate`); `release-smoke` subsets unchanged. Nod-aps asteroid rows join its existing full run.
- The event engine (`crates/pleiades-events`) is not changed. A defect the gates expose becomes its own GitHub issue; ceilings are not widened around it.
- Commits: conventional style, ending with `Co-authored-by: Claude <noreply@anthropic.com>`. Run `cargo fmt --all` before every commit.
- Unset `PLEIADES_DE_KERNEL`, `PLEIADES_AST_KERNEL`, `PLEIADES_OBJECT_SPK_DIR` for test tiers.
- SE tool build: `devenv shell -- env CFLAGS=-std=gnu17 cargo build --release --manifest-path tools/<tool>/Cargo.toml`, then run the built binary *outside* devenv (devenv prints a banner on stdout).

## Review Focus

1. A missing or wrong-hash data file must abort the asteroid mode before any output file is written (no half-written `asteroids.csv`). Test: Task 2 Step 1.
2. The planet output of a tool must be byte-identical with and without the new code (no flag means the old behaviour exactly). Test: Task 2 Step 7, Task 5 Step 3 and Task 7 Step 3.
3. A truncated or hand-edited `asteroids.csv` must fail the gate even though the planet file is fine. Test: Task 3 Step 1 (`asteroid_corpus_tampering_fails_closed`).
4. An asteroid series the engine finds a different number of stations for (an extra or missing loop) must fail, not be paired off. Covered by the existing `compare_exact`; Task 3 Step 1 adds `a_missing_asteroid_station_fails_the_count`.
5. The merged full-gate report must still name the planet run first and keep its summary prefix, because other tests and the CLI print it. Test: Task 3 Step 1 (`full_report_keeps_the_planet_summary_prefix`).

---

# Phase 1 (PR 1): spike + stations — closes #167 (d)

## File structure (phase 1)

- Create: `docs/superpowers/specs/notes/2026-10-10-se-asteroid-agreement.md`: spike measurement note.
- Modify: `tools/se-stations-reference/Cargo.toml`: add `[workspace]` (nested-worktree builds).
- Create: `tools/se-stations-reference/.gitignore`: the three `.se1` files.
- Create: `tools/se-stations-reference/src/pins.rs`: SHA-256, the three pins, `verify_swieph_files`, `fnv1a64`.
- Modify: `tools/se-stations-reference/src/main.rs`: `--asteroids` mode.
- Create: `crates/pleiades-validate/data/stations-corpus/asteroids.csv`, `asteroids-manifest.txt`: generated.
- Modify: `crates/pleiades-validate/src/stations_validation.rs`: second corpus, `Scope::Asteroids`, merged report.
- Modify: `crates/pleiades-validate/src/stations_thresholds.rs`: four asteroid ceilings, `MIN_ROWS_VALIDATED_ASTEROIDS`.
- Modify: `crates/pleiades-validate/src/stations_validation/tests.rs`: asteroid tests.
- Modify: `docs/` validation page (Task 4 finds it): the asteroid event gating paragraph.

### Task 1: Spike — Swiss Ephemeris asteroid positions vs sb441

Throwaway probe outside the repo; only the note is committed.

**Files:**
- Create (scratch, not committed): `$SCRATCH/se-asteroid-spike/{Cargo.toml,src/main.rs}`, `$SCRATCH` = the session scratchpad.
- Create: `docs/superpowers/specs/notes/2026-10-10-se-asteroid-agreement.md`

**Interfaces:**
- Consumes: `crates/pleiades-jpl/data/corpus/asteroid_reference.csv` (columns `epoch_jd,body,x_km,y_km,z_km`, geocentric ecliptic J2000, TDB).
- Produces: per-body maxima `Δλ·cosβ` and `Δβ` in arcsec, used as the angle-ceiling floor in Tasks 3, 7 and 10.

- [ ] **Step 1: Download and hash the data files**

```bash
mkdir -p $SCRATCH/ephe
for f in seas_18.se1 sepl_18.se1 semo_18.se1; do curl -fLo $SCRATCH/ephe/$f https://raw.githubusercontent.com/aloistr/swisseph/master/ephe/$f; done
sha256sum $SCRATCH/ephe/*.se1
```
Expected: `sepl_18.se1` and `semo_18.se1` match the Global Constraints pins exactly. If either differs, stop: upstream changed and the existing nod-aps pins are stale; report it. Record the `seas_18.se1` digest; it becomes `SEAS_18_SHA256` in Task 2.

- [ ] **Step 2: Write the probe**

`$SCRATCH/se-asteroid-spike/Cargo.toml`:
```toml
[package]
name = "se-asteroid-spike"
version = "0.0.0"
edition = "2021"
publish = false

[workspace]

[dependencies]
libswisseph-sys = "0.1.2"
```

`$SCRATCH/se-asteroid-spike/src/main.rs`:
```rust
//! Throwaway: SE (SWIEPH, seas_18) geometric J2000 geocentric asteroid
//! positions vs the sb441 rows of asteroid_reference.csv.
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

use libswisseph_sys::raw::{swe_calc, swe_set_ephe_path};

// SWIEPH | TRUEPOS | J2000 | NONUT | NOGDEFL | NOABERR
const IFLAG: c_int = 2 | 16 | 32 | 64 | 512 | 1024;

fn main() {
    let mut args = std::env::args().skip(1);
    let csv_path = args.next().expect("asteroid_reference.csv path");
    let ephe = CString::new(args.next().expect("ephe dir")).unwrap();
    unsafe { swe_set_ephe_path(ephe.as_ptr()) };
    let bodies = [("Ceres", 17), ("Pallas", 18), ("Juno", 19), ("Vesta", 20)];
    let text = std::fs::read_to_string(csv_path).unwrap();
    for (name, ipl) in bodies {
        let (mut max_lon, mut max_lat, mut n) = (0.0_f64, 0.0_f64, 0usize);
        for line in text.lines().filter(|l| !l.starts_with('#')) {
            let f: Vec<&str> = line.split(',').collect();
            if f.len() != 5 || f[1] != name {
                continue;
            }
            let jd: f64 = f[0].parse().unwrap();
            let (x, y, z): (f64, f64, f64) =
                (f[2].parse().unwrap(), f[3].parse().unwrap(), f[4].parse().unwrap());
            let lon_ref = y.atan2(x).to_degrees().rem_euclid(360.0);
            let lat_ref = z.atan2(x.hypot(y)).to_degrees();
            let mut xx = [0.0_f64; 6];
            let mut serr = [0 as c_char; 256];
            let ret = unsafe { swe_calc(jd, ipl, IFLAG, xx.as_mut_ptr(), serr.as_mut_ptr()) };
            let msg = unsafe { CStr::from_ptr(serr.as_ptr()) }.to_string_lossy();
            assert!(ret >= 0, "swe_calc {name} {jd}: {msg}");
            assert!(ret & 2 != 0, "{name} {jd}: SE fell back from SWIEPH (ret={ret}): {msg}");
            let dlon = ((xx[0] - lon_ref + 180.0).rem_euclid(360.0) - 180.0)
                * lat_ref.to_radians().cos() * 3600.0;
            let dlat = (xx[1] - lat_ref) * 3600.0;
            max_lon = max_lon.max(dlon.abs());
            max_lat = max_lat.max(dlat.abs());
            n += 1;
        }
        println!("{name}: {n} rows, max |dlon*cos(lat)| {max_lon:.4}\", max |dlat| {max_lat:.4}\"");
    }
}
```

- [ ] **Step 3: Build and run**

```bash
cd $SCRATCH/se-asteroid-spike && devenv shell -- env CFLAGS=-std=gnu17 cargo build --release
$SCRATCH/se-asteroid-spike/target/release/se-asteroid-spike <worktree>/crates/pleiades-jpl/data/corpus/asteroid_reference.csv $SCRATCH/ephe
```
(`devenv shell` must run from the repo worktree root; pass `--manifest-path $SCRATCH/se-asteroid-spike/Cargo.toml` from there if needed.)
Expected: four lines with 407 rows each. The `ret & 2` assertion proves SE used the files, not a Moshier fallback.

If every body's maxima look like 5–20″ with longitude dominant, suspect a light-time convention mismatch: rerun with `IFLAG` without `16` (TRUEPOS) and record both runs.

- [ ] **Step 4: Apply the stop condition**

If any body's maximum exceeds 5″ in either column under the matching convention: stop, write the note anyway, and report the numbers to the maintainer. Otherwise continue.

- [ ] **Step 5: Write the note and commit**

`docs/superpowers/specs/notes/2026-10-10-se-asteroid-agreement.md` records the date, SE version (`swe_version`), the three file digests, `IFLAG`, the four result lines verbatim, and the conclusion: "floor for asteroid angle ceilings: <max over bodies>″".

```bash
git add docs/superpowers/specs/notes/2026-10-10-se-asteroid-agreement.md
git commit -m "docs(notes): Swiss Ephemeris seas_18 vs sb441 asteroid agreement (#167)"
```

### Task 2: Stations tool — pinned data files and `--asteroids` mode

**Files:**
- Modify: `tools/se-stations-reference/Cargo.toml`
- Create: `tools/se-stations-reference/.gitignore`
- Create: `tools/se-stations-reference/src/pins.rs`
- Modify: `tools/se-stations-reference/src/main.rs`

**Interfaces:**
- Consumes: `SEAS_18_SHA256` from Task 1 Step 1.
- Produces: `asteroids.csv` with header comment lines then `group,body,jd_tt,lon_deg,kind`, rows `geo,<Ceres|Pallas|Juno|Vesta>,...`; `asteroids-manifest.txt` with one line `slice stations-asteroids file=asteroids.csv role=stations rows=<N> checksum=<fnv1a64 of the CSV text>`.

- [ ] **Step 1: Write the failing tests for `pins.rs`**

Create `tools/se-stations-reference/src/pins.rs` with only the test module first:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_matches_fips_vectors() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn fnv1a64_matches_the_repo_scheme() {
        // fnv1a64("") is the offset basis; "a" is the published FNV-1a vector.
        assert_eq!(fnv1a64(""), 0xcbf2_9ce4_8422_2325);
        assert_eq!(fnv1a64("a"), 0xaf63_dc4c_8601_ec8c);
    }

    #[test]
    fn verification_fails_closed_on_a_missing_or_wrong_file() {
        let dir = std::env::temp_dir().join(format!("se-pins-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let dir_str = dir.to_str().unwrap();
        assert!(verify_swieph_files(dir_str).is_err(), "missing files must fail");
        for (name, _) in PINNED_FILES {
            std::fs::write(dir.join(name), b"not the real file").unwrap();
        }
        let err = verify_swieph_files(dir_str).unwrap_err();
        assert!(err.contains("SHA-256 mismatch"), "{err}");
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
```
Add `mod pins;` at the top of `main.rs` (after the module doc).

- [ ] **Step 2: Run to verify they fail**

Run (from the worktree root): `devenv shell -- env CFLAGS=-std=gnu17 cargo test --manifest-path tools/se-stations-reference/Cargo.toml`
Expected: compile errors, `sha256_hex`/`fnv1a64`/`verify_swieph_files`/`PINNED_FILES` not found. If cargo instead reports "believes it's in a workspace when it's not", do Step 3's `Cargo.toml` edit first.

- [ ] **Step 3: Implement `pins.rs`, `[workspace]` and `.gitignore`**

`tools/se-stations-reference/Cargo.toml`: append, with the comment `tools/se-nodaps-reference/Cargo.toml` carries:
```toml

# Standalone: an empty [workspace] makes this package its own workspace root,
# so it also builds from a git worktree nested under the repo root (see
# se-nodaps-reference).
[workspace]
```

`tools/se-stations-reference/.gitignore`:
```
data/
```

`tools/se-stations-reference/src/pins.rs`, above the test module:
```rust
//! The Swiss Ephemeris data files the `--asteroids` mode reads, pinned by
//! SHA-256 and verified fail-closed before any row is written. The files are
//! gitignored (`data/`); these constants are the committed provenance. Source:
//! https://raw.githubusercontent.com/aloistr/swisseph/master/ephe/ (the same
//! source and the same sepl/semo digests as `tools/se-nodaps-reference`).

/// `(file name, SHA-256)`. `seas_18` holds the asteroids; a geocentric
/// asteroid also needs the Earth, which SE derives from the Earth–Moon
/// barycentre (`sepl_18`) and the Moon (`semo_18`).
pub const PINNED_FILES: [(&str, &str); 3] = [
    ("seas_18.se1", "<SEAS_18_SHA256 from Task 1 Step 1>"),
    ("sepl_18.se1", "ca1393ceab3a44fbc895887cf789c68819ae6a1cbc9b22225872dbe4ccd99a66"),
    ("semo_18.se1", "1ca07bd67c24374d77226180c20a4f9996cba013697894810518e7eb582ca4f7"),
];

/// Checks every pinned file in `ephe_dir`. `Err` names the first missing or
/// mismatched file and how to fetch it.
pub fn verify_swieph_files(ephe_dir: &str) -> Result<(), String> {
    for (name, want) in PINNED_FILES {
        let path = format!("{ephe_dir}/{name}");
        let bytes = std::fs::read(&path).map_err(|e| {
            format!(
                "cannot read {path}: {e}\nDownload: curl -fLo {path} \
                 https://raw.githubusercontent.com/aloistr/swisseph/master/ephe/{name}"
            )
        })?;
        let got = sha256_hex(&bytes);
        if got != want {
            return Err(format!(
                "SHA-256 mismatch for {path}: got {got}, pinned {want}"
            ));
        }
    }
    Ok(())
}

/// FNV-1a 64-bit over the CSV text, the scheme every corpus manifest uses.
pub fn fnv1a64(text: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0001_0000_01b3);
    }
    hash
}
```
Then copy `sha256_hex` verbatim from `tools/se-nodaps-reference/src/main.rs:137-208` (doc comment included) into `pins.rs`, made `pub`. The `<SEAS_18_SHA256 …>` text is replaced by the 64-hex digest recorded in Task 1; it must not be committed as written.

- [ ] **Step 4: Run the pins tests**

Run: `devenv shell -- env CFLAGS=-std=gnu17 cargo test --manifest-path tools/se-stations-reference/Cargo.toml`
Expected: 3 passed.

- [ ] **Step 5: Add the `--asteroids` mode to `main.rs`**

Changes to `tools/se-stations-reference/src/main.rs`:

1. Module doc: add after the `Ephemeris:` paragraph:
```rust
//! Asteroid mode (`--asteroids --out <dir> [--ephe <dir>]`): `geo` series of
//! Ceres, Pallas, Juno and Vesta (SE ids 17–20) over the full span with the
//! `geo` flags but SEFLG_SWIEPH instead of SEFLG_MOSEPH, from the
//! `seas_18`/`sepl_18`/`semo_18` files pinned in `pins.rs` (verified before
//! anything is written). Writes `<dir>/asteroids.csv` and
//! `<dir>/asteroids-manifest.txt` itself, so no devenv banner reaches them:
//! `devenv shell -- env CFLAGS=-std=gnu17 cargo build --release --manifest-path tools/se-stations-reference/Cargo.toml`, then
//! `tools/se-stations-reference/target/release/se-stations-reference --asteroids --out crates/pleiades-validate/data/stations-corpus`.
//! The ephe directory is `$SE_EPHE_PATH`, else `--ephe`, else this tool's `data/`.
```
2. Imports: `use libswisseph_sys::raw::{swe_calc, swe_set_ephe_path, swe_set_sid_mode};` and `use std::fmt::Write as _;`.
3. Constants:
```rust
const SEFLG_SWIEPH: c_int = 2;
const BASE_SWIEPH: c_int = SEFLG_SWIEPH | SEFLG_SPEED;
const ASTEROIDS: [(c_int, &str); 4] = [(17, "Ceres"), (18, "Pallas"), (19, "Juno"), (20, "Vesta")];
const DEFAULT_EPHE_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/data");
```
4. `state`: with SWIEPH requested, a return without the SWIEPH bit means SE fell back to Moshier; fail closed. After the `ret < 0` check add:
```rust
    if iflag & SEFLG_SWIEPH != 0 && ret & SEFLG_SWIEPH == 0 {
        panic!("swe_calc(ipl={ipl}) fell back from SWIEPH at jd_tt={jd_tt} (ret={ret})");
    }
```
5. `scan` gains an output parameter, so the asteroid mode can collect rows and the planet mode keeps printing. Change its signature to `fn scan(out: &mut dyn FnMut(String), group: &str, …)` and replace `println!("{group},{name},{root:.7},{lon:.9},{kind}");` with `out(format!("{group},{name},{root:.7},{lon:.9},{kind}"));`. In the planet path of `main`, pass `&mut |row| println!("{row}")`.
6. `main` dispatches on the flag before any planet output:
```rust
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--asteroids") {
        asteroids_main(&args);
        return;
    }
    // … the existing planet body of main, unchanged except the `scan` call sites …
}

fn asteroids_main(args: &[String]) {
    let value_of = |flag: &str| {
        args.iter()
            .position(|a| a == flag)
            .map(|i| args.get(i + 1).unwrap_or_else(|| panic!("{flag} needs a value")).clone())
    };
    let out_dir = value_of("--out").expect("--asteroids needs --out <dir>");
    let ephe_dir = std::env::var("SE_EPHE_PATH")
        .ok()
        .or_else(|| value_of("--ephe"))
        .unwrap_or_else(|| DEFAULT_EPHE_DIR.to_string());
    if let Err(e) = pins::verify_swieph_files(&ephe_dir) {
        eprintln!("{e}");
        std::process::exit(1);
    }
    let ephe = std::ffi::CString::new(ephe_dir).expect("ephe path has NUL");
    unsafe { swe_set_ephe_path(ephe.as_ptr()) };

    let mut csv = String::new();
    csv.push_str("# Source: Swiss Ephemeris 2.10.03 (libswisseph-sys 0.1.2), SEFLG_SWIEPH with seas_18/sepl_18/semo_18 (SHA-256 pinned in tools/se-stations-reference/src/pins.rs).\n");
    csv.push_str("# A row is a sign change of the longitude speed of swe_calc(jd_tt, body, iflag|SEFLG_SPEED),\n");
    csv.push_str("# bracketed on a 0.25-day grid and bisected to 1e-7 day. jd_tt is TT.\n");
    csv.push_str("# geo: apparent, tropical, true equinox of date (default flags); Ceres, Pallas, Juno, Vesta JD 2415025.5-2488064.5.\n");
    csv.push_str("# kind: R = turns retrograde, D = turns direct. lon_deg is the longitude at the station.\n");
    csv.push_str("group,body,jd_tt,lon_deg,kind\n");
    let mut rows = 0usize;
    for (ipl, name) in ASTEROIDS {
        scan(
            &mut |row| {
                rows += 1;
                writeln!(csv, "{row}").unwrap();
            },
            "geo",
            name,
            ipl,
            BASE_SWIEPH,
            FULL_SPAN,
            PLANET_GRID_DAYS,
        );
    }
    let manifest = format!(
        "slice stations-asteroids file=asteroids.csv role=stations rows={rows} checksum={}\n",
        pins::fnv1a64(&csv)
    );
    std::fs::create_dir_all(&out_dir).unwrap_or_else(|e| panic!("create {out_dir}: {e}"));
    let csv_path = format!("{out_dir}/asteroids.csv");
    let manifest_path = format!("{out_dir}/asteroids-manifest.txt");
    std::fs::write(&csv_path, &csv).unwrap_or_else(|e| panic!("write {csv_path}: {e}"));
    std::fs::write(&manifest_path, &manifest).unwrap_or_else(|e| panic!("write {manifest_path}: {e}"));
    eprintln!("wrote {csv_path} ({rows} rows) and {manifest_path}");
}
```
The `rows` counter and `csv` are both captured mutably by the closure; if the borrow checker objects, collect into a `Vec<String>` inside the closure and build `csv` after the loop.

- [ ] **Step 6: Build, then prove the fail-closed path writes nothing**

```bash
devenv shell -- env CFLAGS=-std=gnu17 cargo build --release --manifest-path tools/se-stations-reference/Cargo.toml
mkdir -p $SCRATCH/empty-ephe
tools/se-stations-reference/target/release/se-stations-reference --asteroids --ephe $SCRATCH/empty-ephe --out $SCRATCH/should-not-exist; echo "exit $?"; ls $SCRATCH/should-not-exist 2>&1
```
Expected: `cannot read …/seas_18.se1`, `exit 1`, and `ls: cannot access …: No such file or directory`.

- [ ] **Step 7: Prove the planet output is byte-identical**

```bash
tools/se-stations-reference/target/release/se-stations-reference > $SCRATCH/stations-planets.csv
cmp $SCRATCH/stations-planets.csv crates/pleiades-validate/data/stations-corpus/stations.csv && echo IDENTICAL
```
Expected: `IDENTICAL` (running outside devenv means there is no banner line). The planet path takes minutes (the true node's 0.005-day grid).

- [ ] **Step 8: Generate the asteroid corpus twice and compare**

```bash
mkdir -p tools/se-stations-reference/data && cp $SCRATCH/ephe/*.se1 tools/se-stations-reference/data/
tools/se-stations-reference/target/release/se-stations-reference --asteroids --out crates/pleiades-validate/data/stations-corpus
tools/se-stations-reference/target/release/se-stations-reference --asteroids --out $SCRATCH/regen
cmp crates/pleiades-validate/data/stations-corpus/asteroids.csv $SCRATCH/regen/asteroids.csv && cmp crates/pleiades-validate/data/stations-corpus/asteroids-manifest.txt $SCRATCH/regen/asteroids-manifest.txt && echo REPRODUCIBLE
grep -c ',R$' crates/pleiades-validate/data/stations-corpus/asteroids.csv; grep -c ',D$' crates/pleiades-validate/data/stations-corpus/asteroids.csv
cut -d, -f2 crates/pleiades-validate/data/stations-corpus/asteroids.csv | grep -v '^#' | sort | uniq -c
git status --short tools/se-stations-reference
```
Expected: `REPRODUCIBLE`; R and D counts equal or one apart per body; each body roughly 2 × 200 ≈ 360–420 rows (asteroid synodic periods are 13–16 months); `git status` shows no `data/` entries (gitignored).

- [ ] **Step 9: Commit**

```bash
cargo fmt --all
git add tools/se-stations-reference crates/pleiades-validate/data/stations-corpus/asteroids.csv crates/pleiades-validate/data/stations-corpus/asteroids-manifest.txt
git commit -m "feat(tools): se-stations-reference --asteroids writes a pinned SWIEPH asteroid station corpus (#167)"
```

### Task 3: Stations gate — asteroid corpus under `Scope::Asteroids`

**Files:**
- Modify: `crates/pleiades-validate/src/stations_validation.rs`
- Modify: `crates/pleiades-validate/src/stations_thresholds.rs`
- Test: `crates/pleiades-validate/src/stations_validation/tests.rs`

**Interfaces:**
- Consumes: `asteroids.csv`, `asteroids-manifest.txt` from Task 2.
- Produces: `pub fn validate_stations_corpus() -> Result<StationsReport, StationsError>` (same signature) now covering both corpora; `pub(crate) const MIN_ROWS_VALIDATED_ASTEROIDS: usize`; `ceilings_for("Ceres" | "Pallas" | "Juno" | "Vesta")`.

- [ ] **Step 1: Write the failing tests**

Append to `crates/pleiades-validate/src/stations_validation/tests.rs`:

```rust
#[test]
fn asteroid_corpus_rows_parse_into_four_geo_series() {
    let series = parse_corpus(ASTEROID_CSV).expect("asteroid corpus parses");
    let names: Vec<&str> = series.iter().map(|s| s.body_name).collect();
    assert_eq!(names, ["Ceres", "Pallas", "Juno", "Vesta"]);
    assert!(series.iter().all(|s| s.group == Group::Geo));
    assert!(series.iter().all(|s| span(s) == FULL_SPAN));
    assert!(series.iter().all(|s| ceilings_for(s.body_name).is_some()));
}

#[test]
fn asteroid_corpus_tampering_fails_closed() {
    let tampered = ASTEROID_CSV.replacen(",R\n", ",D\n", 1);
    assert!(matches!(
        validate_scoped(&tampered, ASTEROID_MANIFEST, Scope::Asteroids),
        Err(StationsError::ChecksumMismatch { .. })
    ));
    let (rows, checksum) = parse_manifest(ASTEROID_MANIFEST).unwrap();
    let drifted = format!(
        "slice stations-asteroids file=asteroids.csv role=stations rows={} checksum={checksum}\n",
        rows + 1
    );
    assert!(matches!(
        validate_scoped(ASTEROID_CSV, &drifted, Scope::Asteroids),
        Err(StationsError::ManifestDrift { .. })
    ));
}

#[test]
fn a_missing_asteroid_station_fails_the_count() {
    let mut lines: Vec<&str> = ASTEROID_CSV.lines().collect();
    let first_row = lines
        .iter()
        .position(|l| l.starts_with("geo,Ceres,"))
        .expect("a Ceres row");
    lines.remove(first_row);
    let csv = lines.join("\n") + "\n";
    let (rows, _) = parse_manifest(ASTEROID_MANIFEST).unwrap();
    let manifest = format!(
        "slice stations-asteroids file=asteroids.csv role=stations rows={} checksum={}\n",
        rows - 1,
        fnv1a64(&csv)
    );
    assert!(matches!(
        validate_scoped(&csv, &manifest, Scope::Asteroids),
        Err(StationsError::CountMismatch { .. })
    ));
}

#[test]
fn asteroid_scope_floor_is_its_own() {
    assert_eq!(Scope::Asteroids.floor(), MIN_ROWS_VALIDATED_ASTEROIDS);
    assert!(Scope::Asteroids.includes(Group::Geo));
    let (rows, _) = parse_manifest(ASTEROID_MANIFEST).unwrap();
    assert_eq!(rows, MIN_ROWS_VALIDATED_ASTEROIDS, "every asteroid row is compared");
}

#[test]
fn full_report_keeps_the_planet_summary_prefix() {
    let planets = StationsReport {
        rows_validated: 2,
        series_lines: vec!["geo Mercury: …".into()],
        summary_line: "Stations gate: 2 stations validated".into(),
    };
    let asteroids = StationsReport {
        rows_validated: 3,
        series_lines: vec!["geo Ceres: …".into()],
        summary_line: "Asteroid stations: 3 stations validated".into(),
    };
    let merged = planets.merged_with(asteroids);
    assert_eq!(merged.rows_validated, 5);
    assert_eq!(merged.series_lines, ["geo Mercury: …", "geo Ceres: …"]);
    assert!(merged.summary_line.starts_with("Stations gate: 2 stations validated; "));
    assert!(merged.summary_line.ends_with("Asteroid stations: 3 stations validated"));
}
```
If `StationsReport`'s fields are private to the module, the tests module (a child) can still build it. `a_missing_asteroid_station_fails_the_count` scans the four asteroid series (about 4 × 36,500 two-day samples), so it takes seconds in release; if it exceeds ~30 s in the dev profile, mark it `#[ignore = "slow: run via \`mise test-full\` or \`cargo test -- --include-ignored\`"]`, the repo's convention.

- [ ] **Step 2: Run to verify they fail**

Run: `cargo test -p pleiades-validate --lib stations_validation -- --include-ignored`
Expected: compile errors: `ASTEROID_CSV`, `ASTEROID_MANIFEST`, `Scope::Asteroids`, `MIN_ROWS_VALIDATED_ASTEROIDS`, `merged_with` not found.

- [ ] **Step 3: Implement**

In `stations_validation.rs`:

```rust
const ASTEROID_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/stations-corpus/asteroids.csv"
));
const ASTEROID_MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/stations-corpus/asteroids-manifest.txt"
));
```
`Scope` gains a variant and the three matches gain arms:
```rust
    /// The asteroid corpus (`asteroids.csv`, Swiss Ephemeris SWIEPH with
    /// seas_18): Ceres, Pallas, Juno, Vesta over 1900–2100. Part of the full
    /// gate only (issue #167 (d)).
    Asteroids,
```
```rust
            Self::Asteroids => true,                          // includes
            Self::Asteroids => MIN_ROWS_VALIDATED_ASTEROIDS,  // floor
            Self::Asteroids => "Asteroid stations",           // title
```
`body_from_name` gains:
```rust
        "Ceres" => (CelestialBody::Ceres, "Ceres"),
        "Pallas" => (CelestialBody::Pallas, "Pallas"),
        "Juno" => (CelestialBody::Juno, "Juno"),
        "Vesta" => (CelestialBody::Vesta, "Vesta"),
```
The summary line in `validate_scoped` becomes scope-dependent; keep the planet text exactly and add the asteroid text:
```rust
    let summary_line = if scope == Scope::Asteroids {
        format!(
            "{}: {validated} stations validated across {} series vs Swiss Ephemeris SWIEPH (seas_18) \
             speed-zero corpus (station-for-station), max time {max_time_s:.1} s, max lon {max_lon_arcsec:.3}\"",
            scope.title(),
            series_lines.len(),
        )
    } else {
        /* the existing format!(…) unchanged */
    };
```
`StationsReport` gains:
```rust
impl StationsReport {
    /// The planet report followed by the asteroid report: rows summed, series
    /// lines in order, summary lines joined with "; ".
    fn merged_with(self, other: StationsReport) -> StationsReport {
        let mut series_lines = self.series_lines;
        series_lines.extend(other.series_lines);
        StationsReport {
            rows_validated: self.rows_validated + other.rows_validated,
            series_lines,
            summary_line: format!("{}; {}", self.summary_line, other.summary_line),
        }
    }
}
```
`validate_stations_corpus` runs both:
```rust
pub fn validate_stations_corpus() -> Result<StationsReport, StationsError> {
    let planets = validate(CORPUS_CSV, MANIFEST)?;
    let asteroids = validate_scoped(ASTEROID_CSV, ASTEROID_MANIFEST, Scope::Asteroids)?;
    Ok(planets.merged_with(asteroids))
}
```
and its doc comment gains: "Then the asteroid corpus (`Scope::Asteroids`, floor `MIN_ROWS_VALIDATED_ASTEROIDS`)." The module doc gains a paragraph: "Asteroids (issue #167 (d)): Ceres, Pallas, Juno and Vesta are compared station for station against `asteroids.csv`, generated with SWIEPH and seas_18 by the same tool's `--asteroids` mode, in the full gate only."

In `stations_thresholds.rs`, provisional entries so the measurement run can complete, replaced in Step 5:
```rust
        // PROVISIONAL, replaced by measured × 1.4 in this task's Step 5.
        "Ceres" | "Pallas" | "Juno" | "Vesta" => Some(Ceilings {
            time_s: 86_400.0,
            lon_arcsec: 3600.0,
        }),
```
and
```rust
/// Fail-closed floor for the asteroid corpus: every row of `asteroids.csv`
/// (<N>, generated 2026-10-10), so a dropped station fails.
pub(crate) const MIN_ROWS_VALIDATED_ASTEROIDS: usize = <N>;
```
where `<N>` is the `rows=` value of `asteroids-manifest.txt`.

- [ ] **Step 4: Measure**

Run: `cargo run -q --release -p pleiades-validate -- validate-stations`
Expected: PASS, with a series line per asteroid: `geo Ceres: <n> compared (engine <n>, corpus <n>), max time <t> s, mean signed time <s> s, max lon <l>"`. Record each body's `max time` and `max lon`.

If an asteroid line fails with `CountMismatch` or `KindMismatch`, the engine and SE disagree on whether a loop exists. Do not loosen anything: open a GitHub issue with the series line and the first mismatching `jd_tt`, and stop to report it to the maintainer.

- [ ] **Step 5: Set the measured ceilings**

Replace the provisional arm with one arm per body. Each value is measured × 1.4, rounded up to two significant figures; `lon_arcsec` is never below the Task 1 floor. Shape (values from Step 4):
```rust
        // measured max <t> s, <l>" (geo; SWIEPH seas_18 corpus, 2026-10-10)
        "Ceres" => Some(Ceilings {
            time_s: <ceil2(t * 1.4)>,
            lon_arcsec: <max(ceil2(l * 1.4), spike floor)>,
        }),
```
Do the same for Pallas, Juno and Vesta.

- [ ] **Step 6: Run the tests and the gate**

```bash
cargo test -p pleiades-validate --lib stations_validation -- --include-ignored
cargo run -q --release -p pleiades-validate -- validate-stations
```
Expected: all pass; the gate's summary ends `; Asteroid stations: <N> stations validated across 4 series …`.

- [ ] **Step 7: Prove a regression is caught**

Temporarily set Ceres's `time_s` to `1.0`, rerun `cargo run -q --release -p pleiades-validate -- validate-stations`, and expect `geo Ceres time_seconds ceiling exceeded`. Revert, then `git diff --stat` shows only the intended changes.

- [ ] **Step 8: Commit**

```bash
cargo fmt --all
git add crates/pleiades-validate/src/stations_validation.rs crates/pleiades-validate/src/stations_thresholds.rs crates/pleiades-validate/src/stations_validation/tests.rs
git commit -m "feat(validate): gate asteroid stations against a Swiss Ephemeris SWIEPH corpus (#167)"
```

### Task 4: Docs, full validation, PR 1

**Files:**
- Modify: the validation page in `docs/` that describes `validate-stations` (find it with `grep -rln "validate-stations" docs | grep -v superpowers`).

- [ ] **Step 1: Add the docs paragraph**

In that page's stations section, add:
```markdown
**Asteroids (issue #167 (d)).** Ceres, Pallas, Juno and Vesta are compared station for station against `stations-corpus/asteroids.csv`, which `tools/se-stations-reference --asteroids` generates from Swiss Ephemeris SWIEPH with the SHA-256-pinned `seas_18`/`sepl_18`/`semo_18` files. Swiss Ephemeris's asteroid positions agree with JPL's sb441-n373s within <spike floor>″ (`docs/superpowers/specs/notes/2026-10-10-se-asteroid-agreement.md`). These series run in the full gate only (`mise run gate-stations`, the nightly `stations-gate` job). asteroid:433-Eros is not gated for events: Swiss Ephemeris keeps it in a separate per-asteroid file that is not pinned. Its positions are gated against sb441.
```

- [ ] **Step 2: Check the planet corpora are untouched**

Run: `git diff --exit-code origin/main -- crates/pleiades-validate/data/stations-corpus/stations.csv crates/pleiades-validate/data/stations-corpus/manifest.txt && echo UNCHANGED`
Expected: `UNCHANGED`.

- [ ] **Step 3: Commit, then run the tiers (no edits or commits while they run)**

```bash
cargo fmt --all && git add -A docs && git commit -m "docs: asteroid station gating (#167)"
mise run ci
mise run test-full
mise run gate-stations
```
Expected: all green. `mise run ci` includes `release-smoke`, whose stations subset must be unchanged (it still reports `734`).

- [ ] **Step 4: Push and open PR 1**

Push with `git -c credential.helper= -c credential.helper='!gh auth git-credential' push -u origin HEAD:asteroid-event-gates-stations`. The PR body cites the spec, the plan, the spike numbers, per-body measured maxima and ceilings, the reproducibility check and the validation run. It ends with "Closes the (d) item of #167", not "Closes #167" if other items of #167 remain open (check `gh issue view 167`). After CI is green, squash-merge with `gh pr merge <n> --squash --delete-branch` (never `--auto`), verify the squash carries the branch patch, and comment the measured maxima on #167.

---

# Phase 2 (PR 2): aspects — closes #168 (f)

Start from a fresh worktree on the merged main. Reuse `pins.rs` by copying `tools/se-stations-reference/src/pins.rs` verbatim (including its tests) into `tools/se-aspects-reference/src/pins.rs`. The tools are standalone crates, and the spec accepts one copy per tool.

## File structure (phase 2)

- Modify: `tools/se-aspects-reference/{Cargo.toml,src/main.rs}`; create `.gitignore` (`data/`), `src/pins.rs`.
- Create: `crates/pleiades-validate/data/aspects-corpus/asteroids.csv`, `asteroids-manifest.txt`.
- Modify: `crates/pleiades-validate/src/aspects_validation.rs`, `aspects_thresholds.rs`, the aspects tests file.

### Task 5: Aspects tool `--asteroids` mode

**Interfaces:**
- Produces: `asteroids.csv` with the planet file's columns `group,first,second,angle_deg,jd_tt,first_lon_deg,second_lon_deg,rel_speed_deg_per_day`; pairs (`first`,`second`) = (`Sun`,`Ceres`), (`Sun`,`Pallas`), (`Sun`,`Juno`), (`Sun`,`Vesta`) over `FULL_SPAN`, and (`Moon`,`Ceres`) over `SHORT_SPAN` with `MOON_GRID_DAYS`; manifest line `slice aspects-asteroids file=asteroids.csv role=aspects rows=<N> checksum=<fnv1a64>`.

- [ ] **Step 1:** Add `pins.rs` (copied) with `mod pins;`, `[workspace]` in `Cargo.toml` (same comment as Task 2 Step 3), and `.gitignore` `data/`. Run `devenv shell -- env CFLAGS=-std=gnu17 cargo test --manifest-path tools/se-aspects-reference/Cargo.toml`. Expected: the 3 pins tests pass.
- [ ] **Step 2:** Read `tools/se-aspects-reference/src/main.rs` in full. Give its row-emitting `scan` an output closure parameter `out: &mut dyn FnMut(String)` in place of its `println!` (as Task 2 Step 5 item 5 did), and route the planet `main` through `&mut |row| println!("{row}")`. Add a `state` SWIEPH-fallback check identical to Task 2 Step 5 item 4, applied to both bodies of a pair. Add `asteroids_main` with the body of Task 2 Step 5 item 6, except: the header lines describe the five pairs, angles `0, 60, 90, 120, 180` and the 30″ graze refusal; the loop is
```rust
const ASTEROIDS: [(c_int, &str); 4] = [(17, "Ceres"), (18, "Pallas"), (19, "Juno"), (20, "Vesta")];
for (ipl, name) in ASTEROIDS {
    scan(&mut push, "geo", (SUN, "Sun"), (ipl, name), GEO_SWIEPH, FULL_SPAN, GRID_DAYS);
}
scan(&mut push, "geo", (MOON, "Moon"), (17, "Ceres"), GEO_SWIEPH, SHORT_SPAN, MOON_GRID_DAYS);
```
adapted to `scan`'s actual parameter order and its body-tuple types, with `GEO_SWIEPH` the tool's `GEO` flags with `SEFLG_MOSEPH` (4) replaced by `SEFLG_SWIEPH` (2); and the manifest's slice name is `aspects-asteroids`, role `aspects`.
- [ ] **Step 3:** Build; prove fail-closed (Task 2 Step 6 with this tool); prove the planet output byte-identical against `crates/pleiades-validate/data/aspects-corpus/aspects.csv` (Task 2 Step 7 with this tool, about 15 minutes); generate twice into the corpus dir and `$SCRATCH` and `cmp` (Task 2 Step 8 with this tool). Expected: `IDENTICAL`, `REPRODUCIBLE`, and per-pair row counts with Sun pairs ≈ 5 angles × 2 × 200 years / synodic period, Moon–Ceres ≈ 5 × 2 × 40 × 13.4. If the tool panics on a graze within 30″, record the pair, angle and instant, and stop to report it: a graze rule is #168 (c)'s open question and is out of scope here.
- [ ] **Step 4:** `cargo fmt --all`; commit `feat(tools): se-aspects-reference --asteroids writes a pinned SWIEPH asteroid aspect corpus (#168)`.

### Task 6: Aspects gate — `Scope::Asteroids` with `ASTEROID_PAIRS`

**Interfaces:**
- Produces: `validate_aspects_corpus()` (same signature) covering both corpora; `MIN_ROWS_VALIDATED_ASTEROIDS` in `aspects_thresholds.rs`; ceilings for pair names `Sun-Ceres`, `Sun-Pallas`, `Sun-Juno`, `Sun-Vesta`, `Moon-Ceres` (the gate's existing `first-second` naming; confirm against `ceilings_for` in `aspects_thresholds.rs`).

- [ ] **Step 1: Failing tests.** In the aspects tests file add, with this gate's names (`AspectsError`, `AspectsReport`):
  - `asteroid_corpus_parses_into_the_five_asteroid_pairs`: `parse_corpus(ASTEROID_CSV)` yields rows only for the five pairs, all `Group::Geo`, and `ceilings_for` covers each.
  - `asteroid_corpus_tampering_fails_closed`: the checksum and manifest-drift cases of Task 3 Step 1, with `validate_scoped(…, Scope::Asteroids)` and the `aspects-asteroids` slice line.
  - `asteroid_scope_floor_is_its_own`: `Scope::Asteroids.floor() == MIN_ROWS_VALIDATED_ASTEROIDS == manifest rows`.
  - `full_report_keeps_the_planet_summary_prefix`: the `merged_with` test of Task 3 Step 1, with `AspectsReport`'s fields (read the struct first).
  
  Run `cargo test -p pleiades-validate --lib aspects_validation -- --include-ignored`. Expected: compile errors for the missing items.
- [ ] **Step 2: Implement.**
  - `ASTEROID_CSV`/`ASTEROID_MANIFEST` constants as in Task 3.
  - `const ASTEROID_PAIRS: [Pair; 5] = [pair(Group::Geo, "Sun", "Ceres", FULL_SPAN), pair(Group::Geo, "Sun", "Pallas", FULL_SPAN), pair(Group::Geo, "Sun", "Juno", FULL_SPAN), pair(Group::Geo, "Sun", "Vesta", FULL_SPAN), pair(Group::Geo, "Moon", "Ceres", SHORT_SPAN)];`
  - `Scope::Asteroids`, with `includes` → `true`, `floor` → `MIN_ROWS_VALIDATED_ASTEROIDS`, `title` → `"Asteroid aspects"`, and a new `fn pairs(self) -> &'static [Pair]` returning `&ASTEROID_PAIRS` for `Asteroids` and `&PAIRS` otherwise. Every use of `PAIRS` in `validate_scoped` becomes `scope.pairs()`.
  - `body_from_name` gains the four asteroids.
  - `AspectsReport::merged_with`, as in Task 3.
  - `validate_aspects_corpus` runs `validate(CORPUS_CSV, MANIFEST)?` then `validate_scoped(ASTEROID_CSV, ASTEROID_MANIFEST, Scope::Asteroids)?` and merges.
  - A provisional ceilings arm for the five pair names (`sep_arcsec: 3600.0`, `lon_arcsec: 3600.0`, plus whatever time field `Ceilings` has set to `86_400.0`; read the struct).
  - `MIN_ROWS_VALIDATED_ASTEROIDS` = the manifest's `rows=`.
- [ ] **Step 3: Measure.** Run `cargo run -q --release -p pleiades-validate -- validate-aspects` (about 15 minutes or more; no edits while it runs). Record each asteroid pair's maxima from its report line. An event-count mismatch: open an issue and stop, as in Task 3 Step 4.
- [ ] **Step 4: Set measured ceilings** (× 1.4, two significant figures, angle ceilings ≥ the Task 1 floor), each commented with its measured maximum and the date.
- [ ] **Step 5:** Run the tests and the gate again (expected: pass). Prove a regression is caught by temporarily setting `Sun-Ceres`'s separation ceiling to `1e-6` (expect `ceiling exceeded`), then revert.
- [ ] **Step 6:** `cargo fmt --all`; commit `feat(validate): gate asteroid aspects against a Swiss Ephemeris SWIEPH corpus (#168)`.
- [ ] **Step 7:** Docs paragraph like Task 4 Step 1, in the aspects section (pairs listed; full gate only; Eros not gated). Planet-corpus check: `git diff --exit-code origin/main -- crates/pleiades-validate/data/aspects-corpus/aspects.csv crates/pleiades-validate/data/aspects-corpus/manifest.txt`. Commit. Run `mise run ci`, `mise run test-full`, `mise run gate-aspects`. Push, open PR 2, merge on green (squash, never `--auto`), and comment the measured maxima on #168.

---

# Phase 3 (PR 3): nod-aps — closes #160 for the asteroid subset

Start from a fresh worktree on the merged main.

## File structure (phase 3)

- Modify: `tools/se-nodaps-reference/src/main.rs`, `.gitignore` (add `seas_18.se1`).
- Create: `crates/pleiades-validate/data/nod-aps-corpus/asteroids.csv`, `asteroids-manifest.txt`.
- Modify: `crates/pleiades-validate/src/nod_aps_validation.rs`, `nod_aps_thresholds.rs`, the nod-aps tests.

### Task 7: Nod-aps tool `--asteroids` mode

**Interfaces:**
- Produces: `asteroids.csv` with the planet file's 29 columns (`label,se_body,method,fopoint,jd_tt` + asc/dsc/peri/apo × 6); 32 rows: labels `Ceres`, `Pallas`, `Juno`, `Vesta`, `se_body` 17–20, `method` 2 (`SE_NODBIT_OSCU`), `fopoint` 0, at the 8 `EPOCHS`; manifest in the planet manifest's format (`build_manifest`), with `rows=32`.

- [ ] **Step 1:** Read `tools/se-nodaps-reference/src/main.rs` in full. Add `const SEAS_18_SHA256` (the Task 1 digest) and extend `verify_swieph_files` with a parameter `files: &[(&str, &str)]`: the existing barycentric call passes the sepl/semo pair, and the asteroid mode passes all three. Add `.gitignore` line `seas_18.se1`.
- [ ] **Step 2:** Add `--asteroids` to `parse_args` (`Config.asteroids: bool`). In `main`, when set, verify the three files, then build the CSV with the existing row writer over `[("Ceres", 17), ("Pallas", 18), ("Juno", 19), ("Vesta", 20)] × EPOCHS` with method `SE_NODBIT_OSCU` and `IFLAG_SWIEPH`, and write `asteroids.csv` and `asteroids-manifest.txt` (`build_manifest` with header text describing the asteroid rows) instead of `nod-aps.csv`/`manifest.txt`. `--dry-run` prints them. Assert `rows == 32` before writing.
- [ ] **Step 3:** Build; prove fail-closed with an empty `--ephe` dir (exit non-zero, no files written); prove the planet output byte-identical (`--out $SCRATCH/planets`, then `cmp` both files against `crates/pleiades-validate/data/nod-aps-corpus/{nod-aps.csv,manifest.txt}`); generate `--asteroids --out crates/pleiades-validate/data/nod-aps-corpus` twice and `cmp`. Expected: `IDENTICAL`, `REPRODUCIBLE`, 32 rows.
- [ ] **Step 4:** `cargo fmt --all`; commit `feat(tools): se-nodaps-reference --asteroids writes osculating asteroid rows (#160)`.

### Task 8: Nod-aps gate — asteroid rows and the `ASTEROID` category

**Interfaces:**
- Produces: `body_from_se` accepting 17–20; `pub const EXPECTED_ASTEROID_ROWS: usize = 32;`; an `ASTEROID` category with `OSCU_ASTEROID_*` ceilings in `nod_aps_thresholds.rs`.

- [ ] **Step 1: Failing tests.** In the nod-aps tests:
  - Change the test that asserts ids 15 and 20 are rejected: 15 (Chiron) and 16 (Pholus) are still rejected, and 17–20 map to `Ceres`, `Pallas`, `Juno`, `Vesta`.
  - Add `asteroid_rows_parse_and_count_32` (parse `ASTEROID_CSV`; 32 rows; all method `Osculating`).
  - Add `asteroid_corpus_tampering_fails_closed`, with this gate's manifest format.
  - Add `asteroid_rows_fall_in_the_asteroid_category` (`category_for(&CelestialBody::Ceres, NodApsMethod::Osculating)` is the new asteroid category).
  
  Run `cargo test -p pleiades-validate --lib nod_aps`. Expected: failures and compile errors.
- [ ] **Step 2: Implement.**
  - `body_from_se` arms 17–20 (keep its doc comment accurate: "0–9 and the asteroids 17–20").
  - `ASTEROID_CSV`/`ASTEROID_MANIFEST` constants.
  - `measure()` reads both corpora (verify each checksum against its own manifest; count planet rows against `EXPECTED_ROWS` and asteroid rows against `EXPECTED_ASTEROID_ROWS` separately).
  - `category_for` returns the new asteroid category for the four bodies.
  - Thresholds: provisional `OSCU_ASTEROID_*` values equal to the `OSCU_PLANET_*` ones, to measure first.
  - The summary line's "asteroid nod_aps … engine-covered, gate-unreferenced" text becomes "asteroids (Ceres, Pallas, Juno, Vesta): osculating, gated vs SWIEPH seas_18 rows".
- [ ] **Step 3: Measure.** Run `cargo run -q --release -p pleiades-validate -- validate-nod-aps` and record the asteroid category maxima (longitude, latitude, distance relative, longitude speed) for each of asc/dsc/peri/apo, as the existing categories report them.
- [ ] **Step 4: Set measured ceilings** (× 1.4, two significant figures; angle ceilings ≥ the Task 1 floor), commented with measured maxima and the date. If any asteroid maximum exceeds the existing `OSCU_PLANET_*` ceiling for the same quantity, stop and report: that would point to an engine or backend defect, not a tolerance to set.
- [ ] **Step 5:** Run the tests and the gate (expected: pass). Prove a regression is caught by temporarily setting `OSCU_ASTEROID_LONGITUDE_ARCSEC` to `1e-6`, then revert. `cargo fmt --all`; commit `feat(validate): gate osculating asteroid nodes and apsides against Swiss Ephemeris rows (#160)`.
- [ ] **Step 6:** Docs paragraph in the nod-aps section (osculating asteroid rows; fictitious bodies stay unreferenced because SE's `swe_nod_aps` does not implement them; Eros not gated). Planet-corpus check: `git diff --exit-code origin/main -- crates/pleiades-validate/data/nod-aps-corpus/nod-aps.csv crates/pleiades-validate/data/nod-aps-corpus/manifest.txt`. Commit. Run `mise run ci` (which runs nod-aps in the release battery) and `mise run test-full`. Push, open PR 3, merge on green (squash, never `--auto`). Comment the measured maxima on #160 and close it with "fictitious bodies remain unreferenced by design", unless the maintainer prefers it left open.
