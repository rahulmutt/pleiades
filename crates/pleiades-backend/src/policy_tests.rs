use crate::*;

#[test]
fn time_scale_policy_summary_has_a_compact_display() {
    let summary = TimeScalePolicySummary::current();

    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        summary.summary_line(),
        CURRENT_TIME_SCALE_POLICY_SUMMARY_TEXT
    );
    assert!(summary.summary_line().contains("TT/TDB"));
    assert!(summary.validate().is_ok());
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
        time_scale_policy_summary_for_report().summary_line(),
        summary.summary_line()
    );
}

#[test]
fn time_scale_policy_summary_validate_rejects_blank_fields() {
    let summary = TimeScalePolicySummary::new(" ");

    let error = summary
        .validate()
        .expect_err("blank policy prose should fail validation");
    assert_eq!(error.to_string(), "time-scale policy summary is blank");
    assert!(summary.validated_summary_line().is_err());
}

#[test]
fn time_scale_policy_summary_validate_rejects_policy_drift() {
    let summary = TimeScalePolicySummary::new(
            "direct backend requests accept TT/TDB; UTC/UT1 inputs require caller-supplied conversion helpers; built-in Delta T model",
        );

    let error = summary
        .validate()
        .expect_err("drifted policy prose should fail validation");
    assert_eq!(
        error.to_string(),
        "time-scale policy summary is out of sync with the current posture"
    );
    assert!(summary.validated_summary_line().is_err());
}

#[test]
fn delta_t_policy_summary_has_a_compact_display() {
    let summary = DeltaTPolicySummary::current();

    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(summary.summary_line(), CURRENT_DELTA_T_POLICY_SUMMARY_TEXT);
    assert!(summary.summary_line().contains("Delta T"));
    assert!(summary.validate().is_ok());
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
        delta_t_policy_summary_for_report().summary_line(),
        summary.summary_line()
    );
}

#[test]
fn delta_t_policy_summary_validate_rejects_blank_fields() {
    let summary = DeltaTPolicySummary::new(" ");

    let error = summary
        .validate()
        .expect_err("blank Delta T policy prose should fail validation");
    assert_eq!(error.to_string(), "Delta T policy summary is blank");
    assert!(summary.validated_summary_line().is_err());
}

#[test]
fn delta_t_policy_summary_validate_rejects_policy_drift() {
    let summary = DeltaTPolicySummary::new("built-in Delta T modeling is documented elsewhere");

    let error = summary
        .validate()
        .expect_err("drifted Delta T policy prose should fail validation");
    assert_eq!(
        error.to_string(),
        "Delta T policy summary is out of sync with the current posture"
    );
    assert!(summary.validated_summary_line().is_err());
}

#[test]
fn utc_convenience_policy_summary_has_a_compact_display() {
    let summary = UtcConveniencePolicySummary::current();

    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        summary.summary_line(),
        CURRENT_UTC_CONVENIENCE_POLICY_SUMMARY_TEXT
    );
    assert!(summary.summary_line().contains("UTC convenience"));
    assert!(summary.validate().is_ok());
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
        utc_convenience_policy_summary_for_report().summary_line(),
        summary.summary_line()
    );
}

#[test]
fn utc_convenience_policy_summary_validate_rejects_blank_fields() {
    let summary = UtcConveniencePolicySummary::new(" ");

    let error = summary
        .validate()
        .expect_err("blank UTC convenience prose should fail validation");
    assert_eq!(error.to_string(), "UTC convenience policy summary is blank");
    assert!(summary.validated_summary_line().is_err());
}

#[test]
fn utc_convenience_policy_summary_validate_rejects_policy_drift() {
    let summary = UtcConveniencePolicySummary::new(
        "built-in UTC convenience conversion is documented elsewhere",
    );

    let error = summary
        .validate()
        .expect_err("drifted UTC convenience policy prose should fail validation");
    assert_eq!(
        error.to_string(),
        "UTC convenience policy summary is out of sync with the current posture"
    );
    assert!(summary.validated_summary_line().is_err());
}

#[test]
fn validated_utc_convenience_policy_summary_for_report_tracks_the_current_posture() {
    assert_eq!(
        validated_utc_convenience_policy_summary_for_report(),
        CURRENT_UTC_CONVENIENCE_POLICY_SUMMARY_TEXT
    );
}

