# License Notes — se-house-reference

## Dependency licenses

- `swisseph` 0.1.1: `license = "AGPL-3.0-only"`
- `libswisseph-sys` 0.1.2: `license = "AGPL-3.0-only"`, `links = "libswisseph"` (statically compiles Swiss Ephemeris C source via bindgen; requires libclang at build time)

## Swiss Ephemeris data files

Not applicable. House cusp calculations (`swe_houses`) are purely analytic — they require no `.se1` ephemeris data files. No SE data files are used, bundled, or distributed by this tool.

## Usage statement

This crate is **verification-only**. It is:

- Excluded from the workspace via `[workspace] exclude` in the root `Cargo.toml` (constraint C1).
- Never a dependency of any shipping crate.
- Never distributed as part of any release artifact.
- Its output (numeric house cusps, stored as CSV in `crates/pleiades-validate/data/`) is committed; the AGPL binding itself is never distributed.

AGPL-3.0-only is acceptable for this use case because the harness is not distributed and exists solely to produce reference values for offline validation. No escalation required.
