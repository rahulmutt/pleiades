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
}

impl fmt::Display for ClaimDriftError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SurfaceDisagreesWithPosture { surface } => {
                write!(f, "surface {surface} disagrees with derived posture")
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
                surface: format!("release-body-claims-summary (missing token `{token}`)"),
            });
        }
    }
    Ok(())
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
    if let Err(ClaimDriftError::SurfaceDisagreesWithPosture { surface }) =
        summary_matches_posture(&release_summary, &posture)
    {
        errors.push(ClaimDriftError::SurfaceDisagreesWithPosture {
            surface: format!("release-body-claims-summary: {surface}"),
        });
    }

    // Surface 2: backend-matrix-summary (includes "Release-grade body claims:" line
    // which also renders posture.summary_line())
    let backend_matrix = render_backend_matrix_summary_text();
    if let Err(ClaimDriftError::SurfaceDisagreesWithPosture { surface }) =
        summary_matches_posture(&backend_matrix, &posture)
    {
        errors.push(ClaimDriftError::SurfaceDisagreesWithPosture {
            surface: format!("backend-matrix-summary: {surface}"),
        });
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
}
