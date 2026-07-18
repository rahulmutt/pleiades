#!/usr/bin/env bash
set -euo pipefail

# Maps a cargo-mutants exit code to a pass/fail verdict for this report-only
# tier. Exit 2 means "surviving mutants found", which is the EXPECTED steady
# state and must not fail the job. Exit 1 (usage error), 3 (tests timed out)
# and 4 (baseline already failing) mean the run could not measure anything and
# must fail loud — a misconfigured run and a clean run must not look alike.
#
# Usage: mutants-exit.sh <cargo-mutants-exit-code>
# Exits 0 on pass, 1 on fail (including a missing or non-numeric argument).

code="${1:-}"

if [[ -z "$code" || ! "$code" =~ ^[0-9]+$ ]]; then
  echo "::error::mutants-exit.sh requires a numeric cargo-mutants exit code, got: '${1:-<missing>}'"
  exit 1
fi

case "$code" in
  0)
    echo "All viable mutants caught."
    exit 0
    ;;
  2)
    echo "Surviving mutants found — expected for a report-only tier; see the uploaded report."
    exit 0
    ;;
  1)
    echo "::error::cargo-mutants usage error (exit 1) — the invocation is misconfigured."
    exit 1
    ;;
  3)
    echo "::error::cargo-mutants tests timed out (exit 3) — a mutant may have caused an infinite loop, or the timeout is too low."
    exit 1
    ;;
  4)
    echo "::error::cargo-mutants baseline tests are already failing (exit 4) — no mutants were tested."
    exit 1
    ;;
  *)
    echo "::error::cargo-mutants returned unexpected exit code $code."
    exit 1
    ;;
esac