#[test]
fn validated_request_policy_component_summaries_track_the_current_posture() {
    assert_eq!(
        validated_time_scale_policy_summary_for_report(),
        CURRENT_TIME_SCALE_POLICY_SUMMARY_TEXT
    );
    assert_eq!(
        validated_delta_t_policy_summary_for_report(),
        CURRENT_DELTA_T_POLICY_SUMMARY_TEXT
    );
    assert_eq!(
        validated_request_policy_summary_for_report(),
        current_request_policy_summary()
            .validated_summary_line()
            .unwrap()
    );
    assert_eq!(
        validated_observer_policy_summary_for_report(),
        CURRENT_OBSERVER_POLICY_SUMMARY_TEXT
    );
    assert_eq!(
        validated_apparentness_policy_summary_for_report(),
        CURRENT_APPARENTNESS_POLICY_SUMMARY_TEXT
    );
}

#[test]
fn request_policy_summary_has_a_compact_display() {
    let summary = RequestPolicySummary::current();

    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        summary.summary_line(),
        RequestPolicySummary {
            time_scale: CURRENT_TIME_SCALE_POLICY_SUMMARY_TEXT,
            observer: CURRENT_OBSERVER_POLICY_SUMMARY_TEXT,
            apparentness: CURRENT_APPARENTNESS_POLICY_SUMMARY_TEXT,
            frame: CURRENT_FRAME_POLICY_SUMMARY_TEXT,
        }
        .summary_line()
    );
    assert!(summary.summary_line().contains("time-scale="));
    assert!(summary.summary_line().contains("observer="));
    assert!(summary.summary_line().contains("apparentness="));
    assert!(summary.summary_line().contains("frame="));
    assert!(summary.validate().is_ok());
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
        validated_request_semantics_summary_for_report(),
        validated_request_policy_summary_for_report()
    );
    assert_eq!(
        request_semantics_summary_for_report(),
        request_policy_summary_for_report()
    );
    assert_eq!(
        request_semantics_summary_for_report().summary_line(),
        request_policy_summary_for_report().summary_line()
    );
    assert_eq!(
        request_semantics_summary_for_report().validated_summary_line(),
        request_policy_summary_for_report().validated_summary_line()
    );
}

#[test]
fn request_policy_summary_validate_rejects_blank_fields() {
    let mut summary = RequestPolicySummary::current();
    summary.frame = " ";

    let error = summary
        .validate()
        .expect_err("blank policy prose should fail validation");
    assert_eq!(
        error,
        RequestPolicySummaryValidationError::BlankField { field: "frame" }
    );
    assert_eq!(
        error.to_string(),
        "the request-policy summary field `frame` is blank"
    );
    assert!(summary.validated_summary_line().is_err());
}

#[test]
fn request_policy_summary_validate_rejects_whitespace_padded_fields() {
    let mut summary = RequestPolicySummary::current();
    summary.observer = " chart houses use observer locations; chart body observers stay separate; body requests stay geocentric; geocentric-only backends reject observer-bearing requests with UnsupportedObserver; malformed observer coordinates remain InvalidObserver; topocentric body positions remain unsupported ";

    let error = summary
        .validate()
        .expect_err("whitespace-padded policy prose should fail validation");
    assert_eq!(
        error,
        RequestPolicySummaryValidationError::WhitespacePaddedField { field: "observer" }
    );
    assert_eq!(
        error.to_string(),
        "the request-policy summary field `observer` has surrounding whitespace"
    );
    assert!(summary.validated_summary_line().is_err());
}

#[test]
fn request_policy_summary_validate_rejects_line_breaks() {
    let mut summary = RequestPolicySummary::current();
    summary.observer = "chart houses use observer locations\nbody requests stay geocentric";

    let error = summary
        .validate()
        .expect_err("multi-line policy prose should fail validation");
    assert_eq!(
        error,
        RequestPolicySummaryValidationError::EmbeddedLineBreak { field: "observer" }
    );
    assert_eq!(
        error.to_string(),
        "the request-policy summary field `observer` contains a line break"
    );
    assert!(summary.validated_summary_line().is_err());
}

#[test]
fn frame_treatment_summary_has_a_compact_display() {
    let summary = FrameTreatmentSummary::new(
            "geocentric ecliptic inputs; equatorial coordinates are derived with a mean-obliquity transform",
        );

    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(summary.summary_line(), "geocentric ecliptic inputs; equatorial coordinates are derived with a mean-obliquity transform");
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(
        validated_frame_treatment_summary_for_report(),
        current_request_policy_summary().frame
    );
    assert!(summary.summary_line().contains("mean-obliquity"));
}

