//! Claim drift gate: proves every rendered release surface agrees with the
//! derived [`ReleasePosture`].
//!
//! ## Gated surfaces
//!
//! | Surface | Why gated |
//! |---------|-----------|
//! | `release-body-claims-summary` | Primary posture surface; directly renders `posture.summary_line()`, which is the canonical source of `{body}@{id}` tokens. |
//! | `backend-matrix-summary` | Secondary surface; includes a "Release-grade body claims:" line that also renders `posture.summary_line()` via `format_release_body_claims_summary_for_report()`. |
//!
//! ## Why other surfaces are excluded
//!
//! - **Pluto-fallback summary** (`render_pluto_fallback_summary_text`) also embeds
//!   `format_release_body_claims_summary_for_report()`, but it additionally requires
//!   a live comparison report (potentially slow). The two surfaces above already
//!   cover the canonical token path; the fallback surface is redundant here and
//!   adds latency without additional coverage.
//! - **Compatibility profile** and other renderers do not emit `{body}@{id}` tokens
//!   at all — adding a raw token check to them would produce an always-failing gate.

use std::fmt;

use pleiades_backend::ReleasePosture;

use crate::claims::derived_release_posture;
use crate::render::summary::{
    render_backend_matrix_summary_text, render_release_body_claims_summary_text,
};

/// A surface whose rendered claims disagree with the derived posture.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ClaimDriftError {
    /// The named surface does not contain the expected posture token.
    SurfaceDisagreesWithPosture { surface: String },
    /// The named surface failed to render and returned an unavailability sentinel
    /// rather than a real value.  This is a render/validation failure, not drift.
    ///
    /// NOTE: detection couples to the renderers' error-sentinel convention: both
    /// `format_release_body_claims_summary_for_report()` (release path) and
    /// `render_backend_matrix_summary_text()` (backend path) return strings
    /// containing `" unavailable ("` when internal validation fails.  If that
    /// convention changes, the detection substring below must change too.
    SurfaceRenderFailed { surface: String, detail: String },
}

impl fmt::Display for ClaimDriftError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SurfaceDisagreesWithPosture { surface } => {
                write!(f, "surface {surface} disagrees with derived posture")
            }
            Self::SurfaceRenderFailed { surface, detail } => {
                write!(
                    f,
                    "surface {surface} failed to render (render/validation error, not drift): {detail}"
                )
            }
        }
    }
}

impl std::error::Error for ClaimDriftError {}

/// Returns `Ok` if `rendered` reflects every release-grade entry in `posture`.
///
/// For each `(backend_id, body)` pair in `posture.release_grade()` the comparator
/// expects the token `{body}@{backend_id}` (e.g. `Pluto@pleiades-data`) to appear
/// somewhere in `rendered`. If any token is absent, the function returns `Err`
/// naming the missing token.
pub(crate) fn summary_matches_posture(
    rendered: &str,
    posture: &ReleasePosture,
) -> Result<(), ClaimDriftError> {
    for (id, body) in posture.release_grade() {
        let token = format!("{body}@{id}");
        if !rendered.contains(&token) {
            return Err(ClaimDriftError::SurfaceDisagreesWithPosture {
                surface: format!("missing token `{token}`"),
            });
        }
    }
    Ok(())
}

/// Sentinel substring shared by both gated renderers' error paths.
///
/// NOTE: this couples to the renderers' error-sentinel convention.
/// `format_release_body_claims_summary_for_report()` returns
/// `"release-grade body claims unavailable (...)"` and
/// `render_backend_matrix_summary_text()` returns
/// `"Backend matrix summary unavailable (...)"` (plus other
/// `"... unavailable (...)"` early returns).  If that convention changes,
/// this substring and `ClaimDriftError::SurfaceRenderFailed`'s doc must
/// change too.
const RENDER_FAILED_SENTINEL: &str = " unavailable (";

/// Classifies a single rendered surface against the derived posture.
///
/// Returns `None` if the surface is healthy, or `Some(ClaimDriftError)`
/// describing the specific failure:
/// - `SurfaceRenderFailed` when the rendered text contains an unavailability
///   sentinel (the renderer itself failed, not a drift problem).
/// - `SurfaceDisagreesWithPosture` when the render succeeded but a required
///   posture token is absent.
///
/// The `name` parameter becomes the `surface` field of the returned error.
pub(crate) fn classify_surface(
    name: &str,
    rendered: &str,
    posture: &ReleasePosture,
) -> Option<ClaimDriftError> {
    if let Some(line) = rendered
        .lines()
        .find(|l| l.contains(RENDER_FAILED_SENTINEL))
    {
        // Capture only a short excerpt so the detail stays readable.
        let detail: String = line.chars().take(120).collect();
        return Some(ClaimDriftError::SurfaceRenderFailed {
            surface: name.to_string(),
            detail,
        });
    }
    match summary_matches_posture(rendered, posture) {
        Ok(()) => None,
        Err(ClaimDriftError::SurfaceDisagreesWithPosture { surface }) => {
            Some(ClaimDriftError::SurfaceDisagreesWithPosture {
                surface: format!("{name}: {surface}"),
            })
        }
        Err(other) => Some(other),
    }
}

