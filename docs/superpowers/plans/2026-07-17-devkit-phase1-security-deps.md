# Devkit Phase 1: Security & Dependency Hygiene Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add SCA (cargo-deny), secret scanning (gitleaks), cadence-based dependency updates (Renovate), and a committed threat model, per Phase 1 of `docs/superpowers/specs/2026-07-17-devkit-adoption-design.md`.

**Architecture:** All tools are mise-pinned and exposed as named mise tasks; CI picks them up by adding the task names to the existing `[tasks.ci]` depends list (ci.yml already runs `mise run ci`). The threat model is a standalone doc referenced (not duplicated) from AGENTS.md.

**Tech Stack:** mise, cargo-deny 0.20.2, gitleaks 8.30.1, Renovate (GitHub App), GitHub Actions.

## Global Constraints

- Every tool version is pinned exactly in `mise.toml` — never a floating version.
- mise-first, devenv.nix fallback only (no devenv.nix change is needed in this phase).
- The `release-gate` mise task's semantics are untouched.
- Workspace distributes `MIT OR Apache-2.0`; the dependency license allowlist admits permissive licenses only. Any copyleft or unknown license reported by cargo-deny → STOP and ask the user; never allowlist it unilaterally.
- If the gitleaks full-history scan reports any finding → STOP and surface it to the user before wiring any gate. Never silently baseline a finding.
- Commit types must not trigger release-plz version bumps: use `chore(...)` / `docs(...)`, never `feat`/`fix`.
- Follow AGENTS.md: smallest change that fully solves the problem.

---

### Task 1: cargo-deny SCA gate

**Files:**
- Modify: `mise.toml` (add tool pin, `[tasks.deny]`, extend `[tasks.ci]` depends)
- Create: `deny.toml`

**Interfaces:**
- Consumes: nothing (first task).
- Produces: mise task `deny` (runs `cargo deny check`). Phase 2's nightly workflow will also invoke `mise run deny`; `[tasks.ci]` gains `"deny"` in `depends`.

- [ ] **Step 1: Pin cargo-deny in mise.toml**

In `mise.toml`, extend the `[tools]` table:

```toml
[tools]
rust = { version = "1.97.1", components = "rustfmt,clippy" }
"cargo:cargo-release" = "1.1.3"
"cargo:git-cliff" = "2.13.1"
"cargo:release-plz" = "0.3.160"
"cargo:cargo-deny" = "0.20.2"
```

- [ ] **Step 2: Install and verify the pinned version**

Run: `mise install && mise x -- cargo deny --version`
Expected: `cargo-deny 0.20.2`

- [ ] **Step 3: Run the check config-less to see it fail**

Run: `mise x -- cargo deny check licenses`
Expected: FAIL (non-zero exit) with `error[rejected]` diagnostics — with no `deny.toml`, the allowlist is empty, so every dependency license is rejected. This proves the gate actually gates.

- [ ] **Step 4: Write deny.toml**

Create `deny.toml` at the repo root:

```toml
# cargo-deny configuration — SCA gate for the pleiades workspace.
# Run via `mise run deny`. See docs/threat-model.md (inbound supply chain).

[graph]
all-features = true

[advisories]
# RustSec advisories are fail-closed; yanked crates are refused too.
yanked = "deny"

[licenses]
# The workspace distributes MIT OR Apache-2.0, so only permissive licenses
# are admitted. Never add a copyleft or unknown license here without an
# explicit maintainer decision.
allow = [
  "MIT",
  "Apache-2.0",
  "Apache-2.0 WITH LLVM-exception",
  "ISC",
  "BSD-2-Clause",
  "BSD-3-Clause",
  "Unicode-3.0",
  "Zlib",
  "CDLA-Permissive-2.0", # webpki-roots trust-anchor data
]

[bans]
# Duplicate major versions are a bloat signal, not (yet) an error.
multiple-versions = "warn"
wildcards = "deny"

[sources]
# Pure crates.io supply chain — no git or alternate-registry dependencies.
unknown-registry = "deny"
unknown-git = "deny"
```

- [ ] **Step 5: Add the mise task and wire it into ci**

In `mise.toml`, add immediately after the `[tasks.audit]` block:

```toml
[tasks.deny]
run = "cargo deny check"
```

and change the `[tasks.ci]` depends line from:

```toml
depends = ["fmt", "lint", "test-full", "docs", "audit", "package-check", "release-smoke", "claims-audit"]
```

to:

