use crate::*;

#[test]
fn pluto_fallback_summary_tracks_the_current_posture() {
    let summary = pluto_fallback_summary_for_report();

    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        summary.summary_line(),
        current_pluto_fallback_summary().summary_line()
    );
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
        validated_pluto_fallback_summary_line_for_report(),
        Ok(summary.summary_line())
    );
    assert!(summary.summary_line().contains("Pluto"));
}

#[test]
fn release_body_claims_summary_tracks_the_current_posture() {
    let summary = release_body_claims_summary_for_report();

    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        summary.summary_line(),
        current_release_body_claims_summary().summary_line()
    );
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
        validated_release_body_claims_summary_line_for_report(),
        Ok(summary.summary_line())
    );
    assert!(summary.summary_line().contains("Sun through Neptune"));
}

#[test]
fn pluto_fallback_summary_rejects_policy_drift() {
    let summary =
        PlutoFallbackSummary::new("Pluto is documented elsewhere as a release-grade major body");

    assert_eq!(
        summary.validate(),
        Err(PlutoFallbackSummaryValidationError::CurrentPolicyOutOfSync)
    );
}

#[test]
fn release_body_claims_summary_rejects_policy_drift() {
    let summary = ReleaseBodyClaimsSummary::new(
        "Sun through Neptune are documented elsewhere as release-grade major bodies",
    );

    assert_eq!(
        summary.validate(),
        Err(ReleaseBodyClaimsSummaryValidationError::CurrentPolicyOutOfSync)
    );
}

#[test]
fn release_body_claims_posture_validation_tracks_the_current_boundary() {
    assert_eq!(
        validate_release_body_claims_posture(
            CURRENT_RELEASE_BODY_CLAIMS_SUMMARY_TEXT,
            CURRENT_PLUTO_FALLBACK_POLICY_SUMMARY_TEXT,
        ),
        Ok(())
    );

    let release_body_claims_summary =
            "Moon and supported lunar points (Mean Node, True Node, Mean Apogee, Mean Perigee) remain source-backed validation bodies; True Apogee and True Perigee remain unsupported; Sun through Neptune are release-grade major-body claims; Pluto remains an explicitly approximate fallback; selected asteroids (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis) remain source-backed validation bodies";
    let pluto_fallback_summary =
            "Pluto remains an explicitly approximate fallback; release-grade major-body claims include Pluto";
    assert_eq!(
        validate_release_body_claims_posture(release_body_claims_summary, pluto_fallback_summary),
        Err(ReleaseBodyClaimsPostureValidationError::MissingPlutoExclusionPhrase)
    );

    let missing_lunar_summary =
            "Sun through Neptune are release-grade major-body claims; Pluto remains an explicitly approximate fallback; selected asteroids (Ceres, Pallas, Juno, Vesta, asteroid:433-Eros, asteroid:99942-Apophis) remain source-backed validation bodies";
    assert_eq!(
        validate_release_body_claims_posture(
            missing_lunar_summary,
            CURRENT_PLUTO_FALLBACK_POLICY_SUMMARY_TEXT,
        ),
        Err(ReleaseBodyClaimsPostureValidationError::MissingLunarValidationPhrase)
    );
}