#[test]
fn frame_treatment_summary_rejects_blank_summary_text() {
    let summary = FrameTreatmentSummary::new("   ");

    assert_eq!(
        summary.validate(),
        Err(FrameTreatmentSummaryValidationError::BlankSummary)
    );
}

#[test]
fn frame_treatment_summary_rejects_whitespace_padded_summary_text() {
    let summary = FrameTreatmentSummary::new(
            " geocentric ecliptic inputs; equatorial coordinates are derived with a mean-obliquity transform ",
        );

    assert_eq!(
        summary.validate(),
        Err(FrameTreatmentSummaryValidationError::WhitespacePaddedSummary)
    );
}

#[test]
fn frame_treatment_summary_rejects_embedded_line_breaks() {
    let summary = FrameTreatmentSummary::new(
            "geocentric ecliptic inputs;\nequatorial coordinates are derived with a mean-obliquity transform",
        );

    assert_eq!(
        summary.validate(),
        Err(FrameTreatmentSummaryValidationError::EmbeddedLineBreak)
    );
}

#[test]
fn frame_policy_summary_tracks_the_current_posture() {
    let summary = FramePolicySummary::new(current_request_policy_summary().frame);

    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        summary.summary_line(),
        current_request_policy_summary().frame
    );
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert!(summary.summary_line().contains("mean-obliquity"));
}

#[test]
fn frame_policy_summary_rejects_policy_drift() {
    let summary = FramePolicySummary::new(
            "geocentric ecliptic inputs; equatorial coordinates are derived with a mean-obliquity transform",
        );

    assert_eq!(
        summary.validate(),
        Err(FramePolicySummaryValidationError::CurrentPolicyOutOfSync)
    );
}

#[test]
fn frame_policy_summary_details_reuse_the_current_posture() {
    let summary = frame_policy_summary_details();

    assert_eq!(
        summary.summary_line(),
        current_request_policy_summary().frame
    );
    assert_eq!(
        frame_policy_summary_for_report(),
        current_request_policy_summary().frame
    );
    assert_eq!(
        validated_frame_policy_summary_for_report(),
        current_request_policy_summary().frame
    );
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
}

#[test]
fn native_sidereal_policy_summary_tracks_the_current_posture() {
    let summary = native_sidereal_policy_summary_for_report();

    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        summary.summary_line(),
        current_native_sidereal_policy_summary().summary_line()
    );
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert!(summary
        .summary_line()
        .contains("native sidereal backend output"));
}

#[test]
fn native_sidereal_policy_summary_rejects_policy_drift() {
    let summary =
        NativeSiderealPolicySummary::new("native sidereal backend output is documented elsewhere");

    assert_eq!(
        summary.validate(),
        Err(NativeSiderealPolicySummaryValidationError::CurrentPolicyOutOfSync)
    );
}

#[test]
fn validated_native_sidereal_policy_summary_for_report_tracks_the_current_posture() {
    assert_eq!(
        validated_native_sidereal_policy_summary_for_report(),
        CURRENT_NATIVE_SIDEREAL_POLICY_SUMMARY_TEXT
    );
}

#[test]
fn zodiac_policy_summary_tracks_the_current_posture() {
    let summary = current_zodiac_policy_summary();

    assert_eq!(summary.to_string(), summary.summary_line());
    assert_eq!(
        summary.summary_line(),
        current_zodiac_policy_summary().summary_line()
    );
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
    assert_eq!(summary.summary_line(), CURRENT_ZODIAC_POLICY_SUMMARY_TEXT);
}

#[test]
fn zodiac_policy_summary_rejects_invalid_cached_prose() {
    assert_eq!(
        ZodiacPolicySummary::new("   ").validate(),
        Err(ZodiacPolicySummaryValidationError::BlankSummary)
    );
    assert_eq!(
        ZodiacPolicySummary::new(" tropical only ").validate(),
        Err(ZodiacPolicySummaryValidationError::WhitespacePaddedSummary)
    );
    assert_eq!(
        ZodiacPolicySummary::new("tropical\nonly").validate(),
        Err(ZodiacPolicySummaryValidationError::EmbeddedLineBreak)
    );
}

#[test]
fn zodiac_policy_summary_rejects_policy_drift() {
    let summary = ZodiacPolicySummary::new("sidereal zodiac output is documented elsewhere");

    assert_eq!(
        summary.validate(),
        Err(ZodiacPolicySummaryValidationError::CurrentPolicyOutOfSync)
    );
}

