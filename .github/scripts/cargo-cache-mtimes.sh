#!/usr/bin/env bash
set -euo pipefail

# Makes a restored `target` cache usable for the workspace crates.
#
# Cargo decides whether a path crate is fresh by comparing the mtime of each
# source file against the mtime of the unit's dep-info file in `target`.
# Registry crates are fingerprinted by checksum, so the cache serves them, but
# a fresh CI checkout gives every tracked file a brand-new mtime and cargo
# rebuilds every first-party crate on every run (FU-23 (b) in
# docs/follow-ups.md: about 200 s of the blocking job on a 4-core runner).
#
# Restoring each file's last-commit time (git-restore-mtime) is the usual fix,
# but it is unsafe with a cache that may come from another branch: a file whose
# last commit predates that cache's build would be judged fresh while its
# content differs, and cargo would link stale objects. This script restores
# nothing from history. It leaves a file old ONLY when its content is identical
# to what the cached build compiled:
#
#   1. read the commit the cached `target` was built from (a marker file that
#      this script writes at the end of every run, so it travels with the
#      cache);
#   2. floor the mtime of every tracked file to a fixed instant older than any
#      cached dep-info file;
#   3. touch back to "now" every file that differs from that commit: changed
#      between the two commits, or modified in the working tree.
#
# A deleted file needs no touch (cargo treats a missing source as dirty), and
# an added file is simply touched. When there is no marker, or its commit is
# not in this clone (a shallow checkout, a force-pushed branch), the script
# changes no mtime at all, which is exactly the behaviour without it.
#
# The marker says "everything in this `target` was built from <commit>", but a
# job writes it whether or not it rebuilt every artifact the cache carries. It
# is therefore only true for the artifacts the saving job builds. A job must
# not trust the marker of a cache saved by a job that builds less than it
# does: the blocking job never builds pleiades-validate's test binaries, so the
# nightly, which does, keeps its own cache lineage and drops the marker of any
# other entry (.github/workflows/nightly.yml, FU-23 (w)). Jobs that build a
# subset of what the saver built (blocking, the gate jobs) may restore either.
#
# Usage: cargo-cache-mtimes.sh [target-dir]
# Run from the repository root, after the cache restore and before any cargo
# invocation. Exits non-zero only on a usage or git error.

target="${1:-target}"
marker="$target/ci-built-from-commit"
# Older than any dep-info file the cache can hold; the exact instant is
# irrelevant as long as it predates every cached build.
floor='2000-01-01T00:00:00Z'

if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  echo "::error::cargo-cache-mtimes.sh must run inside the git checkout"
  exit 1
fi
head="$(git rev-parse HEAD)"

floor_tracked_files() {
  git ls-files -z | xargs -0 --no-run-if-empty touch --no-create --no-dereference --date="$floor"
}

# Files whose content differs from the cached build's tree, NUL-separated.
# Rename detection is off so a rename appears as one delete (filtered out)
# and one add (touched). `git diff HEAD` adds working-tree modifications,
# which never occur on CI but keep the script honest when run by hand.
changed_files() {
  local cached="$1"
  {
    git diff --name-only -z --no-renames --diff-filter=d "$cached" "$head"
    git diff --name-only -z --no-renames --diff-filter=d HEAD
  } | sort -zu
}

touch_changed_files() {
  changed_files "$1" | xargs -0 --no-run-if-empty touch --no-create --no-dereference
}

if [[ ! -f "$marker" ]]; then
  echo "cargo-cache-mtimes: no marker at $marker; mtimes left as checked out"
else
  cached="$(tr -d '[:space:]' <"$marker")"
  if ! git cat-file -e "${cached}^{commit}" 2>/dev/null; then
    echo "cargo-cache-mtimes: cached build commit ${cached} is not in this clone; mtimes left as checked out"
  else
    floor_tracked_files
    touch_changed_files "$cached"
    changed="$(changed_files "$cached" | tr -dc '\0' | wc -c)"
    echo "cargo-cache-mtimes: cached build is ${cached}; ${changed} file(s) differ and will rebuild"
  fi
fi

mkdir -p "$target"
printf '%s\n' "$head" >"$marker"
