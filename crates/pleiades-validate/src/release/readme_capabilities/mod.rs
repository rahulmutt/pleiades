//! README capability blocks (issue #166): each release backend's README
//! carries a block, between `<!-- audit:capabilities <backend-id> -->` and
//! `<!-- /audit:capabilities -->`, rendered from that backend's
//! `BackendMetadata`. `workspace-audit` checks the blocks, and
//! `pleiades-validate readme-capabilities` regenerates them.

mod registry;
mod render;

pub(crate) use registry::{rendered_blocks, RenderedBlock};

use std::ops::Range;
use std::path::Path;

use super::workspace_audit::WorkspaceAuditViolation;

pub(crate) const OPEN_PREFIX: &str = "<!-- audit:capabilities ";
const OPEN_END: &str = " -->";
pub(crate) const CLOSE: &str = "<!-- /audit:capabilities -->";
const REGENERATE: &str = "run `mise run readme-capabilities` to regenerate it";

/// One block found in a README: its backend id and the byte range between
/// its markers.
pub(crate) struct ParsedBlock<'a> {
    pub id: &'a str,
    pub body: Range<usize>,
}

/// Every block in `text`, in order, or the first marker defect: an opening
/// marker without ` -->` or without a closing marker, a block inside a
/// block, a closing marker with no opening one, or one backend twice.
pub(crate) fn parse_blocks(text: &str) -> Result<Vec<ParsedBlock<'_>>, String> {
    let mut blocks: Vec<ParsedBlock<'_>> = Vec::new();
    let mut position = 0;
    loop {
        let next_open = text[position..].find(OPEN_PREFIX).map(|at| position + at);
        let gap_end = next_open.unwrap_or(text.len());
        if text[position..gap_end].contains(CLOSE) {
            return Err(format!("a `{CLOSE}` marker has no opening marker"));
        }
        let Some(open) = next_open else {
            return Ok(blocks);
        };
        let id_start = open + OPEN_PREFIX.len();
        let id_end = text[id_start..]
            .find(OPEN_END)
            .map(|at| id_start + at)
            .filter(|&end| !text[id_start..end].contains('\n'))
            .ok_or_else(|| {
                format!("an opening `{OPEN_PREFIX}` marker is not closed by `-->` on its line")
            })?;
        let id = text[id_start..id_end].trim();
        let body_start = id_end + OPEN_END.len();
        let close = text[body_start..]
            .find(CLOSE)
            .map(|at| body_start + at)
            .ok_or_else(|| format!("the `{id}` block has no `{CLOSE}` marker"))?;
        if text[body_start..close].contains(OPEN_PREFIX) {
            return Err(format!("the `{id}` block contains another opening marker"));
        }
        if blocks.iter().any(|block| block.id == id) {
            return Err(format!("the `{id}` block appears twice"));
        }
        blocks.push(ParsedBlock {
            id,
            body: body_start..close,
        });
        position = close + CLOSE.len();
    }
}

/// The capability-block violations of one README. `readme` is its
/// workspace-relative path with `/` separators; `expected` is every
/// registered block, rendered.
pub(crate) fn audit_capability_blocks(
    path: &Path,
    readme: &str,
    text: &str,
    expected: &[RenderedBlock],
) -> Vec<WorkspaceAuditViolation> {
    let violation = |rule: &'static str, detail: String| WorkspaceAuditViolation {
        path: path.to_path_buf(),
        rule,
        detail,
    };
    let parsed = match parse_blocks(text) {
        Ok(parsed) => parsed,
        Err(detail) => return vec![violation("readme.capabilities-block-malformed", detail)],
    };
    let mut violations = Vec::new();
    for block in &parsed {
        match expected
            .iter()
            .find(|candidate| candidate.backend_id == block.id)
        {
            None => violations.push(violation(
                "readme.capabilities-block-unknown",
                format!(
                    "the `{}` capability block names no registered backend",
                    block.id
                ),
            )),
            Some(candidate) if candidate.readme != readme => violations.push(violation(
                "readme.capabilities-block-unknown",
                format!(
                    "the `{}` capability block belongs in {}, not here",
                    block.id, candidate.readme
                ),
            )),
            Some(candidate) => {
                if let Some(detail) = first_difference(&text[block.body.clone()], &candidate.body) {
                    violations.push(violation(
                        "readme.capabilities-block-stale",
                        format!(
                            "the `{}` capability block differs from its metadata: {detail}; {REGENERATE}",
                            block.id
                        ),
                    ));
                }
            }
        }
    }
    for candidate in expected
        .iter()
        .filter(|candidate| candidate.readme == readme)
    {
        if !parsed.iter().any(|block| block.id == candidate.backend_id) {
            violations.push(violation(
                "readme.capabilities-block-missing",
                format!(
                    "the README has no `{}` capability block; add `{OPEN_PREFIX}{} -->` and `{CLOSE}` where it belongs, then {REGENERATE}",
                    candidate.backend_id, candidate.backend_id
                ),
            ));
        }
    }
    violations
}

