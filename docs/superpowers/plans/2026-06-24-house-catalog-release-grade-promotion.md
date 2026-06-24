# House-Catalog Release-Grade Promotion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Promote the twelve `target`-tier house systems to `ReleaseGradeNumeric`, each backed by Swiss-Ephemeris numeric-gate evidence, closing the house half of the Phase 6 target compatibility catalog.

**Architecture:** Extend the SE reference generator to emit the eleven standard 12-cusp target systems into `cusps.csv` and Gauquelin's 36 sectors into a new variable-length `sectors.csv` (via raw `libswisseph-sys` FFI to avoid the wrapper's 13-element buffer overflow). Extend the house numeric gate to validate both files, set per-family residual ceilings from measured maxima, and promote the catalog claim tiers — keeping the bidirectional overclaim audit green at every commit.

**Tech Stack:** Rust (workspace crates `pleiades-houses`, `pleiades-validate`, `pleiades-core`), the `swisseph` / `libswisseph-sys` crates (dev-only reference tool), FNV-1a-64 corpus checksums via `pleiades_apparent::fnv1a64`.

## Global Constraints

- Reference engine: **SwissEphemeris 2.10.03** via `libswisseph-sys 0.1.2`; regenerating with the same version reproduces existing rows byte-identically. Copy this version verbatim into the manifest.
- Corpus checksums use `pleiades_apparent::fnv1a64` (a **non-canonical FNV prime** — a stock FNV implementation will not reproduce it). Recompute only via that function.
- Ceiling rule (verbatim from Phase 5): each family ceiling = `ceil(measured_max_residual_arcsec × 2)` with a **1.0″ floor**.
- The overclaim audit is **bidirectional**: a system that the corpus validates MUST be `ReleaseGradeNumeric`, and vice-versa. Never commit a state where corpus evidence and `claim_tier` disagree.
- Fail-safe: any target system whose measured residual exceeds a defensible release-grade ceiling stays `DescriptorOnly` and is recorded as a known gap — never promoted with a loosened ceiling. The catalog must not be narrowed; claims must not exceed evidence.
- `cargo fmt` + `cargo clippy` must be clean before each commit.

**Target systems** (`HouseSystem` variant — `swe_houses` code — formula family):
`EqualMidheaven`(D, Equal), `EqualAries`(N, Equal), `Vehlow`(V, Equal), `Sripati`(S, Quadrant), `Carter`(F, EquatorialProjection), `Horizon`(H, GreatCircle), `Apc`(Y, GreatCircle), `KrusinskiPisaGoelzer`(U, GreatCircle), `Sunshine`(I, SolarArc), `PullenSd`(L, Sector), `PullenSr`(Q, Sector), `Gauquelin`(G, Sector — 36 sectors).
The eleven non-Gauquelin systems are standard 12-cusp; Gauquelin is the 36-sector special case.

---

### Task 1: Generator — emit the eleven standard target systems

**Files:**
- Modify: `tools/se-house-reference/src/main.rs` (the `HSYS` table, ~lines 7-21)

**Interfaces:**
- Produces: a regenerated `cusps.csv` on stdout with 5 fixtures × 23 systems = 115 data rows.

- [ ] **Step 1: Add the eleven standard target systems to `HSYS`**

In `tools/se-house-reference/src/main.rs`, extend the `HSYS` constant (after the existing `("Morinus", …)` entry) with:

```rust
    ("EqualMidheaven", HouseSystemKind::EqualMc),
    ("EqualAries",     HouseSystemKind::Equal1Aries),
    ("Vehlow",         HouseSystemKind::VehlowEqual),
    ("Sripati",        HouseSystemKind::Sripati),
    ("Carter",         HouseSystemKind::CarterPoliEquatorial),
    ("Horizon",        HouseSystemKind::AzimuthalHorizontalSystem),
    ("Apc",            HouseSystemKind::ApcHouses),
    ("KrusinskiPisaGoelzer", HouseSystemKind::KrusinskiPisaGoelzer),
    ("Sunshine",       HouseSystemKind::SunshineMakranskySolutionTreindl),
    ("PullenSd",       HouseSystemKind::PullenSdSinusoidalDeltaExNeoPorphyry),
    ("PullenSr",       HouseSystemKind::PullenSrSinusoidalRatio),
```

> The corpus `system_code` strings MUST equal the exact `HouseSystem` Rust variant names (e.g. `EqualMidheaven`, `KrusinskiPisaGoelzer`) — the gate maps codes to variants by name in Task 3. Gauquelin is intentionally NOT added here; it is handled in Task 2.

- [ ] **Step 2: Run the generator and verify the new row count**

Run:
```bash
cargo run --manifest-path tools/se-house-reference/Cargo.toml > /tmp/claude-1000/-workspace/851b32f2-16ae-44ca-9ae8-a34736ba306c/scratchpad/cusps_new.csv
grep -c . /tmp/claude-1000/-workspace/851b32f2-16ae-44ca-9ae8-a34736ba306c/scratchpad/cusps_new.csv
cut -d, -f6 /tmp/claude-1000/-workspace/851b32f2-16ae-44ca-9ae8-a34736ba306c/scratchpad/cusps_new.csv | tail -n +2 | sort -u | wc -l
```
Expected: 116 lines (1 header + 115 data rows); 23 distinct system codes.