#[test]
fn validated_zodiac_policy_summary_for_report_tracks_the_current_posture() {
    assert_eq!(
        validated_zodiac_policy_summary_for_report(),
        CURRENT_ZODIAC_POLICY_SUMMARY_TEXT
    );
}

#[test]
fn observer_policy_summary_validates_the_current_report_prose() {
    let summary = observer_policy_summary_for_report();
    assert_eq!(summary.summary_line(), summary.to_string());
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
}

#[test]
fn apparentness_policy_summary_validates_the_current_report_prose() {
    let summary = apparentness_policy_summary_for_report();
    assert_eq!(summary.summary_line(), summary.to_string());
    assert_eq!(summary.validated_summary_line(), Ok(summary.summary_line()));
}

#[test]
fn observer_policy_summary_rejects_invalid_cached_prose() {
    assert!(matches!(
        ObserverPolicySummary::new("").validate(),
        Err(ObserverPolicySummaryValidationError::BlankSummary)
    ));
    assert!(matches!(
        ObserverPolicySummary::new(" observer ").validate(),
        Err(ObserverPolicySummaryValidationError::WhitespacePaddedSummary)
    ));
    assert!(matches!(
        ObserverPolicySummary::new("observer\npolicy").validate(),
        Err(ObserverPolicySummaryValidationError::EmbeddedLineBreak)
    ));
    assert!(matches!(
        ObserverPolicySummary::new("observer policy drift").validate(),
        Err(ObserverPolicySummaryValidationError::CurrentPolicyOutOfSync)
    ));
}

#[test]
fn apparentness_policy_summary_rejects_invalid_cached_prose() {
    assert!(matches!(
        ApparentnessPolicySummary::new("").validate(),
        Err(ApparentnessPolicySummaryValidationError::BlankSummary)
    ));
    assert!(matches!(
        ApparentnessPolicySummary::new(" apparent ").validate(),
        Err(ApparentnessPolicySummaryValidationError::WhitespacePaddedSummary)
    ));
    assert!(matches!(
        ApparentnessPolicySummary::new("apparent\npolicy").validate(),
        Err(ApparentnessPolicySummaryValidationError::EmbeddedLineBreak)
    ));
    assert!(matches!(
        ApparentnessPolicySummary::new("apparentness policy drift").validate(),
        Err(ApparentnessPolicySummaryValidationError::CurrentPolicyOutOfSync)
    ));
}

#[test]
fn validate_observer_policy_rejects_invalid_observer_locations_even_when_supported() {
    let mut observer_request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Tt,
        ),
    );
    observer_request.observer = Some(ObserverLocation::new(
        Latitude::from_degrees(95.0),
        Longitude::from_degrees(-0.1),
        Some(45.0),
    ));
    let error = validate_observer_policy(&observer_request, "toy backend", true).expect_err(
        "invalid observer locations should fail even when topocentric support is available",
    );
    assert_eq!(error.kind, EphemerisErrorKind::InvalidObserver);
    assert!(error
        .message
        .contains("observer latitude must stay within [-90, 90]"));
    assert!(error.message.contains("received invalid observer location"));
}

#[test]
fn request_policy_summary_validation_rejects_stale_field_text() {
    fn assert_field_out_of_sync(
        mut summary: RequestPolicySummary,
        field: &'static str,
        mutate: impl FnOnce(&mut RequestPolicySummary),
    ) {
        mutate(&mut summary);

        let error = summary
            .validate()
            .expect_err("stale request-policy wording should fail validation");

        assert_eq!(
            error,
            RequestPolicySummaryValidationError::FieldOutOfSync { field }
        );
        assert_eq!(
                error.to_string(),
                format!(
                    "the request-policy summary field `{field}` is out of sync with the current posture"
                )
            );
    }

    let current = current_request_policy_summary();

    assert_field_out_of_sync(current, "time_scale", |summary| {
        summary.time_scale =
                "direct backend requests accept TT/TDB; UTC/UT1 inputs require caller-supplied conversion helpers";
    });
    assert_field_out_of_sync(current, "observer", |summary| {
        summary.observer = "chart houses use observer locations; chart body observers stay separate; body requests stay geocentric";
    });
    assert_field_out_of_sync(current, "apparentness", |summary| {
        summary.apparentness = "current first-party backends accept mean geometric output only";
    });
    assert_field_out_of_sync(current, "frame", |summary| {
        summary.frame = "ecliptic body positions are the default request shape";
    });
}