/// `None` when `found == expected`; otherwise where they first differ, by
/// line, as "line N: found `..`, expected `..`".
fn first_difference(found: &str, expected: &str) -> Option<String> {
    if found == expected {
        return None;
    }
    // `split('\n')` keeps a stray `\r`, so a CRLF block differs visibly.
    // A final newline ends the last line rather than starting an empty one.
    let mut found_lines = found.strip_suffix('\n').unwrap_or(found).split('\n');
    let mut expected_lines = expected.strip_suffix('\n').unwrap_or(expected).split('\n');
    let show = |side: Option<&str>| {
        side.map_or("(end of block)".to_string(), |text| {
            text.escape_debug().to_string()
        })
    };
    let mut line = 1;
    loop {
        match (found_lines.next(), expected_lines.next()) {
            (Some(f), Some(e)) if f == e => line += 1,
            (None, None) => return Some("the final newline differs".to_string()),
            (f, e) => {
                return Some(format!(
                    "line {line}: found `{}`, expected `{}`",
                    show(f),
                    show(e)
                ));
            }
        }
    }
}

/// `text` with each block registered for `readme` replaced by its rendering.
/// Blocks of other ids are left for the audit to report. Malformed markers
/// are an error: nothing is rewritten.
pub(crate) fn rewrite_capability_blocks(
    readme: &str,
    text: &str,
    expected: &[RenderedBlock],
) -> Result<String, String> {
    let parsed = parse_blocks(text)?;
    let mut out = String::with_capacity(text.len());
    let mut copied = 0;
    for block in &parsed {
        let Some(candidate) = expected
            .iter()
            .find(|candidate| candidate.backend_id == block.id && candidate.readme == readme)
        else {
            continue;
        };
        out.push_str(&text[copied..block.body.start]);
        out.push_str(&candidate.body);
        copied = block.body.end;
    }
    out.push_str(&text[copied..]);
    Ok(out)
}

/// The `readme-capabilities` command over the workspace at `root`.
/// Rewrites every registered README's blocks, or with `check` writes nothing
/// and fails on any violation. Either way a missing block fails.
pub(crate) fn readme_capabilities_at(
    root: &Path,
    expected: &[RenderedBlock],
    check: bool,
) -> Result<String, String> {
    // Sorted so that `dedup` collapses a README whose blocks are not adjacent.
    let mut readmes: Vec<&str> = expected.iter().map(|block| block.readme).collect();
    readmes.sort_unstable();
    readmes.dedup();
    let mut report = Vec::new();
    let mut violations = Vec::new();
    for readme in readmes {
        let path = root.join(readme);
        let text = std::fs::read_to_string(&path).map_err(|error| format!("{readme}: {error}"))?;
        if check {
            violations.extend(audit_capability_blocks(&path, readme, &text, expected));
            continue;
        }
        let rewritten = rewrite_capability_blocks(readme, &text, expected)
            .map_err(|detail| format!("{readme}: {detail}"))?;
        if rewritten == text {
            report.push(format!("unchanged {readme}"));
        } else {
            std::fs::write(&path, &rewritten).map_err(|error| format!("{readme}: {error}"))?;
            report.push(format!("updated {readme}"));
        }
        violations.extend(audit_capability_blocks(&path, readme, &rewritten, expected));
    }
    if violations.is_empty() {
        report.push(format!(
            "README capability blocks: OK ({} blocks)",
            expected.len()
        ));
        Ok(report.join("\n"))
    } else {
        let lines: Vec<String> = violations
            .iter()
            .map(|violation| {
                format!(
                    "{} [{}]: {}",
                    violation.path.display(),
                    violation.rule,
                    violation.detail
                )
            })
            .collect();
        Err(format!(
            "README capability blocks failed:\n{}",
            lines.join("\n")
        ))
    }
}

#[cfg(test)]
mod tests;