- [ ] **Step 3: Commit**

```bash
git add tools/se-house-reference/src/main.rs
git commit -m "feat(tool): emit 11 standard target house systems in SE reference generator"
```

---

### Task 2: Generator — Gauquelin 36-sector path via raw FFI

**Files:**
- Modify: `tools/se-house-reference/Cargo.toml` (add `libswisseph-sys` dependency)
- Modify: `tools/se-house-reference/src/main.rs` (write a second `sectors.csv` output)

**Interfaces:**
- Produces: a `sectors.csv` file with header `chart_id,jd_ut,lat_deg,lon_deg,elev_m,system_code,n_sectors,s1..s36` and 5 Gauquelin data rows (one per fixture), each with `n_sectors=36`.

- [ ] **Step 1: Add the raw FFI dependency**

In `tools/se-house-reference/Cargo.toml`, under `[dependencies]`, add:
```toml
libswisseph-sys = "0.1.2"
```

- [ ] **Step 2: Make the generator write both files to an output directory**

Rewrite `tools/se-house-reference/src/main.rs`'s `main` so it takes an output directory as `argv[1]` (default `.`), writes the existing 23-system table to `<dir>/cusps.csv`, and writes Gauquelin sectors to `<dir>/sectors.csv`. Add this Gauquelin helper using the raw FFI with a correctly-sized 37-element buffer:

```rust
/// SE writes 37 doubles for Gauquelin (`G`): index 0 is unused, 1..=36 are the
/// sector cusps. The safe `swisseph` wrapper uses a 13-element buffer and would
/// overflow, so call the raw FFI directly with a 37-element buffer.
fn gauquelin_sectors(jd: f64, lat: f64, lon: f64) -> ([f64; 36], f64, f64) {
    let mut cusps = [0.0f64; 37];
    let mut ascmc = [0.0f64; 10];
    unsafe {
        libswisseph_sys::swe_houses(
            jd, lat, lon, b'G' as i32,
            cusps.as_mut_ptr(), ascmc.as_mut_ptr(),
        );
    }
    let mut sectors = [0.0f64; 36];
    sectors.copy_from_slice(&cusps[1..=36]);
    (sectors, ascmc[0], ascmc[1])
}
```

Write `sectors.csv` with this loop (same `fixtures` slice as `cusps.csv`):

```rust
let mut sectors_out = String::new();
sectors_out.push_str("chart_id,jd_ut,lat_deg,lon_deg,elev_m,system_code,n_sectors");
for i in 1..=36 { sectors_out.push_str(&format!(",s{i}")); }
sectors_out.push('\n');
for &(id, jd, lat, lon, elev) in fixtures {
    let (sectors, _asc, _mc) = gauquelin_sectors(jd, lat, lon);
    sectors_out.push_str(&format!("{id},{jd},{lat},{lon},{elev},Gauquelin,36"));
    for s in sectors { sectors_out.push_str(&format!(",{s:.6}")); }
    sectors_out.push('\n');
}
std::fs::write(format!("{out_dir}/sectors.csv"), sectors_out).expect("write sectors.csv");
```

(Mirror the same `std::fs::write(format!("{out_dir}/cusps.csv"), …)` for the cusps output instead of printing to stdout.)

- [ ] **Step 3: Run the generator into the scratchpad and verify both files**

Run:
```bash
cargo run --manifest-path tools/se-house-reference/Cargo.toml -- /tmp/claude-1000/-workspace/851b32f2-16ae-44ca-9ae8-a34736ba306c/scratchpad
SP=/tmp/claude-1000/-workspace/851b32f2-16ae-44ca-9ae8-a34736ba306c/scratchpad
grep -c . $SP/cusps.csv; grep -c . $SP/sectors.csv
awk -F, 'NR==2{print NF}' $SP/sectors.csv
```
Expected: `cusps.csv` 116 lines; `sectors.csv` 6 lines (1 header + 5 data); each sectors data row has 43 fields (6 meta + `n_sectors` + 36 sectors).

- [ ] **Step 4: Commit**

```bash
git add tools/se-house-reference/Cargo.toml tools/se-house-reference/src/main.rs
git commit -m "feat(tool): emit Gauquelin 36 sectors via raw FFI into sectors.csv"
```

---

### Task 3: Sector corpus parser + manifest sectors-slice parser (TDD)

**Files:**
- Modify: `crates/pleiades-validate/src/house_validation.rs` (add `HouseSectorRow`, `parse_house_sectors`; extend `HouseManifest` + `parse_house_manifest`)

