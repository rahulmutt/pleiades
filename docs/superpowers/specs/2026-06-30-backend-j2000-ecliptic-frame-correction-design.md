# Backend J2000 Ecliptic Frame Correction — Design

**Date:** 2026-06-30
**Branch:** `feat/equatorial-declination-output` (same branch, sequenced before the
equatorial RA/Dec feature's remaining tasks — see Sequencing).
**Status:** Approved design; implementation plan to follow.
**Investigation:** `/.superpowers/sdd/backend-frame-investigation.md` (read-only
recon that confirmed root cause, blast radius, and code-vs-artifact verdict).

## Problem

The first-party packaged backend (`pleiades-data::PackagedDataBackend`) emits a
**frame-inconsistent** geocentric ecliptic position for every kernel-fit body
(Sun, Moon, all planets, Pluto): **longitude is J2000** but **latitude is
of-date**. This was discovered while building the apparent equatorial (RA/Dec)
feature — the new equatorial gate showed ~44–48″ declination errors at the 1900
and 2100 epochs.

**Confirmed root cause (single line):** `icrf_to_ecliptic`
(`crates/pleiades-jpl/src/spk/chain.rs:148`) rotates the ICRF/J2000-equatorial
vector about the X-axis by `instant.mean_obliquity()` — the **of-date** mean
obliquity — instead of the fixed **J2000** obliquity ε₀. An X-rotation leaves the
X-axis (the J2000 mean equinox) fixed, so longitude stays referred to J2000;
but latitude is measured from the tilted (of-date) plane, so it comes out
of-date. `icrf_velocity_to_ecliptic` (`chain.rs:113`) has the identical bug.

**Evidence (direct backend probe, no pipeline):** the Sun's raw packaged ecliptic
latitude at JD 2415025.5 (1900) is **+1.79″** (≈0, of-date — the Sun lies on the
ecliptic of date), whereas a true J2000 frame would carry **~−45″** (≈46″·sin λ).
Its longitude (286.65°) is J2000. Confirmed on Sun, Moon, Mars, Jupiter.

**Why it was invisible:** the of-date latitude is **baked into the committed
bytes** (`packaged-artifact.bin` and the de440 corpus CSVs). The accuracy gate
compares the artifact against the de440 holdout corpus — but **both sides share
the same of-date frame**, so the gate measures fit quality, not frame
correctness (it is bug-blind). A prior task added a band-aid in
`pleiades-apparent/src/precession.rs:71-81` (`lat + sin(λ)·Δε`) that cancels the
double-counted tilt at the *output*; its root-cause comment is wrong (the 3-step
precession primitive is rigorously correct for a genuine J2000 input). This
design fixes the source and removes the band-aid.

## Goal

Every first-party backend emits a consistent **J2000 ecliptic** frame (longitude
**and** latitude) at the boundary, so the apparent pipeline's single J2000→date
precession yields correct of-date results, and the J2000-at-the-backend-boundary
contract — currently silently violated — becomes true and **provably enforced**.

## Non-goals

- No change to the artifact binary *format* (`ARTIFACT_VERSION` stays 7; only
  latitude-channel **values** move on regeneration).
- No change to VSOP87 (already consistent J2000) or the JPL snapshot fixture
  (already J2000).
- The equatorial RA/Dec feature's remaining tasks (SE gate, CLI wiring, docs)
  are resumed *after* this correction, not redesigned here.

## Global Constraints

- **Trusted reference for "J2000 frame correct":** an **independent** J2000
  source, never the regenerated corpus (which shares the fixed code and is thus
  self-consistent by construction). VSOP87 (documented J2000, separate code path)
  for Sun + planets + Pluto; the J2000 snapshot fixture for the Moon.
- **Regeneration is deterministic and byte-identity-gated** via
  `PLEIADES_DE_KERNEL` (`/workspace/.kernels/de440.bsp`). The gated tests
  (`pleiades-data/tests/artifact_regen.rs`, `pleiades-jpl/tests/corpus_regen.rs`)
  are the reproducibility guard: regenerate, commit, confirm byte-identity.
- **ε₀ single source of truth:** one shared `OBLIQUITY_J2000_DEG`
  (= 23.439_291_111_111_11°, currently in `pleiades-apparent/src/precession.rs:15`)
  used by the SPK reduction, precession, and `Instant::mean_obliquity` at T=0 —
  no duplicated literal.
- **Pole/wrap conventions** for any RA/Dec or longitude residual carry over from
  the equatorial gates (cos(Dec)-weighted, wrap-aware longitude; signed Dec/lat).
- **TDD + frequent commits**; one test-first cycle per task; commit per task.

## Architecture (the correction, by component)

### A. ε₀ code fix (`pleiades-jpl/src/spk/chain.rs`)
Change `icrf_to_ecliptic` (line 148) and `icrf_velocity_to_ecliptic` (line 113)
to rotate by the fixed J2000 obliquity ε₀ instead of `instant.mean_obliquity()`.
This makes the reduction a true ICRF/J2000-equatorial → J2000-ecliptic rotation,
consistent in both components. A unit test asserts the J2000 reduction directly
(e.g. Sun/Moon/planet at 1900 yields the J2000-frame latitude, not ~0).

### B. Artifact + corpus regeneration
Regenerate `packaged-artifact.bin` and the de440 corpus CSVs
(`pleiades-jpl/data/corpus/*.csv`: holdout, interior, boundary, fast-clusters) in
one pass from the de440 kernel, so the accuracy gate's truth set moves into the
corrected frame together with the artifact. Re-pin byte-identity / checksum
goldens (`codec.rs`, `coverage/profile.rs`, `coverage/regen.rs`) and the
committed accuracy-baseline summary string. Eros (snapshot-fit) is already J2000
and does not change.

### C. Keystone: independent frame-correctness gate (NEW)
A new validation gate cross-checks the regenerated packaged backend's **raw**
J2000 ecliptic latitude against an **independent** J2000 source at far epochs
(1900 and 2100, where Δε is large enough to discriminate frames): VSOP87 for
Sun + planets + Pluto, the J2000 snapshot fixture for the Moon, within a tight
tolerance. This is the gate that distinguishes "correct J2000 frame" from
"self-consistently wrong" — it is the permanent regression guard derived from the
discovery probe, and it is non-negotiable.

### D. Band-aid revert (`pleiades-apparent/src/precession.rs:71-81`)
Remove the `lat_corrected = lat + sin(λ)·Δε` term, restoring the rigorous 3-step
precession primitive. With a true-J2000 backend latitude this is now correct;
end-to-end proof is the equatorial-goldens (JPL Horizons) gate passing **without**
the band-aid.

### E. ELP → J2000 (`pleiades-elp`)
ELP is a Meeus Ch.47 *of-date* lunar series (consistent of-date in both
components — a separate violation of the J2000-boundary contract). Add a
**date→J2000 precession step** in the ELP backend output so it emits J2000 at the
boundary; the apparent pipeline's forward J2000→date precession then round-trips
to the correct of-date Moon (round-trip error <0.001″). This also corrects a
**latent double-precession** for any ELP-sourced Moon that passes through the
apparent pipeline. *(Rejected alternative: mark ELP "already of-date" and
special-case the pipeline to skip its precession — breaks the uniform-boundary
goal with per-backend branching.)*

### F. Live-SPK ↔ snapshot reconciliation
The ε₀ fix automatically makes the live `SpkBackend` J2000, matching the
already-J2000 snapshot fixture. The existing cross-check only runs at J2000
(where ε_date ≈ ε₀, so it is frame-blind); add/extend a **non-J2000-epoch**
cross-check so the live-SPK = snapshot consistency is locked going forward.

### G. Tolerance recalibration + claims
- Topocentric latitude tolerance: currently 90″ with a stale "data ceiling"
  rationale that was *absorbing this bug*; tighten toward the true post-fix
  residual (~10″) with documented headroom, and correct the rationale.
- Re-run the equatorial Sun-Dec 6″ gate (tightest in the suite; Dec derives from
  the corrected latitude) and the lilith 80″ ceiling; re-pin every edited
  goldens-CSV checksum.
- Update metadata/claims so the "J2000 at the backend boundary" posture is now
  true and asserted (it is currently silently violated). The backend-boundary
  mean-obliquity *equatorial* strings remain true and unchanged.

## Acceptance criteria

1. `icrf_to_ecliptic` / `icrf_velocity_to_ecliptic` use a fixed ε₀; unit test
   proves a J2000-frame latitude at a far epoch.
2. Regenerated artifact + corpus committed; `PLEIADES_DE_KERNEL`-gated
   byte-identity tests pass; accuracy gate green against the regenerated corpus.
3. **Keystone gate green:** packaged raw J2000 latitude matches VSOP87 (Sun +
   planets) and the snapshot fixture (Moon) at 1900/2100 within tight tolerance.
4. Band-aid removed; equatorial-goldens (Horizons) gate green end-to-end without
   it.
5. ELP emits J2000 at the boundary (tested); no double-precession for ELP Moon.
6. Live-SPK = snapshot at a non-J2000 epoch (tested).
7. Tolerances recalibrated with corrected rationales; all goldens re-pinned;
   `cargo test --workspace` green; release gate (`run_all_numeric_gates`) green.

## Blast radius (files that move)

Code: `pleiades-jpl/src/spk/chain.rs`, `pleiades-types` (shared ε₀),
`pleiades-apparent/src/precession.rs` (revert), `pleiades-elp` (precess-back),
new gate in `pleiades-validate`. Data: `packaged-artifact.bin`,
`pleiades-jpl/data/corpus/*.csv`. Goldens/tolerances: accuracy baseline
(`thresholds.rs:60`, `accuracy_baseline.rs:653` + summary string), topocentric
(`topocentric-goldens.csv` + `topocentric_validation.rs` 90″ tol + rationale +
checksum), equatorial Sun-Dec gate (re-run + re-pin), lilith ceiling (re-run),
byte/checksum goldens (`codec.rs`, `coverage/profile.rs`, `coverage/regen.rs`),
mean-mode lookup tests (`pleiades-data/src/tests/lookup.rs` — move with regen),
non-J2000 SPK cross-check (`pleiades-jpl/.../cross_check_tests.rs`). Claims/docs:
J2000-boundary posture strings.

## Task decomposition (sequenced, same branch)

- **B1** — ε₀ fix in `icrf_to_ecliptic` + `icrf_velocity_to_ecliptic`; shared ε₀
  constant; unit test for J2000-frame latitude. (Coupled to B2: the code fix
  invalidates the committed bytes.)
- **B2** — Regenerate `packaged-artifact.bin` + de440 corpus CSVs from the
  kernel; re-pin byte/checksum goldens + accuracy-baseline summary; confirm gated
  byte-identity tests pass.
- **B3** — Keystone frame-correctness gate (VSOP87 + snapshot cross-check).
- **B4** — Revert the precession band-aid; equatorial-goldens (Horizons) gate
  green end-to-end.
- **B5** — ELP → J2000 (date→J2000 precession in the ELP backend) + tests.
- **B6** — Live-SPK ↔ snapshot non-J2000 cross-check test.
- **B7** — Tolerance recalibration (topocentric, equatorial re-run, lilith) +
  re-pin + claims/docs (J2000-boundary posture).

Then **resume the equatorial RA/Dec feature**: close Task 3's review (band-aid now
gone), Task 4 (SE convention-parity gate), Task 5 (CLI + release-gate wiring),
Task 6 (claims/README/PLAN/follow-ups).

## Risks

- **Self-consistent regeneration masking residual error** — mitigated by the
  keystone independent gate (C); without it, the whole effort could ship a
  self-consistent wrong frame.
- **Regeneration non-determinism** — mitigated; byte-identity-gated.
- **Tightening tolerances surfaces other latent issues** — acceptable and
  intended; investigate any that appear rather than re-loosening.
- **ELP precess-back interaction with the pipeline** — verified by an end-to-end
  Moon-of-date assertion against an independent reference.