```toml
depends = ["fmt", "lint", "test-full", "docs", "audit", "package-check", "release-smoke", "claims-audit", "deny"]
```

- [ ] **Step 6: Run the gate and reconcile the allowlist**

Run: `mise run deny`
Expected: PASS (`advisories ok`, `bans ok`, `licenses ok`, `sources ok`).

If `licenses` reports a rejection for a license **not** in the Step 4 list: check it is a permissive OSI license of a real transitive dependency (`mise x -- cargo deny list | grep <license>` shows which crate carries it). If permissive, append it to `allow` with an inline comment naming the crate. If copyleft or unrecognizable, STOP and ask the user (Global Constraints).
If `advisories` reports an existing CVE in the current tree: STOP and surface it to the user with the advisory ID — do not add an `ignore` entry unilaterally.

- [ ] **Step 7: Commit**

```bash
git add mise.toml deny.toml
git commit -m "chore(security): add cargo-deny SCA gate (advisories, licenses, bans, sources)"
```

---

### Task 2: gitleaks secret scanning

**Files:**
- Modify: `mise.toml` (add tool pin, `[tasks.secrets]`, extend `[tasks.ci]` depends)
- Create: `.githooks/pre-commit`
- Modify: `.github/workflows/ci.yml` (checkout `fetch-depth: 0` so CI scans full history)

**Interfaces:**
- Consumes: `[tasks.ci]` depends list as left by Task 1.
- Produces: mise task `secrets` (runs `gitleaks git .`), committed hook `.githooks/pre-commit` (activated by `git config core.hooksPath .githooks` — Phase 4 documents this in the README quickstart).

- [ ] **Step 1: Pin gitleaks in mise.toml**

In `mise.toml` `[tools]`, add after the `"cargo:cargo-deny"` line:

```toml
gitleaks = "8.30.1"
```

- [ ] **Step 2: Install and verify**

Run: `mise install && mise x -- gitleaks version`
Expected: `8.30.1`

- [ ] **Step 3: Full-history scan**

Run: `mise x -- gitleaks git .`
Expected: exit 0 with `no leaks found`.
If ANY finding is reported: STOP. Present the finding (file, commit, rule) to the user and wait for a rotation/handling decision before continuing this task (Global Constraints).

- [ ] **Step 4: Add the mise task and wire it into ci**

In `mise.toml`, add immediately after the `[tasks.deny]` block:

```toml
[tasks.secrets]
run = "gitleaks git ."
```

and extend the `[tasks.ci]` depends list (as left by Task 1) to:

```toml
depends = ["fmt", "lint", "test-full", "docs", "audit", "package-check", "release-smoke", "claims-audit", "deny", "secrets"]
```

- [ ] **Step 5: Give CI full history to scan**

In `.github/workflows/ci.yml`, change the checkout step of the `test` job from:

```yaml
      - name: Checkout repository
        uses: actions/checkout@v4
```

to:

```yaml
      - name: Checkout repository
        uses: actions/checkout@v4
        with:
          fetch-depth: 0 # gitleaks (mise run secrets) scans full history
```

- [ ] **Step 6: Create the pre-commit hook**

Create `.githooks/pre-commit`:

```bash
#!/usr/bin/env bash
# Blocks commits containing secrets. Activate once per clone with:
#   git config core.hooksPath .githooks
set -euo pipefail
exec mise exec -- gitleaks git --pre-commit --staged .
```

Run: `chmod +x .githooks/pre-commit`

- [ ] **Step 7: Activate and verify the hook blocks a planted secret**

Run:

```bash
git config core.hooksPath .githooks
echo 'aws_access_key_id = "AKIAQ2K9KJ3M5X7ZP0AB"' > hook-selftest.txt
git add hook-selftest.txt
git commit -m "hook selftest (must be blocked)"
```

Expected: the commit is BLOCKED — gitleaks exits 1 reporting `aws-access-key-id`, and no commit is created (`git log -1` still shows the previous commit).

Then clean up the plant:

```bash
git restore --staged hook-selftest.txt && rm hook-selftest.txt
```

- [ ] **Step 8: Verify the hook passes a clean commit, committing this task**

```bash
git add mise.toml .githooks/pre-commit .github/workflows/ci.yml
git commit -m "chore(security): add gitleaks secret scanning (task, CI, pre-commit hook)"
```

Expected: the hook runs, finds no leaks, and the commit succeeds — this doubles as the hook's clean-path test.

---