**Interfaces:**
- Produces:
  - `pub(crate) struct HouseSectorRow { chart_id: String, jd_ut: f64, lat_deg: f64, lon_deg: f64, elev_m: f64, system_code: String, sectors: Vec<f64> }`
  - `pub(crate) fn parse_house_sectors(csv: &str) -> Result<Vec<HouseSectorRow>, HouseCorpusError>`
  - `HouseManifest` gains `sector_rows: Option<usize>` and `sector_checksum: Option<u64>`.
- Consumes: existing `HouseCorpusError::MalformedRow { row, line, reason }`.

- [ ] **Step 1: Write failing tests for the sector parser and two-slice manifest**

Add to the `tests` module in `house_validation.rs`:

```rust
const SECTOR_SAMPLE: &str = "chart_id,jd_ut,lat_deg,lon_deg,elev_m,system_code,n_sectors,s1,s2,s3\n\
c0,2451545,0,0,0,Gauquelin,3,10.0,20.0,30.0\n";

#[test]
fn parses_a_well_formed_sector_row() {
    let rows = parse_house_sectors(SECTOR_SAMPLE).expect("valid");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].system_code, "Gauquelin");
    assert_eq!(rows[0].sectors, vec![10.0, 20.0, 30.0]);
}

#[test]
fn rejects_sector_count_mismatch() {
    // n_sectors=3 but only two sector fields present.
    let bad = "chart_id,jd_ut,lat_deg,lon_deg,elev_m,system_code,n_sectors,s1,s2,s3\n\
c0,2451545,0,0,0,Gauquelin,3,10.0,20.0\n";
    assert!(matches!(
        parse_house_sectors(bad),
        Err(HouseCorpusError::MalformedRow { .. })
    ));
}

#[test]
fn parses_two_slice_manifest() {
    let m = "#Reference-Engine: SwissEphemeris 2.10.03\n#CrossCheck-Engine: not-run\n\
slice cusps file=cusps.csv role=cusps rows=115 checksum=111\n\
slice sectors file=sectors.csv role=sectors rows=5 checksum=222\n";
    let parsed = parse_house_manifest(m).expect("valid");
    assert_eq!(parsed.rows, 115);
    assert_eq!(parsed.checksum, 111);
    assert_eq!(parsed.sector_rows, Some(5));
    assert_eq!(parsed.sector_checksum, Some(222));
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p pleiades-validate --lib house_validation::tests::parses_a_well_formed_sector_row 2>&1 | tail -5`
Expected: FAIL — `cannot find function parse_house_sectors`.

- [ ] **Step 3: Implement `HouseSectorRow` + `parse_house_sectors`**

Add near `parse_house_corpus`:

```rust
/// A single parsed row from the variable-length house-sector corpus CSV.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HouseSectorRow {
    pub(crate) chart_id: String,
    pub(crate) jd_ut: f64,
    pub(crate) lat_deg: f64,
    pub(crate) lon_deg: f64,
    pub(crate) elev_m: f64,
    pub(crate) system_code: String,
    /// Variable-length sector cusps, degrees (e.g. 36 for Gauquelin).
    pub(crate) sectors: Vec<f64>,
}

/// Parse the variable-length house-sector corpus CSV. Schema:
/// `chart_id,jd_ut,lat_deg,lon_deg,elev_m,system_code,n_sectors,s1..sN`.
/// Fails closed: malformed, short, or count-mismatched rows return MalformedRow.
pub(crate) fn parse_house_sectors(csv: &str) -> Result<Vec<HouseSectorRow>, HouseCorpusError> {
    let mut rows = Vec::new();
    let mut data_row = 0usize;
    for line in csv.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') || trimmed.is_empty() || trimmed.starts_with("chart_id,") {
            continue;
        }
        data_row += 1;
        let parts: Vec<&str> = trimmed.split(',').collect();
        if parts.len() < 7 {
            return Err(HouseCorpusError::MalformedRow {
                row: data_row, line: line.to_string(),
                reason: format!("expected at least 7 fields, got {}", parts.len()),
            });
        }
        let parse_f64 = |idx: usize, name: &str| -> Result<f64, HouseCorpusError> {
            parts[idx].trim().parse().map_err(|_| HouseCorpusError::MalformedRow {
                row: data_row, line: line.to_string(),
                reason: format!("{name} {:?} is not a valid float", parts[idx]),
            })
        };
        let jd_ut = parse_f64(1, "jd_ut")?;
        let lat_deg = parse_f64(2, "lat_deg")?;
        let lon_deg = parse_f64(3, "lon_deg")?;
        let elev_m = parse_f64(4, "elev_m")?;
        let n_sectors: usize = parts[6].trim().parse().map_err(|_| HouseCorpusError::MalformedRow {
            row: data_row, line: line.to_string(),
            reason: format!("n_sectors {:?} is not a valid usize", parts[6]),
        })?;
        if parts.len() != 7 + n_sectors {
            return Err(HouseCorpusError::MalformedRow {
                row: data_row, line: line.to_string(),
                reason: format!("n_sectors={n_sectors} implies {} fields, got {}", 7 + n_sectors, parts.len()),
            });
        }
        let mut sectors = Vec::with_capacity(n_sectors);
        for (i, field) in parts[7..].iter().enumerate() {
            sectors.push(field.trim().parse().map_err(|_| HouseCorpusError::MalformedRow {
                row: data_row, line: line.to_string(),
                reason: format!("sector field[{}] {:?} is not a valid float", 7 + i, field),
            })?);
        }
        rows.push(HouseSectorRow {
            chart_id: parts[0].trim().to_string(),
            jd_ut, lat_deg, lon_deg, elev_m,
            system_code: parts[5].trim().to_string(),
            sectors,
        });
    }
    Ok(rows)
}
```

