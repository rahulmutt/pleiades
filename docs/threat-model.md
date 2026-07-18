# Threat Model — pleiades workspace

Lightweight threat model per the devkit `security-practices` skill.
Revisit whenever a trust boundary changes (see "Revisit when" below).

## Assets

1. **Integrity of the published `pleiades-*` crates** — consumers compute
   charts from them; a corrupted release is the worst outcome.
2. **Integrity of committed reference corpora and the packaged artifact** —
   all validation gates derive their authority from them.
3. **Release credentials** — `RELEASE_PLZ_TOKEN` and `CARGO_REGISTRY_TOKEN`
   (GitHub Actions secrets).
4. **Contributor machines** running dev tasks.

## Assumed adversaries

- An opportunistic supply-chain attacker (malicious or compromised
  crates.io dependency, typosquat).
- An attacker with a compromised contributor account or PR flow, attempting
  to land a secret, a corrupted corpus row, or a malicious dependency.
- A malicious data source: crafted kernel/JSON/CSV fed to ingestion, or an
  intercepted Horizons endpoint.

Not assumed: targeted nation-state attackers, physical access.

## Trust boundaries and their controls

| # | Boundary | What crosses it | Controls |
| --- | --- | --- | --- |
| 1 | Data ingestion (`pleiades-jpl::ingest`, kernel/corpus loading, `pleiades-compression` decode) | Untrusted bytes | Parsers return structured errors and must never panic (AGENTS.md rule); corpora are checksum-pinned with fail-closed gates; fuzzed continuously by four cargo-fuzz targets (`spk_kernel`, `compression_framing`, `compression_payload`, `ingest_corpus`) on a daily cron; findings are fixed with regression tests in the blocking tier |
| 2 | `horizons-fetch` (default-off feature) | HTTPS to JPL Horizons | rustls with pinned webpki-roots trust anchors, pure-Rust TLS; feature is default-off so consumers never get network unless they opt in |
| 3 | CI release pipeline | `RELEASE_PLZ_TOKEN`, `CARGO_REGISTRY_TOKEN` | Secrets injected at runtime, never committed (gitleaks pre-commit + CI + history scan: `mise run secrets`); publishing only from `main` via release-plz |
| 4 | Inbound supply chain (third-party crates) | Code we did not write | `mise run deny` (RustSec advisories, license allowlist, bans, crates.io-only sources); committed `Cargo.lock`; Renovate cadence updates gated by CI; AGENTS.md minimal-dependency policy |
| 5 | Outbound supply chain (published crates) | Our code into consumers | Fail-closed `release-gate` before publish; per-crate tags; `docs/release-reproducibility.md` |

## Deliberate choices

- **SAST:** clippy with `-D warnings` fills the SAST role for Rust. No
  separate SAST tool — the workspace has no web/injection surface. Revisit
  if a service or web boundary appears.
- **No secrets in code paths:** the only credentialed operations live in CI.

## Out of scope

- Denial of service / resource exhaustion on a local computation library —
  callers own their compute budgets.
- Confidentiality of chart inputs — the library runs in the caller's
  process and transmits nothing.
- OS-level or physical compromise of developer machines.

## Revisit when

- A new network path or credentialed operation is added.
- User-supplied files are accepted in a new format.
- Any component becomes a hosted service.