### Task 3: Renovate configuration

**Files:**
- Create: `renovate.json`

**Interfaces:**
- Consumes: nothing from earlier tasks (CI gating of Renovate PRs comes from the existing on-push ci.yml).
- Produces: Renovate onboarding config. Requires the **manual step** (below) before any PR appears.

- [ ] **Step 1: Write renovate.json**

Create `renovate.json` at the repo root:

```json
{
  "$schema": "https://docs.renovatebot.com/renovate-schema.json",
  "extends": ["config:recommended"],
  "packageRules": [
    {
      "description": "Group all non-major updates into one PR; majors stay separate",
      "matchUpdateTypes": ["minor", "patch"],
      "groupName": "non-major dependencies"
    },
    {
      "description": "release-plz owns intra-workspace crate versions",
      "matchManagers": ["cargo"],
      "matchPackageNames": ["/^pleiades-/"],
      "enabled": false
    }
  ]
}
```

No manager config is needed: Renovate auto-detects `Cargo.toml`/`Cargo.lock` (cargo), `.github/workflows/*.yml` (github-actions), and `mise.toml` (mise) — the last is why Renovate was chosen over Dependabot.

- [ ] **Step 2: Validate the config**

Run: `mise x node@22 -- npx --yes --package renovate -- renovate-config-validator renovate.json`
Expected: `Config validated successfully` (exit 0). This uses an ephemeral node — it is NOT added to `mise.toml`. If the environment has no network for npx, skip this step and note that Renovate's onboarding run reports config errors in its dashboard issue.

- [ ] **Step 3: Commit**

```bash
git add renovate.json
git commit -m "chore(deps): add Renovate config for cadence-based updates"
```

- [ ] **Step 4: Record the manual step for the user**

Report in the task summary: a maintainer must install the Renovate GitHub App on this repository (https://github.com/apps/renovate) — until then the config is inert. Do not attempt this yourself.

---

### Task 4: Threat model + AGENTS.md pointer

**Files:**
- Create: `docs/threat-model.md`
- Modify: `AGENTS.md` (the "Security and reliability" subsection)

**Interfaces:**
- Consumes: task names `deny` / `secrets` (Tasks 1–2) — referenced by name in the doc.
- Produces: `docs/threat-model.md`, referenced from AGENTS.md; Phase 4 will also link it from the README.

- [ ] **Step 1: Write docs/threat-model.md**

Create `docs/threat-model.md`:

```markdown
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
| 1 | Data ingestion (`pleiades-jpl::ingest`, kernel/corpus loading, `pleiades-compression` decode) | Untrusted bytes | Parsers return structured errors and must never panic (AGENTS.md rule); corpora are checksum-pinned with fail-closed gates; fuzzing planned (design Phase 3) |
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
```

- [ ] **Step 2: Point AGENTS.md at it**

In `AGENTS.md`, in the "Security and reliability" subsection, change:

```markdown
- Treat parsing, serialization, and external data ingestion as untrusted input boundaries.
```

to:

```markdown
- Treat parsing, serialization, and external data ingestion as untrusted input boundaries.
- Read `docs/threat-model.md` before touching security-sensitive code (ingestion, `horizons-fetch`, the release pipeline), and update it when a trust boundary changes.
```

- [ ] **Step 3: Verify referenced paths exist**

Run: `ls docs/threat-model.md docs/release-reproducibility.md && mise tasks | grep -E '^(deny|secrets)'`
Expected: both files listed; both task names present.

- [ ] **Step 4: Commit**

```bash
git add docs/threat-model.md AGENTS.md
git commit -m "docs(security): commit lightweight threat model and point AGENTS.md at it"
```

---

### Task 5: Phase acceptance check

**Files:**
- None created or modified — verification only.

**Interfaces:**
- Consumes: everything above.
- Produces: evidence that Phase 1's spec acceptance criteria hold.

- [ ] **Step 1: Run the full blocking gate**

Run: `mise run ci`
Expected: PASS, with `deny` and `secrets` visibly executed in the task list. (This is the current, pre-tiering `ci` — it is slow; that is Phase 2's problem, not this phase's.)

- [ ] **Step 2: Confirm nothing release-facing changed**

Run: `git diff 57e422cb..HEAD -- release-plz.toml release.toml cliff.toml crates/`
Expected: empty output — Phase 1 touched no crate code and no release configuration.
(If commits have landed on `main` since this plan was written, substitute the commit this phase started from.)
