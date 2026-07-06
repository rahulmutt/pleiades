# pleiades-fict

Fictitious/hypothetical body backend for the pleiades astrology workspace.

Computes the Swiss-Ephemeris default `seorbel.txt` fictitious bodies (SE numbers
40–58: the Uranian/Hamburg planets, Transpluto, Vulcan, White Moon, Waldemath,
and the historical pre-discovery Neptune/Pluto predictions) from committed
osculating orbital elements (transcribed from Swiss Ephemeris `seorbel.txt`)
via an unperturbed Kepler orbit (Kepler-third-law mean motion, except the
T-term bodies), rotated to the J2000 mean ecliptic. Heliocentric-source bodies
are geocentricized via the packaged Sun source; the two geocentric-orbit
bodies (White Moon, Waldemath) are served directly.

## Claim tier

These bodies are *definitional*: correctness means parity with SE's
`seorbel.txt`-driven output, not agreement with observation or a perturbed
theory. Evidence: unperturbed Kepler orbit from committed `seorbel.txt`
elements; SE `swe_calc` parity via `validate-fictitious` (bodies ≥ 40).

Enforcement: the fail-closed, two-tier `validate-fictitious` gate (CLI aliases
`validate-fictitious`, `fictitious-gate`), wired into the release gate set,
checks every body against a committed 570-row Swiss-Ephemeris corpus
(checksum-guarded via fnv1a64, pinned by row count).

## Known limitation: Nibiru

All 18 non-Nibiru bodies reach sub-arcsecond SE parity (measured max longitude
residual 0.459″, `NeptuneLeverrier`). **Nibiru (SE body 49) is the sole
exception:** its `seorbel.txt` reference equinox is ~370 AD, roughly 1630
years before J2000 — well outside the accurate range of the IAU-1976
ecliptic-precession extrapolation used to rotate its elements to the J2000
frame. As a result Nibiru carries a larger, still arcsecond-level residual
(measured max longitude 1.262″) and a documented per-body gate carve-out
(its own wider ceiling) rather than being allowed to inflate the shared
non-Nibiru ceilings.