- [ ] **Step 4: Extend `HouseManifest` + `parse_house_manifest` for the sectors slice**

In `struct HouseManifest`, add fields:
```rust
    /// Number of data rows in the sectors slice, if present.
    pub(crate) sector_rows: Option<usize>,
    /// FNV-1a-64 checksum of the sectors CSV, if present.
    pub(crate) sector_checksum: Option<u64>,
```

In `parse_house_manifest`, replace the single `if trimmed.starts_with("slice ")` block so it branches on the slice role (the existing `cusps` slice has `role=cusps`; the new one has `role=sectors`):

```rust
        if trimmed.starts_with("slice ") {
            let is_sectors = trimmed.contains("role=sectors");
            for token in trimmed.split_whitespace() {
                if let Some(val) = token.strip_prefix("rows=") {
                    let parsed = val.parse::<usize>().map_err(|_| HouseCorpusError::MalformedManifest {
                        reason: format!("rows value {val:?} is not a valid usize"),
                    })?;
                    if is_sectors { sector_rows = Some(parsed); } else { rows = Some(parsed); }
                } else if let Some(val) = token.strip_prefix("checksum=") {
                    let parsed = val.parse::<u64>().map_err(|_| HouseCorpusError::MalformedManifest {
                        reason: format!("checksum value {val:?} is not a valid u64"),
                    })?;
                    if is_sectors { sector_checksum = Some(parsed); } else { checksum = Some(parsed); }
                }
            }
        }
```

