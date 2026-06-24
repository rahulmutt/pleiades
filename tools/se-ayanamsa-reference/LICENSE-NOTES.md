# License Notes — se-ayanamsa-reference

## Dependency licenses

- `swisseph` 0.1.1: `license = "AGPL-3.0-only"`
- `libswisseph-sys` 0.1.2: `license = "AGPL-3.0-only"`, `links = "libswisseph"` (statically compiles Swiss Ephemeris C source via bindgen; requires libclang at build time)

## Swiss Ephemeris data files

The ayanamsa reference values are produced by `swe_get_ayanamsa` after `swe_set_sid_mode`. For the modes used here (Fagan/Bradley, Lahiri, Raman, Krishnamurti, and True Citra / SE code 27) the computation is analytic and required no `.se1` ephemeris data files or `sefstars.txt` fixed-star catalog at generation time. No SE data files are used, bundled, or distributed by this tool.

## Usage statement

This crate is **verification-only**. It is:

- Excluded from the workspace via `[workspace] exclude` in the root `Cargo.toml` (constraint C1), and carries its own `Cargo.lock` under `tools/` so the AGPL `-sys` package never enters the workspace lockfile.
- Never a dependency of any shipping crate.
- Never distributed as part of any release artifact.
- Its output (numeric ayanamsa reference values, stored as CSV in `crates/pleiades-validate/data/ayanamsa-corpus/`) is committed; the AGPL binding itself is never distributed.

AGPL-3.0-only is acceptable for this use case because the harness is not distributed and exists solely to produce reference values for offline validation. No escalation required.
