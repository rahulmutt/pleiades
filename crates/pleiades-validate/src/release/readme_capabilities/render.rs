//! Rendering of one backend's capability block from its metadata.

use pleiades_backend::{BackendMetadata, BodyClaim, BodyClaimTier};

/// Where a block's body lines come from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BodySource {
    /// The backend's `body_claims`, grouped by tier.
    Metadata,
    /// The bodies are fixed at runtime (for example by loaded kernels); the
    /// block says so in this sentence instead.
    Runtime(&'static str),
}

const TIERS: [(BodyClaimTier, &str); 4] = [
    (BodyClaimTier::ReleaseGrade, "Release-grade"),
    (BodyClaimTier::Constrained, "Constrained"),
    (BodyClaimTier::Approximate, "Approximate"),
    (BodyClaimTier::Unsupported, "Unsupported"),
];

/// One line per capability, in the spec's order.
pub(crate) fn render_capability_lines(
    metadata: &BackendMetadata,
    bodies: &BodySource,
) -> Vec<String> {
    let caps = &metadata.capabilities;
    let mut lines = vec![
        modes_line(
            "Coordinates",
            ("mean geometric", caps.mean),
            ("apparent", caps.apparent),
            "mean",
            "apparent",
        ),
        modes_line(
            "Observer",
            ("geocentric", caps.geocentric),
            ("topocentric", caps.topocentric),
            "geocentric",
            "topocentric",
        ),
        format!(
            "Frames: {} · Time scales: {}",
            join(
                metadata
                    .supported_frames
                    .iter()
                    .map(|frame| frame.to_string().to_lowercase())
            ),
            join(
                metadata
                    .supported_time_scales
                    .iter()
                    .map(ToString::to_string)
            ),
        ),
        format!("Window: {}", window(metadata)),
    ];
    match bodies {
        BodySource::Metadata => lines.extend(body_lines(&metadata.body_claims)),
        BodySource::Runtime(sentence) => lines.push(format!("Bodies: {sentence}")),
    }
    if caps.native_sidereal {
        lines.push("Sidereal: native".to_string());
    }
    let limits: Vec<&str> = [
        (!caps.batch, "not batch-capable"),
        (!metadata.deterministic, "not deterministic"),
        (!metadata.offline, "needs network access"),
    ]
    .into_iter()
    .filter_map(|(applies, text)| applies.then_some(text))
    .collect();
    if !limits.is_empty() {
        lines.push(format!("Limits: {}", limits.join("; ")));
    }
    lines
}

/// `"<label>: first and second"`, or `"<label>: first; <rejected> requests
/// are rejected"` when only one mode is supported.
fn modes_line(
    label: &str,
    first: (&str, bool),
    second: (&str, bool),
    first_request: &str,
    second_request: &str,
) -> String {
    match (first.1, second.1) {
        (true, true) => format!("{label}: {} and {}", first.0, second.0),
        (true, false) => format!(
            "{label}: {}; {second_request} requests are rejected",
            first.0
        ),
        (false, true) => format!(
            "{label}: {}; {first_request} requests are rejected",
            second.0
        ),
        (false, false) => format!("{label}: none"),
    }
}

fn window(metadata: &BackendMetadata) -> String {
    let range = &metadata.nominal_range;
    if range.start.is_none() && range.end.is_none() {
        "no nominal limit".to_string()
    } else {
        range.summary_line()
    }
}

fn body_lines(claims: &[BodyClaim]) -> Vec<String> {
    let mut lines = Vec::new();
    for (tier, label) in TIERS {
        let in_tier: Vec<&BodyClaim> = claims.iter().filter(|claim| claim.tier == tier).collect();
        let Some(first) = in_tier.first() else {
            if tier == BodyClaimTier::ReleaseGrade {
                lines.push("Release-grade bodies: none".to_string());
            }
            continue;
        };
        let names = join(in_tier.iter().map(|claim| claim.body.to_string()));
        if in_tier.iter().all(|claim| claim.accuracy == first.accuracy) {
            lines.push(format!(
                "{label} bodies ({}): {names}",
                first.accuracy.label()
            ));
        } else {
            lines.push(format!("{label} bodies: {names}"));
        }
    }
    lines
}

fn join(items: impl Iterator<Item = String>) -> String {
    items.collect::<Vec<_>>().join(", ")
}

/// The text between a block's markers: a newline, then one bullet per line.
pub(crate) fn render_block_body(lines: &[String]) -> String {
    let mut body = String::from("\n");
    for line in lines {
        body.push_str("- ");
        body.push_str(line);
        body.push('\n');
    }
    body
}