/// Checks every gated release-facing surface against the derived posture.
///
/// Returns `Ok(())` when all surfaces carry the expected `{body}@{id}` tokens.
/// Returns `Err(Vec<ClaimDriftError>)` listing all surfaces that disagree.
pub(crate) fn check_claim_drift() -> Result<(), Vec<ClaimDriftError>> {
    let posture = derived_release_posture();
    let mut errors: Vec<ClaimDriftError> = Vec::new();

    // Surface 1: release-body-claims-summary (primary — directly renders posture.summary_line())
    let release_summary = render_release_body_claims_summary_text();
    if let Some(err) = classify_surface("release-body-claims-summary", &release_summary, &posture) {
        errors.push(err);
    }

    // Surface 2: backend-matrix-summary (includes "Release-grade body claims:" line
    // which also renders posture.summary_line())
    let backend_matrix = render_backend_matrix_summary_text();
    if let Some(err) = classify_surface("backend-matrix-summary", &backend_matrix, &posture) {
        errors.push(err);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Positive test: the freshly-rendered surfaces agree with the derived posture.
    #[test]
    fn drift_passes_for_freshly_rendered_surfaces() {
        assert!(check_claim_drift().is_ok());
    }

    /// Tamper test: proves the comparator has teeth.
    ///
    /// We use the REAL derived posture (which has ≥1 release-grade entry, e.g.
    /// `Pluto@pleiades-data`) and pass an empty string that is guaranteed to be
    /// missing all `{body}@{id}` tokens. The comparator must return `Err`.
    ///
    /// We also verify a second scenario: take the real rendered summary, strip one
    /// token from it, and assert that the comparator flags the tampered string.
    #[test]
    fn drift_detects_tampered_posture() {
        let posture = derived_release_posture();

        // Precondition: the real posture has at least one release-grade entry.
        let rg = posture.release_grade();
        assert!(
            !rg.is_empty(),
            "derived_release_posture() must have ≥1 release-grade entry for this test to be meaningful"
        );

        // Scenario 1: empty string is missing every token → must fail.
        assert!(
            summary_matches_posture("", &posture).is_err(),
            "summary_matches_posture should Err for empty rendered string"
        );

        // Scenario 2: take the real rendered summary, delete one {body}@{id} token
        // from it, and assert the comparator flags the tampered string.
        let real_rendered = render_release_body_claims_summary_text();
        let (first_id, first_body) = &rg[0];
        let token_to_remove = format!("{first_body}@{first_id}");
        let tampered = real_rendered.replace(&token_to_remove, "REMOVED");
        assert!(
            summary_matches_posture(&tampered, &posture).is_err(),
            "summary_matches_posture should Err when a release-grade token is removed"
        );

        // Scenario 3: the real rendered summary must pass.
        assert!(
            summary_matches_posture(&real_rendered, &posture).is_ok(),
            "summary_matches_posture should Ok for the real rendered summary"
        );
    }

    /// Fix-2 test: a render-failure sentinel is reported as `SurfaceRenderFailed`,
    /// not as `SurfaceDisagreesWithPosture`.
    ///
    /// We feed a synthetic "unavailable" string (matching the sentinel substring
    /// `" unavailable ("`) into `classify_surface` and assert it returns
    /// `SurfaceRenderFailed`.  This proves the gate is HONEST: a render/validation
    /// failure is never misreported as drift.
    #[test]
    fn render_failure_sentinel_reported_as_render_failed_not_drift() {
        let posture = derived_release_posture();

        // Synthetic sentinel strings that both renderers may produce on failure.
        let release_sentinel =
            "Release-grade body claims summary\nRelease-grade body claims: release-grade body claims unavailable (boom)\n";
        let backend_sentinel = "Backend matrix summary unavailable (boom)";

        for (name, sentinel) in [
            ("release-body-claims-summary", release_sentinel),
            ("backend-matrix-summary", backend_sentinel),
        ] {
            let result = classify_surface(name, sentinel, &posture);
            match result {
                Some(ClaimDriftError::SurfaceRenderFailed { surface, .. }) => {
                    assert_eq!(surface, name, "surface name should match");
                }
                Some(ClaimDriftError::SurfaceDisagreesWithPosture { surface }) => {
                    panic!(
                        "Expected SurfaceRenderFailed for sentinel input on '{name}', got SurfaceDisagreesWithPosture {{ surface: {surface:?} }}"
                    );
                }
                None => {
                    panic!("Expected Some(SurfaceRenderFailed) for sentinel input on '{name}', got None");
                }
            }
        }
    }
}