Add `let mut sector_rows: Option<usize> = None;` and `let mut sector_checksum: Option<u64> = None;` next to the existing locals, and add `sector_rows,` / `sector_checksum,` to the returned `HouseManifest { … }`. (The `cusps` slice's `rows`/`checksum` remain required as today; the sector fields stay `Option`.)

- [ ] **Step 5: Run the new tests to verify they pass**

Run: `cargo test -p pleiades-validate --lib house_validation::tests::parses 2>&1 | tail -8`
Expected: PASS for `parses_a_well_formed_sector_row`, `rejects_sector_count_mismatch`, `parses_two_slice_manifest` (and the existing `parses_manifest_fields` still passes).

- [ ] **Step 6: Commit**

```bash
cargo fmt && git add crates/pleiades-validate/src/house_validation.rs
git commit -m "feat(validate): variable-length house-sector parser + two-slice manifest"
```

---

### Task 4: Promote the eleven standard 12-cusp systems (corpus + gate + ceilings + tiers)

This task lands the eleven standard systems end-to-end so every commit stays green: corpus rows, code↔system mapping, family ceilings measured from the data, and the tier flips — together.

**Files:**
- Replace: `crates/pleiades-validate/data/houses-corpus/cusps.csv` (60 → 115 rows)
- Modify: `crates/pleiades-validate/data/houses-corpus/manifest.txt` (row count + checksum)
- Modify: `crates/pleiades-validate/src/house_validation.rs` (`system_for_code`, `code_for_system`, completeness loop, `validated_systems`, count tests)
- Modify: `crates/pleiades-houses/src/thresholds.rs` (family ceilings + doc table)
- Modify: `crates/pleiades-houses/src/catalog/mod.rs` (flip eleven descriptors to `new_release_grade` in BOTH `RELEASE_HOUSE_SYSTEMS` ~955-1162 and `BUILT_IN_HOUSE_SYSTEMS` ~1163-1532)
- Modify: `README.md` (house-systems-pass count)

**Interfaces:**
- Consumes: `parse_house_sectors` (not yet — Task 5), `pleiades_apparent::fnv1a64`.
- Produces: `HouseCorpusReport::validated_systems()` now contains the 12 baseline + 11 standard target systems (23 total).

- [ ] **Step 1: Install the regenerated `cusps.csv`**

```bash
cp /tmp/claude-1000/-workspace/851b32f2-16ae-44ca-9ae8-a34736ba306c/scratchpad/cusps.csv crates/pleiades-validate/data/houses-corpus/cusps.csv
grep -c . crates/pleiades-validate/data/houses-corpus/cusps.csv
```
Expected: 116 lines (1 header + 115 data rows).

- [ ] **Step 2: Extend `system_for_code` and `code_for_system`**

In `house_validation.rs`, add these arms to BOTH functions (matching the exact variant names emitted by the generator):

```rust
        "EqualMidheaven" => Some(HouseSystem::EqualMidheaven),
        "EqualAries" => Some(HouseSystem::EqualAries),
        "Vehlow" => Some(HouseSystem::Vehlow),
        "Sripati" => Some(HouseSystem::Sripati),
        "Carter" => Some(HouseSystem::Carter),
        "Horizon" => Some(HouseSystem::Horizon),
        "Apc" => Some(HouseSystem::Apc),
        "KrusinskiPisaGoelzer" => Some(HouseSystem::KrusinskiPisaGoelzer),
        "Sunshine" => Some(HouseSystem::Sunshine),
        "PullenSd" => Some(HouseSystem::PullenSd),
        "PullenSr" => Some(HouseSystem::PullenSr),
```

For `code_for_system`, add the inverse arms (e.g. `HouseSystem::EqualMidheaven => "EqualMidheaven",` … `HouseSystem::PullenSr => "PullenSr",`).

- [ ] **Step 3: Set provisional generous ceilings and update the manifest with a placeholder checksum**

In `thresholds.rs`, temporarily keep the existing generous `GreatCircle` (15/5) and `SolarArc | Sector | Custom | Unknown` (60/10) ceilings (they are tightened in Step 6). In `manifest.txt`, set the cusps slice to the new row count and a placeholder checksum:
```
slice cusps file=cusps.csv role=cusps rows=115 checksum=0
```

- [ ] **Step 4: Read the real checksum from the failing gate, then pin it**

Run: `cargo run -q -p pleiades-validate -- validate-houses 2>&1 | grep -i checksum`
Expected: `house corpus checksum mismatch: expected 0, got <ACTUAL>`.
Replace `checksum=0` in `manifest.txt` with `checksum=<ACTUAL>`.

- [ ] **Step 5: Add a temporary per-family residual measurement test**

Add this `#[ignore]` test to the `tests` module (it prints the per-family maxima needed to set ceilings):

```rust
#[test]
#[ignore]
fn measure_per_family_residuals() {
    use std::collections::BTreeMap;
    let rows = parse_house_corpus(CORPUS_CSV).unwrap();
    let mut max: BTreeMap<String, (f64, f64)> = BTreeMap::new(); // family -> (cusp, angle)
    for (idx, row) in rows.iter().enumerate() {
        let system = system_for_code(&row.system_code).unwrap();
        let family = pleiades_houses::descriptor(&system).unwrap().formula_family().to_string();
        let snap = recompute_pleiades(idx + 1, row, &system).unwrap();
        let e = max.entry(family).or_insert((0.0, 0.0));
        for (i, &want) in row.cusps.iter().enumerate() {
            e.0 = e.0.max(wrap_arcsec(snap.cusps[i].degrees(), want));
        }
        e.1 = e.1.max(wrap_arcsec(snap.angles.ascendant.degrees(), row.asc));
        e.1 = e.1.max(wrap_arcsec(snap.angles.midheaven.degrees(), row.mc));
    }
    for (fam, (c, a)) in &max { println!("FAMILY {fam}: max_cusp={c:.4} max_angle={a:.4}"); }
}
```

Run: `cargo test -p pleiades-validate --lib measure_per_family_residuals -- --ignored --nocapture 2>&1 | grep FAMILY`
Record each family's `max_cusp` / `max_angle`. **If any promoted system's family shows a max residual too large to defend as release-grade** (sanity bound: cusp ≫ 30″ with no documented pathology), drop that system: remove its rows from `cusps.csv` and its `system_for_code`/`code_for_system` arms, leave it `DescriptorOnly`, and record it as a known gap in Task 7's profile/README edits.

- [ ] **Step 6: Set tightened family ceilings from the measured maxima**

In `thresholds.rs`, set each family ceiling to `ceil(measured_max × 2)` with a 1.0″ floor, for the families now exercised by corpus rows: `Equal`, `WholeSign`, `Quadrant`, `EquatorialProjection`, `GreatCircle`, `SolarArc`, `Sector`. Update the doc-comment measured-maxima table to list all corpus-validated families with their measured values, and remove the "NOT corpus-validated" note for any family now backed by rows. Update `every_family_has_finite_positive_ceilings` if the match arms changed shape.

- [ ] **Step 7: Update the gate's validated-systems count tests**

In `house_validation.rs` tests, change `corpus_report_exposes_twelve_validated_systems` to assert `report.validated_systems().len() == 23` (rename to `…_twenty_three_…`), and update `gate_passes_over_committed_corpus`'s `rows_validated` assertion from `60` to `115`. Update the release-report `summary_line`/`failure_count` assertions in `release_report_expands_to_all_built_in_house_systems` only if the printed counts changed (re-run to get the exact strings).

- [ ] **Step 8: Run the gate to confirm it passes with tightened ceilings**

Run: `cargo run -q -p pleiades-validate -- validate-houses`
Expected: success summary `House gate: 115 rows / … systems, max cusp residual …″, cross-check …`. If a ceiling is exceeded, re-derive it from Step 5's measurement (do not loosen beyond `ceil(max×2)`; if that is indefensible, drop the system per Step 5).

- [ ] **Step 9: Flip the eleven descriptors to `new_release_grade`**

In `catalog/mod.rs`, for each of the eleven systems (`EqualMidheaven`, `EqualAries`, `Vehlow`, `Sripati`, `Carter`, `Horizon`, `Apc`, `KrusinskiPisaGoelzer`, `Sunshine`, `PullenSd`, `PullenSr`), change the descriptor constructor from `HouseSystemDescriptor::new(` to `HouseSystemDescriptor::new_release_grade(` in **both** `RELEASE_HOUSE_SYSTEMS` and `BUILT_IN_HOUSE_SYSTEMS`. Example (Vehlow):

```rust
    // before
    HouseSystemDescriptor::new(
        HouseSystem::Vehlow, "Vehlow Equal", &[/* … */],
        "…", false, None,
    ),
    // after
    HouseSystemDescriptor::new_release_grade(
        HouseSystem::Vehlow, "Vehlow Equal", &[/* … */],
        "…", false, None,
    ),
```

Leave `Albategnius` and `Gauquelin` as `new` (Gauquelin is promoted in Task 5).

- [ ] **Step 10: Update the README house count (Check C token)**

In `README.md` line ~20, change `12 house systems pass the SE numeric gate` to `23 house systems pass the SE numeric gate`. (The audit's Check C looks for the literal token `" 23 house systems pass"`.)

- [ ] **Step 11: Remove the temporary measurement test and run the full house + audit suite**

Delete the `measure_per_family_residuals` test. Then run:
```bash
cargo fmt && cargo test -p pleiades-validate -p pleiades-houses -p pleiades-core 2>&1 | tail -15
cargo run -q -p pleiades-validate -- compat-claims-audit
```
Expected: all tests pass; `compat-claims-audit` reports no violations (Check A sees 23 validated systems matching 23 release-grade descriptors; Check B profile counts match; Check C README token found).

- [ ] **Step 12: Commit**

```bash
cargo clippy --all-targets 2>&1 | tail -3
git add crates/pleiades-validate crates/pleiades-houses README.md
git commit -m "feat(houses): promote 11 standard target house systems to release-grade"
```

---

### Task 5: Promote Gauquelin (sectors.csv + sector gate path + tier)

**Files:**
- Create: `crates/pleiades-validate/data/houses-corpus/sectors.csv`
- Modify: `crates/pleiades-validate/data/houses-corpus/manifest.txt` (add sectors slice)
- Modify: `crates/pleiades-validate/src/house_validation.rs` (embed sectors.csv, gate the sector residuals, add Gauquelin to `validated_systems`)
- Modify: `crates/pleiades-houses/src/catalog/mod.rs` (flip Gauquelin to `new_release_grade`)
- Modify: `README.md` (23 → 24)

**Interfaces:**
- Consumes: `parse_house_sectors`, `HouseManifest::{sector_rows, sector_checksum}` (Task 3).
- Produces: `validated_systems()` containing `HouseSystem::Gauquelin` (24 total).

- [ ] **Step 1: Install sectors.csv and embed it**

```bash
cp /tmp/claude-1000/-workspace/851b32f2-16ae-44ca-9ae8-a34736ba306c/scratchpad/sectors.csv crates/pleiades-validate/data/houses-corpus/sectors.csv
```
Add the embed constant next to `CORPUS_CSV`:
```rust
const CORPUS_SECTORS_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/houses-corpus/sectors.csv"
));
```

- [ ] **Step 2: Add the sectors slice to the manifest with a placeholder checksum**

Append to `manifest.txt`:
```
slice sectors file=sectors.csv role=sectors rows=5 checksum=0
```

- [ ] **Step 3: Wire the sector gate into `validate_house_corpus`**

After the cusps checksum/row-count gates, add the sectors checksum + row-count gate and a residual loop. Insert before the `validated_systems` assembly:

```rust
    // Sectors slice: checksum + row count + per-sector residuals (Gauquelin).
    let sector_checksum = pleiades_apparent::fnv1a64(CORPUS_SECTORS_CSV);
    let manifest_sector_checksum = manifest.sector_checksum.ok_or_else(|| {
        HouseCorpusError::MalformedManifest { reason: "sectors slice checksum missing".into() }
    })?;
    if sector_checksum != manifest_sector_checksum {
        return Err(HouseCorpusError::ChecksumMismatch {
            expected: manifest_sector_checksum, actual: sector_checksum,
        });
    }
    let sector_rows = parse_house_sectors(CORPUS_SECTORS_CSV)?;
    if Some(sector_rows.len()) != manifest.sector_rows {
        return Err(HouseCorpusError::ManifestDrift {
            field: "sector_rows".into(),
            expected: manifest.sector_rows.map(|n| n.to_string()).unwrap_or_default(),
            actual: sector_rows.len().to_string(),
        });
    }
    for (idx, row) in sector_rows.iter().enumerate() {
        let data_row = idx + 1;
        let system = system_for_code(&row.system_code).ok_or_else(|| {
            HouseCorpusError::UnknownSystemCode { row: data_row, code: row.system_code.clone() }
        })?;
        let family = pleiades_houses::descriptor(&system)
            .map(|d| d.formula_family())
            .unwrap_or(pleiades_houses::HouseFormulaFamily::Unknown);
        let ceiling = pleiades_houses::thresholds::house_family_ceiling(family);
        let snapshot = recompute_pleiades(data_row, &HouseCorpusRow {
            chart_id: row.chart_id.clone(), jd_ut: row.jd_ut, lat_deg: row.lat_deg,
            lon_deg: row.lon_deg, elev_m: row.elev_m, system_code: row.system_code.clone(),
            cusps: [0.0; 12], asc: 0.0, mc: 0.0,
        }, &system)?;
        for (i, &want) in row.sectors.iter().enumerate() {
            let got = snapshot.cusps[i].degrees();
            let resid = wrap_arcsec(got, want);
            if resid > max_cusp_residual_arcsec { max_cusp_residual_arcsec = resid; }
            if resid > ceiling.cusp_arcsec {
                return Err(HouseCorpusError::CeilingExceeded {
                    row: data_row, system: row.system_code.clone(), cusp: i + 1,
                    got, want, residual_arcsec: resid, ceiling_arcsec: ceiling.cusp_arcsec,
                });
            }
        }
    }
```

Then extend the `validated_systems` assembly to also include resolved sector-row systems:
```rust
    for row in &sector_rows {
        if let Some(sys) = system_for_code(&row.system_code) {
            if !validated_systems.contains(&sys) { validated_systems.push(sys); }
        }
    }
```

Add the `Gauquelin` arms to `system_for_code` (`"Gauquelin" => Some(HouseSystem::Gauquelin),`) and `code_for_system` (`HouseSystem::Gauquelin => "Gauquelin",`).

- [ ] **Step 4: Read and pin the sectors checksum**

Run: `cargo run -q -p pleiades-validate -- validate-houses 2>&1 | grep -i checksum`
Expected: a mismatch reporting the sectors `got` value. Replace `checksum=0` in the sectors slice with that value. (If the cusps gate fails first, the printed mismatch is for cusps — but that was pinned in Task 4; the sectors mismatch should be the one shown.)

- [ ] **Step 5: Measure the Sector-family residual for Gauquelin and confirm the ceiling holds**

Run: `cargo run -q -p pleiades-validate -- validate-houses`
Expected: success. If Gauquelin's sector residual exceeds the `Sector` ceiling set in Task 4 (which was derived from Pullen SD/SR), re-measure the Sector family across both Pullen and Gauquelin rows and set the ceiling to `ceil(max × 2)`; if Gauquelin alone is indefensible, leave it `DescriptorOnly`, remove sectors.csv/its slice, and record it as a known gap instead (do not proceed to Step 6 for Gauquelin).

- [ ] **Step 6: Promote Gauquelin and bump the README count**

In `catalog/mod.rs`, flip Gauquelin from `new` to `new_release_grade` in both `RELEASE_HOUSE_SYSTEMS` and `BUILT_IN_HOUSE_SYSTEMS`. In `README.md`, change `23 house systems pass` to `24 house systems pass`.

- [ ] **Step 7: Update the validated-systems count test**

In `house_validation.rs` tests, change the validated-systems count assertion from `23` to `24` and assert membership of `HouseSystem::Gauquelin`.

- [ ] **Step 8: Run the full house + audit suite**

```bash
cargo fmt && cargo test -p pleiades-validate -p pleiades-houses -p pleiades-core 2>&1 | tail -15
cargo run -q -p pleiades-validate -- compat-claims-audit
```
Expected: all green; audit reports no violations (24 validated ⟺ 24 release-grade ⟺ profile ⟺ README token ` 24 house systems pass`).

- [ ] **Step 9: Commit**

```bash
cargo clippy --all-targets 2>&1 | tail -3
git add crates/pleiades-validate crates/pleiades-houses README.md
git commit -m "feat(houses): promote Gauquelin to release-grade via variable-length sector gate"
```

---

### Task 6: Strict high-latitude rejection for the four latitude-sensitive systems

**Files:**
- Modify: `crates/pleiades-validate/src/house_validation.rs` (strict-rejection block in `validate_house_corpus`; a dedicated test)

**Interfaces:**
- Consumes: `calculate_houses`, `HighLatitudePolicy::SwissEphemerisFallback`, the promoted descriptors.

- [ ] **Step 1: Write a failing test asserting strict rejection for the four systems**

Add to the `tests` module:

```rust
#[test]
fn promoted_latitude_sensitive_systems_reject_above_bound() {
    use pleiades_core::{calculate_houses, HouseRequest, HouseSystem};
    for system in [HouseSystem::Horizon, HouseSystem::Apc,
                   HouseSystem::KrusinskiPisaGoelzer, HouseSystem::Sunshine] {
        for lat in [70.0_f64, 80.0] {
            let observer = ObserverLocation::new(
                Latitude::from_degrees(lat), Longitude::from_degrees(0.0), Some(0.0));
            let req = HouseRequest::new(gate_instant(), observer, system.clone());
            assert!(calculate_houses(&req).is_err(),
                "{system:?} at {lat}° must be rejected by the strict high-latitude policy");
        }
    }
}
```

- [ ] **Step 2: Run it**

Run: `cargo test -p pleiades-validate --lib promoted_latitude_sensitive_systems_reject_above_bound 2>&1 | tail -8`
Expected: PASS (the descriptors already declare `max_abs_latitude_deg = Some(66.0)`). If it FAILS for any system, that system's high-latitude handling differs — record it and gate only the systems that reject, documenting the rest in the constraint notes.

- [ ] **Step 3: Fold the four systems into the gate's strict-rejection block**

In `validate_house_corpus`, the strict-rejection loop currently iterates `baseline_house_systems()`. Extend it to also cover the promoted latitude-sensitive systems by iterating `built_in_house_systems()` filtered to `claim_tier == ReleaseGradeNumeric && max_abs_latitude_deg.is_some()` instead of `baseline_house_systems()`, keeping the existing Strict-rejection assertion and the `SwissEphemerisFallback`-succeeds assertion. For the fallback-equals-Porphyry assertion, keep it only for the baseline quadrant systems (whose documented fallback is Porphyry); for the new systems assert only that `SwissEphemerisFallback` returns `Ok` (their documented fallback target is verified here, not assumed). Add a short code comment recording which systems use which fallback.

- [ ] **Step 4: Run the gate and the strict-rejection test**

Run:
```bash
cargo test -p pleiades-validate --lib house_validation 2>&1 | tail -8
cargo run -q -p pleiades-validate -- validate-houses
```
Expected: all house tests pass; gate succeeds.

- [ ] **Step 5: Commit**

```bash
cargo fmt && git add crates/pleiades-validate/src/house_validation.rs
git commit -m "test(validate): strict high-latitude rejection for promoted latitude-sensitive systems"
```

---

### Task 7: Final consistency sweep — profile, known gaps, release gates

**Files:**
- Modify: `crates/pleiades-core/src/compatibility/*` only if a profile test asserts an exact release-grade count or known-gap list (verify first)
- Modify: `PLAN.md` (mark Phase 6 house-system promotion done / record any known gaps)
- Possibly: `crates/pleiades-validate/src/compatibility/mod.rs` known-gap text, if any system was left `DescriptorOnly`

**Interfaces:**
- Consumes: nothing new; this task verifies cross-surface consistency.

- [ ] **Step 1: Run the full release gate set**

Run:
```bash
cargo run -q -p pleiades-validate -- compat-claims-audit
mise run release-smoke 2>&1 | tail -20
```
Expected: audit clean; release-smoke passes all numeric gates (house, ayanamsa, apparent, topocentric, corpus) plus the overclaim audit. Fix any count/string assertion the smoke surfaces by updating the asserting test to the new value.

- [ ] **Step 2: Record outcome in PLAN.md**

Update `PLAN.md`'s Phase 6 / "Important current limits" / status line to state that the target house-system set is now release-grade (24 of 25 built-in; `Albategnius` remains an out-of-catalog descriptor-only built-in), plus any target system left as a known gap by the Step-5 fail-safes. Keep README, the compatibility profile, and PLAN.md mutually consistent.

- [ ] **Step 3: Full workspace test + lint**

Run:
```bash
cargo fmt --check && cargo clippy --all-targets 2>&1 | tail -3
cargo test --workspace 2>&1 | tail -20
```
Expected: clean; all tests pass.

- [ ] **Step 4: Commit**

```bash
git add PLAN.md crates README.md
git commit -m "docs: mark Phase 6 house-system release-grade promotion done; align surfaces"
```

---

## Self-Review notes

- **Spec coverage:** Generator 11 systems (T1) + Gauquelin FFI (T2); variable-length `sectors.csv` schema + parser (T3); gate wiring + `validated_systems` (T4/T5); per-family ceilings via measured maxima with fail-safe (T4 S5-6, T5 S5); bidirectional tier promotion + profile + README (T4/T5/T7); strict high-latitude rejection folded in (T6); known-gap reporting (T4 S5, T5 S5, T7). All spec sections map to a task.
- **Bidirectional-audit safety:** corpus rows, code mapping, ceilings, and tier flips for each system land in the same task (T4 for the eleven, T5 for Gauquelin), so `compat-claims-audit` and the `real_catalogs_pass_check_a` unit test are green at every commit boundary.
- **Type consistency:** `parse_house_sectors`, `HouseSectorRow`, `HouseManifest::{sector_rows, sector_checksum}`, and `CORPUS_SECTORS_CSV` are defined in T3/T5 and consumed by name in T5's gate code; `system_for_code`/`code_for_system` arm strings match the generator's `system_code` variant names exactly.
