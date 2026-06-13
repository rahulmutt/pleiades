use super::*;
use core::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use pleiades_types::CoordinateValidationError;

#[test]
fn family_and_accuracy_labels_are_stable() {
    assert_eq!(BackendFamily::Algorithmic.to_string(), "Algorithmic");
    assert_eq!(BackendFamily::ReferenceData.to_string(), "ReferenceData");
    assert_eq!(BackendFamily::CompressedData.to_string(), "CompressedData");
    assert_eq!(BackendFamily::Composite.to_string(), "Composite");
    assert_eq!(
        BackendFamily::Other("custom".to_string()).to_string(),
        "Other(custom)"
    );

    assert!(BackendFamily::ReferenceData.is_data_backed());
    assert!(BackendFamily::CompressedData.is_data_backed());
    assert!(!BackendFamily::Algorithmic.is_data_backed());
    assert!(BackendFamily::Algorithmic.is_algorithmic());
    assert!(BackendFamily::Composite.is_routing());
    assert_eq!(
        BackendFamily::Algorithmic.posture().to_string(),
        "algorithmic"
    );
    assert_eq!(
        BackendFamily::ReferenceData.posture().to_string(),
        "data-backed"
    );
    assert_eq!(
        BackendFamily::CompressedData.posture().to_string(),
        "data-backed"
    );
    assert_eq!(BackendFamily::Composite.posture().to_string(), "routing");
    assert_eq!(
        BackendFamily::Other("custom".to_string())
            .posture()
            .to_string(),
        "other"
    );
    assert_eq!(BackendFamily::Algorithmic.posture_label(), "algorithmic");
    assert_eq!(BackendFamily::ReferenceData.posture_label(), "data-backed");
    assert_eq!(BackendFamily::CompressedData.posture_label(), "data-backed");
    assert_eq!(BackendFamily::Composite.posture_label(), "routing");
    assert_eq!(
        BackendFamily::Other("custom".to_string()).posture_label(),
        "other"
    );

    assert_eq!(AccuracyClass::Exact.to_string(), "Exact");
    assert_eq!(AccuracyClass::High.to_string(), "High");
    assert_eq!(AccuracyClass::Moderate.to_string(), "Moderate");
    assert_eq!(AccuracyClass::Approximate.to_string(), "Approximate");
    assert_eq!(AccuracyClass::Unknown.to_string(), "Unknown");

    assert_eq!(QualityAnnotation::Exact.to_string(), "Exact");
    assert_eq!(QualityAnnotation::Interpolated.to_string(), "Interpolated");
    assert_eq!(QualityAnnotation::Approximate.to_string(), "Approximate");
    assert_eq!(QualityAnnotation::Unknown.to_string(), "Unknown");

    assert_eq!(
        EphemerisErrorKind::UnsupportedBody.to_string(),
        "UnsupportedBody"
    );
    assert_eq!(
        EphemerisErrorKind::UnsupportedCoordinateFrame.to_string(),
        "UnsupportedCoordinateFrame"
    );
    assert_eq!(
        EphemerisErrorKind::UnsupportedTimeScale.to_string(),
        "UnsupportedTimeScale"
    );
    assert_eq!(
        EphemerisErrorKind::InvalidObserver.to_string(),
        "InvalidObserver"
    );
    assert_eq!(
        EphemerisErrorKind::OutOfRangeInstant.to_string(),
        "OutOfRangeInstant"
    );
    assert_eq!(
        EphemerisErrorKind::MissingDataset.to_string(),
        "MissingDataset"
    );
    assert_eq!(
        EphemerisErrorKind::NumericalFailure.to_string(),
        "NumericalFailure"
    );
    assert_eq!(
        EphemerisErrorKind::InvalidRequest.to_string(),
        "InvalidRequest"
    );

    let error = EphemerisError::new(EphemerisErrorKind::InvalidRequest, "example failure");
    assert_eq!(error.summary_line(), "InvalidRequest: example failure");
    assert_eq!(error.to_string(), error.summary_line());
}

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
fn backend_metadata_has_a_compact_display() {
    let metadata = BackendMetadata {
        id: BackendId::new("toy"),
        version: "0.1.0".to_string(),
        family: BackendFamily::Algorithmic,
        provenance: BackendProvenance::new("example backend"),
        nominal_range: TimeRange::new(None, None),
        supported_time_scales: vec![TimeScale::Tt, TimeScale::Tdb],
        body_coverage: vec![CelestialBody::Sun, CelestialBody::Moon],
        supported_frames: vec![CoordinateFrame::Ecliptic, CoordinateFrame::Equatorial],
        capabilities: BackendCapabilities::default(),
        accuracy: AccuracyClass::Approximate,
        deterministic: true,
        offline: true,
    };

    assert_eq!(metadata.to_string(), metadata.summary_line());
    assert_eq!(
        metadata.validated_summary_line(),
        Ok(metadata.summary_line())
    );
    assert!(metadata.summary_line().contains("id=toy"));
    assert!(metadata.summary_line().contains("version=0.1.0"));
    assert!(metadata.summary_line().contains("family=Algorithmic"));
    assert!(metadata
        .summary_line()
        .contains("family posture=algorithmic"));
    assert!(metadata.summary_line().contains("accuracy=Approximate"));
    assert!(metadata.summary_line().contains("deterministic=true"));
    assert!(metadata.summary_line().contains("offline=true"));
    assert!(metadata.summary_line().contains("time scales=[TT, TDB]"));
    assert!(metadata.summary_line().contains("bodies=[Sun, Moon]"));
    assert!(metadata
        .summary_line()
        .contains("frames=[Ecliptic, Equatorial]"));
    assert!(metadata.summary_line().contains("capabilities=["));
    assert!(metadata
        .summary_line()
        .contains("provenance=example backend"));
    assert!(metadata.validate().is_ok());
}

#[test]
fn backend_metadata_validation_rejects_blank_and_duplicate_fields() {
    let mut metadata = BackendMetadata {
        id: BackendId::new(" "),
        version: "0.1.0".to_string(),
        family: BackendFamily::Algorithmic,
        provenance: BackendProvenance::new("example backend"),
        nominal_range: TimeRange::new(None, None),
        supported_time_scales: vec![TimeScale::Tt, TimeScale::Tt],
        body_coverage: vec![CelestialBody::Sun, CelestialBody::Sun],
        supported_frames: vec![CoordinateFrame::Ecliptic],
        capabilities: BackendCapabilities::default(),
        accuracy: AccuracyClass::Approximate,
        deterministic: true,
        offline: true,
    };

    let error = metadata
        .validate()
        .expect_err("blank backend ids should fail validation");
    assert_eq!(
        error.summary_line(),
        "backend metadata field `id` is blank or whitespace-padded"
    );
    assert_eq!(error.to_string(), error.summary_line());
    assert!(metadata.validated_summary_line().is_err());

    metadata.id = BackendId::new("toy");
    metadata.provenance.summary = " ".to_string();

    let error = metadata
        .validate()
        .expect_err("blank provenance summaries should fail validation");
    assert_eq!(
        error.summary_line(),
        "backend metadata field `provenance summary` is blank or whitespace-padded"
    );
    assert_eq!(error.to_string(), error.summary_line());

    metadata.provenance.summary = "example backend".to_string();
    metadata.provenance.data_sources = vec![" source A".to_string()];

    let error = metadata
        .validate()
        .expect_err("whitespace-padded provenance sources should fail validation");
    assert_eq!(
        error.summary_line(),
        "backend metadata field `provenance data sources` is blank or whitespace-padded"
    );
    assert_eq!(error.to_string(), error.summary_line());

    metadata.provenance.data_sources = vec!["source A".to_string()];
    metadata.supported_time_scales = vec![TimeScale::Tt];
    metadata.body_coverage = vec![CelestialBody::Sun];
    metadata.supported_frames = vec![CoordinateFrame::Ecliptic, CoordinateFrame::Ecliptic];

    let error = metadata
        .validate()
        .expect_err("duplicate supported frames should fail validation");
    assert_eq!(
        error.summary_line(),
        "backend metadata field `supported frames` contains duplicate entry `Ecliptic`"
    );
    assert_eq!(error.to_string(), error.summary_line());

    metadata.supported_frames = vec![CoordinateFrame::Ecliptic];
    metadata.provenance.data_sources = vec!["source A".to_string(), "source A".to_string()];

    let error = metadata
        .validate()
        .expect_err("duplicate provenance sources should fail validation");
    assert_eq!(
        error.summary_line(),
        "backend metadata field `provenance data sources` contains duplicate entry `source A`"
    );
    assert_eq!(error.to_string(), error.summary_line());

    metadata.provenance.data_sources = vec!["source A".to_string()];
    metadata.nominal_range = TimeRange::new(
        Some(Instant::new(
            JulianDay::from_days(2_451_546.0),
            TimeScale::Tt,
        )),
        Some(Instant::new(
            JulianDay::from_days(2_451_545.0),
            TimeScale::Tt,
        )),
    );

    let error = metadata
        .validate()
        .expect_err("out-of-order nominal ranges should fail validation");
    assert_eq!(
        error.summary_line(),
        "backend metadata nominal range end must not precede the start"
    );
    assert_eq!(error.to_string(), error.summary_line());

    metadata.nominal_range = TimeRange::new(
        Some(Instant::new(
            JulianDay::from_days(2_451_545.0),
            TimeScale::Tt,
        )),
        Some(Instant::new(
            JulianDay::from_days(2_451_546.0),
            TimeScale::Tdb,
        )),
    );

    let error = metadata
        .validate()
        .expect_err("mixed nominal-range scales should fail validation");
    assert_eq!(
        error.summary_line(),
        "backend metadata nominal range bounds must use the same time scale"
    );
    assert_eq!(error.to_string(), error.summary_line());

    metadata.nominal_range = TimeRange::new(
        Some(Instant::new(
            JulianDay::from_days(2_451_545.0),
            TimeScale::Tt,
        )),
        Some(Instant::new(
            JulianDay::from_days(2_451_546.0),
            TimeScale::Tt,
        )),
    );
    metadata.capabilities = BackendCapabilities {
        geocentric: false,
        topocentric: false,
        apparent: false,
        mean: false,
        batch: true,
        native_sidereal: false,
    };

    let error = metadata
        .validate()
        .expect_err("capability flags without a position or value mode should fail validation");
    assert_eq!(
            error.summary_line(),
            "backend metadata field `capabilities` is invalid: backend capabilities must support geocentric or topocentric positions"
        );
    assert_eq!(error.to_string(), error.summary_line());

    metadata.capabilities = BackendCapabilities::default();
    metadata.nominal_range = TimeRange::new(
        Some(Instant::new(
            JulianDay::from_days(f64::INFINITY),
            TimeScale::Tt,
        )),
        Some(Instant::new(
            JulianDay::from_days(2_451_546.0),
            TimeScale::Tt,
        )),
    );

    let error = metadata
        .validate()
        .expect_err("non-finite nominal-range bounds should fail validation");
    assert_eq!(
        error.summary_line(),
        "backend metadata nominal range must use finite Julian-day bounds"
    );
    assert_eq!(error.to_string(), error.summary_line());
}

#[test]
fn backend_capabilities_validation_rejects_missing_position_or_value_modes() {
    let mut capabilities = BackendCapabilities::default();
    assert!(capabilities.validate().is_ok());

    capabilities.geocentric = false;
    capabilities.topocentric = false;
    let error = capabilities
        .validate()
        .expect_err("capabilities without a position mode should fail validation");
    assert_eq!(
        error.summary_line(),
        "backend capabilities must support geocentric or topocentric positions"
    );
    assert_eq!(error.to_string(), error.summary_line());

    capabilities.geocentric = true;
    capabilities.topocentric = false;
    capabilities.apparent = false;
    capabilities.mean = false;
    let error = capabilities
        .validate()
        .expect_err("capabilities without a value mode should fail validation");
    assert_eq!(
        error.summary_line(),
        "backend capabilities must support mean or apparent output"
    );
    assert_eq!(error.to_string(), error.summary_line());
}

#[test]
fn ephemeris_request_has_a_compact_display() {
    let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let request = EphemerisRequest::new(CelestialBody::Mars, instant);
    let request = EphemerisRequest {
        observer: Some(ObserverLocation::new(
            Latitude::from_degrees(51.5),
            Longitude::from_degrees(-0.1),
            None,
        )),
        ..request
    };

    assert_eq!(request.to_string(), request.summary_line());
    assert_eq!(
            request.summary_line(),
            "body=Mars; instant=JD 2451545 TT; frame=Ecliptic; zodiac=Tropical; apparent=Mean; observer=latitude=51.5°, longitude=359.9°, elevation=n/a"
        );
    assert!(request.summary_line().contains("body=Mars"));
    assert!(request.summary_line().contains("observer="));
}

#[test]
fn ephemeris_result_has_a_compact_display() {
    let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let mut result = EphemerisResult::new(
        BackendId::new("toy"),
        CelestialBody::Sun,
        instant,
        CoordinateFrame::Ecliptic,
        ZodiacMode::Tropical,
        Apparentness::Mean,
    );
    result.ecliptic = Some(EclipticCoordinates::new(
        Longitude::from_degrees(12.5),
        Latitude::from_degrees(-3.25),
        Some(1.234),
    ));
    result.equatorial = Some(EquatorialCoordinates::new(
        Angle::from_degrees(98.0),
        Latitude::from_degrees(0.5),
        None,
    ));
    result.motion = Some(Motion::new(Some(0.1), Some(-0.2), Some(0.003)));
    result.quality = QualityAnnotation::Exact;

    assert_eq!(result.to_string(), result.summary_line());
    assert_eq!(
            result.summary_line(),
            "backend=toy; body=Sun; instant=JD 2451545 TT; frame=Ecliptic; zodiac=Tropical; apparent=Mean; quality=Exact; ecliptic=longitude=12.5°, latitude=-3.25°, distance=1.234 AU; equatorial=right_ascension=98°, declination=0.5°, distance=n/a; motion=longitude_speed=0.1 deg/day, latitude_speed=-0.2 deg/day, distance_speed=0.003 AU/day"
        );
    assert!(result.summary_line().contains("backend=toy"));
    assert!(result.summary_line().contains("quality=Exact"));
    assert!(result.summary_line().contains("ecliptic=longitude=12.5°"));
}

#[test]
fn backend_provenance_summary_has_a_compact_display() {
    let provenance = BackendProvenance {
        summary: "toy backend for tests".to_string(),
        data_sources: vec!["source A".to_string(), "source B".to_string()],
    };

    assert_eq!(provenance.to_string(), provenance.summary_line());
    assert_eq!(provenance.summary_line(), "toy backend for tests");
    assert!(provenance.summary_line().contains("toy backend for tests"));
    assert_eq!(
        provenance.validated_summary_line(),
        Ok(provenance.summary_line())
    );
    assert!(provenance.validate().is_ok());
}

#[test]
fn backend_provenance_validation_rejects_blank_summary_and_duplicate_sources() {
    let mut provenance = BackendProvenance {
        summary: " ".to_string(),
        data_sources: vec!["source A".to_string(), "source A".to_string()],
    };

    let error = provenance
        .validate()
        .expect_err("blank provenance summaries should fail validation");
    assert_eq!(
        error.summary_line(),
        "backend provenance summary must not be blank or whitespace-padded"
    );
    assert_eq!(error.to_string(), error.summary_line());

    provenance.summary = "toy backend".to_string();
    provenance.data_sources = vec![" source A".to_string()];

    let error = provenance
        .validate()
        .expect_err("whitespace-padded provenance sources should fail validation");
    assert_eq!(
        error.summary_line(),
        "backend provenance data source at index 0 must not be blank or whitespace-padded"
    );
    assert_eq!(error.to_string(), error.summary_line());

    provenance.data_sources = vec!["source A".to_string(), "source A".to_string()];

    let error = provenance
        .validate()
        .expect_err("duplicate provenance sources should fail validation");
    assert_eq!(
        error.summary_line(),
        "backend provenance data sources contain duplicate entry `source A`"
    );
    assert_eq!(error.to_string(), error.summary_line());
    assert!(provenance.validated_summary_line().is_err());
}

#[test]
fn backend_capabilities_summary_has_a_compact_display() {
    let capabilities = BackendCapabilities::default();

    assert_eq!(capabilities.to_string(), capabilities.summary_line());
    assert_eq!(
        capabilities.validated_summary_line(),
        Ok(capabilities.summary_line())
    );
    assert_eq!(
            capabilities.summary_line(),
            "geocentric=true; topocentric=false; apparent=true; mean=true; batch=true; native_sidereal=false"
        );
    assert!(capabilities.summary_line().contains("geocentric="));
    assert!(capabilities.summary_line().contains("topocentric="));
    assert!(capabilities.summary_line().contains("apparent="));
    assert!(capabilities.summary_line().contains("native_sidereal="));
}

#[test]
fn backend_capabilities_validated_summary_line_rejects_missing_modes() {
    let capabilities = BackendCapabilities {
        geocentric: false,
        topocentric: false,
        apparent: false,
        mean: false,
        ..BackendCapabilities::default()
    };

    assert_eq!(
        capabilities.validated_summary_line(),
        Err(BackendCapabilitiesValidationError::MissingPositionMode)
    );
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

struct ToyBackend;

impl EphemerisBackend for ToyBackend {
    fn metadata(&self) -> BackendMetadata {
        BackendMetadata {
            id: BackendId::new("toy"),
            version: "0.1.0".to_string(),
            family: BackendFamily::Algorithmic,
            provenance: BackendProvenance::new("toy backend for tests"),
            nominal_range: TimeRange::new(None, None),
            supported_time_scales: vec![TimeScale::Tt],
            body_coverage: vec![CelestialBody::Sun],
            supported_frames: vec![CoordinateFrame::Ecliptic],
            capabilities: BackendCapabilities::default(),
            accuracy: AccuracyClass::Approximate,
            deterministic: true,
            offline: true,
        }
    }

    fn supports_body(&self, body: CelestialBody) -> bool {
        body == CelestialBody::Sun
    }

    fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
        if req.body != CelestialBody::Sun {
            return Err(EphemerisError::new(
                EphemerisErrorKind::UnsupportedBody,
                "only the Sun is supported",
            ));
        }

        let mut result = EphemerisResult::new(
            BackendId::new("toy"),
            req.body.clone(),
            req.instant,
            req.frame,
            req.zodiac_mode.clone(),
            req.apparent,
        );
        result.quality = QualityAnnotation::Exact;
        result.ecliptic = Some(EclipticCoordinates::new(
            Longitude::from_degrees(120.0),
            Latitude::from_degrees(0.0),
            Some(1.0),
        ));
        Ok(result)
    }
}

#[test]
fn request_policy_helpers_reject_unsupported_shapes() {
    let time_scale_request = EphemerisRequest {
        body: CelestialBody::Sun,
        instant: Instant::new(JulianDay::from_days(2451545.0), TimeScale::Utc),
        observer: Some(ObserverLocation::new(
            Latitude::from_degrees(51.5),
            Longitude::from_degrees(-0.1),
            Some(45.0),
        )),
        frame: CoordinateFrame::Equatorial,
        zodiac_mode: ZodiacMode::Tropical,
        apparent: Apparentness::Apparent,
    };
    let conversion = TimeScaleConversion::new(TimeScale::Utc, TimeScale::Tt, 64.184);
    assert!(time_scale_request
        .validate_time_scale_conversion(conversion)
        .is_ok());
    let converted = time_scale_request
        .clone()
        .with_time_scale_conversion(conversion)
        .expect("UTC request should convert with the caller-supplied policy");
    assert_eq!(converted.instant.scale, TimeScale::Tt);
    assert_eq!(converted.body, time_scale_request.body);
    assert_eq!(converted.observer, time_scale_request.observer);
    assert_eq!(converted.frame, time_scale_request.frame);
    assert_eq!(converted.zodiac_mode, time_scale_request.zodiac_mode);
    assert_eq!(converted.apparent, time_scale_request.apparent);

    let checked_offset = time_scale_request
        .clone()
        .with_instant_time_scale_offset_checked(TimeScale::Tt, 64.184)
        .expect("UTC request should accept the checked offset helper");
    assert_eq!(checked_offset.instant.scale, TimeScale::Tt);
    assert_eq!(checked_offset.body, time_scale_request.body);
    assert_eq!(checked_offset.observer, time_scale_request.observer);
    assert_eq!(checked_offset.frame, time_scale_request.frame);
    assert_eq!(checked_offset.zodiac_mode, time_scale_request.zodiac_mode);
    assert_eq!(checked_offset.apparent, time_scale_request.apparent);

    let tt_from_tdb_request = EphemerisRequest {
        instant: Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tdb),
        ..time_scale_request.clone()
    };
    let tt_from_tdb = tt_from_tdb_request
        .clone()
        .with_tt_from_tdb(-0.001_657)
        .expect("TDB request should convert back to TT with a caller-supplied offset");
    assert_eq!(tt_from_tdb.instant.scale, TimeScale::Tt);
    assert_eq!(tt_from_tdb.body, tt_from_tdb_request.body);
    assert_eq!(tt_from_tdb.observer, tt_from_tdb_request.observer);
    assert_eq!(tt_from_tdb.frame, tt_from_tdb_request.frame);
    assert_eq!(tt_from_tdb.zodiac_mode, tt_from_tdb_request.zodiac_mode);
    assert_eq!(tt_from_tdb.apparent, tt_from_tdb_request.apparent);

    let tt_from_tdb_signed = tt_from_tdb_request
        .clone()
        .with_tt_from_tdb_signed(-0.001_657)
        .expect("TDB request should convert back to TT with a signed offset");
    assert_eq!(tt_from_tdb_signed.instant.scale, TimeScale::Tt);
    assert_eq!(tt_from_tdb_signed.body, tt_from_tdb_request.body);
    assert_eq!(tt_from_tdb_signed.observer, tt_from_tdb_request.observer);
    assert_eq!(tt_from_tdb_signed.frame, tt_from_tdb_request.frame);
    assert_eq!(
        tt_from_tdb_signed.zodiac_mode,
        tt_from_tdb_request.zodiac_mode
    );
    assert_eq!(tt_from_tdb_signed.apparent, tt_from_tdb_request.apparent);

    let tt_from_tdb_unsigned = tt_from_tdb_request
        .clone()
        .with_tt_from_tdb(0.001_657)
        .expect("TDB request should convert back to TT with a duration offset");
    assert_eq!(tt_from_tdb_unsigned.instant.scale, TimeScale::Tt);
    assert_eq!(tt_from_tdb_unsigned.body, tt_from_tdb_request.body);
    assert_eq!(tt_from_tdb_unsigned.observer, tt_from_tdb_request.observer);
    assert_eq!(tt_from_tdb_unsigned.frame, tt_from_tdb_request.frame);
    assert_eq!(
        tt_from_tdb_unsigned.zodiac_mode,
        tt_from_tdb_request.zodiac_mode
    );
    assert_eq!(tt_from_tdb_unsigned.apparent, tt_from_tdb_request.apparent);

    let tt_from_ut1_request = EphemerisRequest {
        instant: Instant::new(JulianDay::from_days(2451545.0), TimeScale::Ut1),
        ..time_scale_request.clone()
    };
    let tt_from_ut1 = tt_from_ut1_request
        .clone()
        .with_tt_from_ut1_signed(64.184)
        .expect("UT1 request should convert to TT with a signed offset");
    assert_eq!(tt_from_ut1.instant.scale, TimeScale::Tt);
    assert_eq!(tt_from_ut1.body, tt_from_ut1_request.body);
    assert_eq!(tt_from_ut1.observer, tt_from_ut1_request.observer);
    assert_eq!(tt_from_ut1.frame, tt_from_ut1_request.frame);
    assert_eq!(tt_from_ut1.zodiac_mode, tt_from_ut1_request.zodiac_mode);
    assert_eq!(tt_from_ut1.apparent, tt_from_ut1_request.apparent);

    let tt_from_ut1_unsigned = tt_from_ut1_request
        .clone()
        .with_tt_from_ut1(Duration::from_secs_f64(64.184))
        .expect("UT1 request should convert to TT with a duration offset");
    assert_eq!(tt_from_ut1_unsigned.instant.scale, TimeScale::Tt);
    assert_eq!(tt_from_ut1_unsigned.body, tt_from_ut1_request.body);
    assert_eq!(tt_from_ut1_unsigned.observer, tt_from_ut1_request.observer);
    assert_eq!(tt_from_ut1_unsigned.frame, tt_from_ut1_request.frame);
    assert_eq!(
        tt_from_ut1_unsigned.zodiac_mode,
        tt_from_ut1_request.zodiac_mode
    );
    assert_eq!(tt_from_ut1_unsigned.apparent, tt_from_ut1_request.apparent);

    let tt_from_utc_request = EphemerisRequest {
        instant: Instant::new(JulianDay::from_days(2451545.0), TimeScale::Utc),
        ..time_scale_request.clone()
    };
    let tt_from_utc = tt_from_utc_request
        .clone()
        .with_tt_from_utc_signed(64.184)
        .expect("UTC request should convert to TT with a signed offset");
    assert_eq!(tt_from_utc.instant.scale, TimeScale::Tt);
    assert_eq!(tt_from_utc.body, tt_from_utc_request.body);
    assert_eq!(tt_from_utc.observer, tt_from_utc_request.observer);
    assert_eq!(tt_from_utc.frame, tt_from_utc_request.frame);
    assert_eq!(tt_from_utc.zodiac_mode, tt_from_utc_request.zodiac_mode);
    assert_eq!(tt_from_utc.apparent, tt_from_utc_request.apparent);

    let tdb_from_tt_request = EphemerisRequest {
        instant: Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt),
        ..time_scale_request.clone()
    };
    let tdb_from_tt = tdb_from_tt_request
        .clone()
        .with_tdb_from_tt_signed(-0.001_657)
        .expect("TT request should convert to TDB with a signed offset");
    assert_eq!(tdb_from_tt.instant.scale, TimeScale::Tdb);
    assert_eq!(tdb_from_tt.body, tdb_from_tt_request.body);
    assert_eq!(tdb_from_tt.observer, tdb_from_tt_request.observer);
    assert_eq!(tdb_from_tt.frame, tdb_from_tt_request.frame);
    assert_eq!(tdb_from_tt.zodiac_mode, tdb_from_tt_request.zodiac_mode);
    assert_eq!(tdb_from_tt.apparent, tdb_from_tt_request.apparent);

    let tdb_from_tt_unsigned = tdb_from_tt_request
        .clone()
        .with_tdb_from_tt(Duration::from_secs_f64(0.001_657))
        .expect("TT request should convert to TDB with a duration offset");
    assert_eq!(tdb_from_tt_unsigned.instant.scale, TimeScale::Tdb);
    assert_eq!(tdb_from_tt_unsigned.body, tdb_from_tt_request.body);
    assert_eq!(tdb_from_tt_unsigned.observer, tdb_from_tt_request.observer);
    assert_eq!(tdb_from_tt_unsigned.frame, tdb_from_tt_request.frame);
    assert_eq!(
        tdb_from_tt_unsigned.zodiac_mode,
        tdb_from_tt_request.zodiac_mode
    );
    assert_eq!(tdb_from_tt_unsigned.apparent, tdb_from_tt_request.apparent);

    let tdb_from_ut1_request = EphemerisRequest {
        instant: Instant::new(JulianDay::from_days(2451545.0), TimeScale::Ut1),
        ..time_scale_request.clone()
    };
    let tdb_from_ut1 = tdb_from_ut1_request
        .clone()
        .with_tdb_from_ut1_signed(Duration::from_secs_f64(64.184), -0.001_657)
        .expect("UT1 request should convert to TDB with caller-supplied offsets");
    assert_eq!(tdb_from_ut1.instant.scale, TimeScale::Tdb);
    assert_eq!(tdb_from_ut1.body, tdb_from_ut1_request.body);
    assert_eq!(tdb_from_ut1.observer, tdb_from_ut1_request.observer);
    assert_eq!(tdb_from_ut1.frame, tdb_from_ut1_request.frame);
    assert_eq!(tdb_from_ut1.zodiac_mode, tdb_from_ut1_request.zodiac_mode);
    assert_eq!(tdb_from_ut1.apparent, tdb_from_ut1_request.apparent);

    let tdb_from_ut1_unsigned = tdb_from_ut1_request
        .clone()
        .with_tdb_from_ut1(
            Duration::from_secs_f64(64.184),
            Duration::from_secs_f64(0.001_657),
        )
        .expect("UT1 request should convert to TDB with duration offsets");
    assert_eq!(tdb_from_ut1_unsigned.instant.scale, TimeScale::Tdb);
    assert_eq!(tdb_from_ut1_unsigned.body, tdb_from_ut1_request.body);
    assert_eq!(
        tdb_from_ut1_unsigned.observer,
        tdb_from_ut1_request.observer
    );
    assert_eq!(tdb_from_ut1_unsigned.frame, tdb_from_ut1_request.frame);
    assert_eq!(
        tdb_from_ut1_unsigned.zodiac_mode,
        tdb_from_ut1_request.zodiac_mode
    );
    assert_eq!(
        tdb_from_ut1_unsigned.apparent,
        tdb_from_ut1_request.apparent
    );

    let tdb_from_utc_request = EphemerisRequest {
        instant: Instant::new(JulianDay::from_days(2451545.0), TimeScale::Utc),
        ..time_scale_request.clone()
    };
    let tdb_from_utc = tdb_from_utc_request
        .clone()
        .with_tdb_from_utc_signed(Duration::from_secs_f64(64.184), -0.001_657)
        .expect("UTC request should convert to TDB with caller-supplied offsets");
    assert_eq!(tdb_from_utc.instant.scale, TimeScale::Tdb);
    assert_eq!(tdb_from_utc.body, tdb_from_utc_request.body);
    assert_eq!(tdb_from_utc.observer, tdb_from_utc_request.observer);
    assert_eq!(tdb_from_utc.frame, tdb_from_utc_request.frame);
    assert_eq!(tdb_from_utc.zodiac_mode, tdb_from_utc_request.zodiac_mode);
    assert_eq!(tdb_from_utc.apparent, tdb_from_utc_request.apparent);

    let tdb_from_utc_unsigned = tdb_from_utc_request
        .clone()
        .with_tdb_from_utc(
            Duration::from_secs_f64(64.184),
            Duration::from_secs_f64(0.001_657),
        )
        .expect("UTC request should convert to TDB with duration offsets");
    assert_eq!(tdb_from_utc_unsigned.instant.scale, TimeScale::Tdb);
    assert_eq!(tdb_from_utc_unsigned.body, tdb_from_utc_request.body);
    assert_eq!(
        tdb_from_utc_unsigned.observer,
        tdb_from_utc_request.observer
    );
    assert_eq!(tdb_from_utc_unsigned.frame, tdb_from_utc_request.frame);
    assert_eq!(
        tdb_from_utc_unsigned.zodiac_mode,
        tdb_from_utc_request.zodiac_mode
    );
    assert_eq!(
        tdb_from_utc_unsigned.apparent,
        tdb_from_utc_request.apparent
    );

    let error = time_scale_request
        .clone()
        .with_instant_time_scale_offset_checked(TimeScale::Tt, f64::NAN)
        .expect_err("non-finite offsets should be rejected at the request layer");
    assert_eq!(error, TimeScaleConversionError::NonFiniteOffset);

    let error = time_scale_request
        .validate_time_scale_conversion(TimeScaleConversion::new(
            TimeScale::Tt,
            TimeScale::Tt,
            64.184,
        ))
        .expect_err("mismatched source scales should fail validation before retagging");
    assert_eq!(
        error,
        TimeScaleConversionError::Expected {
            expected: TimeScale::Tt,
            actual: TimeScale::Utc
        }
    );
    let error = validate_request_policy(
        &time_scale_request,
        "toy backend",
        &[TimeScale::Tt],
        &[CoordinateFrame::Ecliptic],
        true,
        false,
    )
    .expect_err("UTC should be rejected when only TT is supported");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedTimeScale);
    assert_eq!(
        error.message,
        "toy backend expects one of [TT] for request instants"
    );

    let frame_request = EphemerisRequest {
        instant: Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt),
        observer: None,
        frame: CoordinateFrame::Equatorial,
        apparent: Apparentness::Mean,
        ..time_scale_request.clone()
    };
    let error = validate_request_policy(
        &frame_request,
        "toy backend",
        &[TimeScale::Tt],
        &[CoordinateFrame::Ecliptic],
        true,
        false,
    )
    .expect_err("equatorial frame should be rejected when only ecliptic is supported");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedCoordinateFrame);
    assert_eq!(
        error.message,
        "toy backend only returns [Ecliptic] coordinates"
    );

    let apparent_request = EphemerisRequest {
        frame: CoordinateFrame::Ecliptic,
        apparent: Apparentness::Apparent,
        observer: None,
        ..frame_request.clone()
    };
    let error = validate_request_policy(
        &apparent_request,
        "toy backend",
        &[TimeScale::Tt],
        &[CoordinateFrame::Ecliptic],
        true,
        false,
    )
    .expect_err("apparent requests should be rejected when only mean output is supported");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedApparentness);
    assert_eq!(
            error.message,
            "toy backend currently returns mean geometric coordinates only; apparent corrections are not implemented"
        );

    let invalid_observer_apparent_request = EphemerisRequest {
        observer: Some(ObserverLocation::new(
            Latitude::from_degrees(95.0),
            Longitude::from_degrees(-0.1),
            Some(45.0),
        )),
        ..apparent_request.clone()
    };
    let unsupported_apparent_metadata = BackendMetadata {
        id: BackendId::new("toy backend"),
        version: "0.1.0".to_string(),
        family: BackendFamily::Algorithmic,
        provenance: BackendProvenance::new("toy backend"),
        nominal_range: TimeRange::new(None, None),
        supported_time_scales: vec![TimeScale::Tt],
        body_coverage: vec![CelestialBody::Sun],
        supported_frames: vec![CoordinateFrame::Ecliptic],
        capabilities: BackendCapabilities {
            geocentric: true,
            topocentric: false,
            apparent: false,
            mean: true,
            batch: true,
            native_sidereal: false,
        },
        accuracy: AccuracyClass::Approximate,
        deterministic: true,
        offline: true,
    };
    let error = validate_request_against_metadata(
        &invalid_observer_apparent_request,
        &unsupported_apparent_metadata,
    )
    .expect_err("unsupported apparentness should be reported before observer validation");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedApparentness);
    assert_eq!(
            error.message,
            "toy backend currently returns mean geometric coordinates only; apparent corrections are not implemented"
        );

    let batch_seed_request = EphemerisRequest {
        apparent: Apparentness::Mean,
        ..apparent_request.clone()
    };
    let batch_error = validate_requests_against_metadata(
            &[batch_seed_request, invalid_observer_apparent_request.clone()],
            &unsupported_apparent_metadata,
        )
        .expect_err("batch validation should report unsupported apparentness before invalid observer validation");
    assert_eq!(
        batch_error.kind,
        EphemerisErrorKind::UnsupportedApparentness
    );
    assert_eq!(
            batch_error.message,
            "batch request 2: toy backend currently returns mean geometric coordinates only; apparent corrections are not implemented"
        );

    let mean_request = EphemerisRequest {
        apparent: Apparentness::Mean,
        ..apparent_request.clone()
    };
    let error = validate_request_policy(
        &mean_request,
        "toy backend",
        &[TimeScale::Tt],
        &[CoordinateFrame::Ecliptic],
        false,
        true,
    )
    .expect_err("mean requests should be rejected when only apparent output is supported");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedApparentness);
    assert_eq!(
            error.message,
            "toy backend currently returns apparent coordinates only; mean geometric coordinates are not implemented"
        );

    let metadata = BackendMetadata {
        id: BackendId::new("toy backend"),
        version: "0.1.0".to_string(),
        family: BackendFamily::Algorithmic,
        provenance: BackendProvenance::new("toy backend"),
        nominal_range: TimeRange::new(None, None),
        supported_time_scales: vec![TimeScale::Tt],
        body_coverage: vec![CelestialBody::Sun],
        supported_frames: vec![CoordinateFrame::Ecliptic],
        capabilities: BackendCapabilities {
            geocentric: true,
            topocentric: false,
            apparent: false,
            mean: true,
            batch: true,
            native_sidereal: false,
        },
        accuracy: AccuracyClass::Approximate,
        deterministic: true,
        offline: true,
    };

    let frame_error = validate_request_against_metadata(&frame_request, &metadata)
        .expect_err("equatorial requests should still be rejected through metadata preflight");
    assert_eq!(
        frame_error.kind,
        EphemerisErrorKind::UnsupportedCoordinateFrame
    );
    assert_eq!(
        frame_error.message,
        "toy backend only returns [Ecliptic] coordinates"
    );

    let metadata_frame_error = metadata
        .validate_request(&frame_request)
        .expect_err("metadata request validation should match the shared preflight");
    assert_eq!(metadata_frame_error.kind, frame_error.kind);
    assert_eq!(metadata_frame_error.message, frame_error.message);

    let topocentric_request = EphemerisRequest {
        observer: Some(ObserverLocation::new(
            Latitude::from_degrees(51.5),
            Longitude::from_degrees(-0.1),
            Some(45.0),
        )),
        apparent: Apparentness::Mean,
        ..apparent_request.clone()
    };
    let error = validate_request_against_metadata(&topocentric_request, &metadata)
        .expect_err("metadata preflight should reject observer-bearing geocentric requests");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedObserver);
    assert!(error.message.contains("toy backend is geocentric only"));
    assert!(error.message.contains(
        &topocentric_request
            .observer
            .as_ref()
            .unwrap()
            .summary_line()
    ));

    let invalid_observer_request = EphemerisRequest {
        observer: Some(ObserverLocation::new(
            Latitude::from_degrees(95.0),
            Longitude::from_degrees(-0.1),
            Some(45.0),
        )),
        ..apparent_request.clone()
    };
    let routing_metadata = BackendMetadata {
        family: BackendFamily::Composite,
        capabilities: BackendCapabilities {
            topocentric: true,
            ..metadata.capabilities.clone()
        },
        ..metadata.clone()
    };
    let routing_error =
        validate_request_against_metadata(&invalid_observer_request, &routing_metadata)
            .expect_err("routing metadata should still reject invalid observer locations");
    assert_eq!(routing_error.kind, EphemerisErrorKind::InvalidObserver);
    assert!(routing_error
        .message
        .contains("request received invalid observer location"));

    let geocentric_only_request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt),
    );
    let geocentric_only_invalid_observer_request = EphemerisRequest {
        observer: Some(ObserverLocation::new(
            Latitude::from_degrees(95.0),
            Longitude::from_degrees(-0.1),
            Some(45.0),
        )),
        ..geocentric_only_request.clone()
    };
    let geocentric_only_error = validate_request_against_metadata(
            &geocentric_only_invalid_observer_request,
            &metadata,
        )
        .expect_err("geocentric-only metadata should still reject malformed observer locations as invalid input");
    assert_eq!(
        geocentric_only_error.kind,
        EphemerisErrorKind::InvalidObserver
    );
    assert!(geocentric_only_error
        .message
        .contains("request received invalid observer location"));

    let geocentric_only_batch_error = validate_requests_against_metadata(
        &[
            geocentric_only_request.clone(),
            geocentric_only_invalid_observer_request.clone(),
        ],
        &metadata,
    )
    .expect_err(
        "batch metadata should preserve invalid observer precedence for geocentric-only requests",
    );
    assert_eq!(
        geocentric_only_batch_error.kind,
        EphemerisErrorKind::InvalidObserver
    );
    assert!(geocentric_only_batch_error
        .message
        .contains("batch request 2: request received invalid observer location"));

    let invalid_observer_frame_request = EphemerisRequest {
        observer: Some(ObserverLocation::new(
            Latitude::from_degrees(95.0),
            Longitude::from_degrees(-0.1),
            Some(45.0),
        )),
        ..frame_request.clone()
    };
    let invalid_observer_frame_error =
        validate_request_against_metadata(&invalid_observer_frame_request, &metadata)
            .expect_err("frame policy should still win before malformed observer validation");
    assert_eq!(
        invalid_observer_frame_error.kind,
        EphemerisErrorKind::UnsupportedCoordinateFrame
    );
    assert_eq!(
        invalid_observer_frame_error.message,
        "toy backend only returns [Ecliptic] coordinates"
    );

    let invalid_observer_time_scale_request = EphemerisRequest {
        instant: Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tdb),
        observer: Some(ObserverLocation::new(
            Latitude::from_degrees(95.0),
            Longitude::from_degrees(-0.1),
            Some(45.0),
        )),
        ..geocentric_only_request.clone()
    };
    let invalid_observer_time_scale_error =
        validate_request_against_metadata(&invalid_observer_time_scale_request, &metadata)
            .expect_err("time-scale policy should still win before malformed observer validation");
    assert_eq!(
        invalid_observer_time_scale_error.kind,
        EphemerisErrorKind::UnsupportedTimeScale
    );
    assert_eq!(
        invalid_observer_time_scale_error.message,
        "toy backend expects one of [TT] for request instants"
    );

    let invalid_observer_apparent_request = EphemerisRequest {
        observer: Some(ObserverLocation::new(
            Latitude::from_degrees(95.0),
            Longitude::from_degrees(-0.1),
            Some(45.0),
        )),
        ..apparent_request.clone()
    };
    let invalid_observer_apparent_error =
        validate_request_against_metadata(&invalid_observer_apparent_request, &metadata)
            .expect_err("apparentness should still win before malformed observer validation");
    assert_eq!(
        invalid_observer_apparent_error.kind,
        EphemerisErrorKind::UnsupportedApparentness
    );
    assert_eq!(
            invalid_observer_apparent_error.message,
            "toy backend currently returns mean geometric coordinates only; apparent corrections are not implemented"
        );

    let invalid_observer_frame_batch_error = validate_requests_against_metadata(
        &[
            geocentric_only_request.clone(),
            invalid_observer_frame_request.clone(),
        ],
        &metadata,
    )
    .expect_err(
        "batch metadata should preserve frame precedence before invalid observer validation",
    );
    assert_eq!(
        invalid_observer_frame_batch_error.kind,
        EphemerisErrorKind::UnsupportedCoordinateFrame
    );
    assert_eq!(
        invalid_observer_frame_batch_error.message,
        "batch request 2: toy backend only returns [Ecliptic] coordinates"
    );

    let unsupported_body_request = EphemerisRequest {
        body: CelestialBody::Mars,
        frame: CoordinateFrame::Ecliptic,
        apparent: Apparentness::Mean,
        observer: None,
        ..frame_request.clone()
    };
    let error = validate_request_against_metadata(&unsupported_body_request, &metadata)
        .expect_err("metadata preflight should reject bodies outside the declared coverage");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedBody);
    assert_eq!(error.message, "toy backend does not support Mars");

    let unsupported_body_with_invalid_observer_request = EphemerisRequest {
        observer: Some(ObserverLocation::new(
            Latitude::from_degrees(95.0),
            Longitude::from_degrees(-0.1),
            Some(45.0),
        )),
        ..unsupported_body_request.clone()
    };
    let unsupported_body_with_invalid_observer_error = validate_request_against_metadata(
        &unsupported_body_with_invalid_observer_request,
        &metadata,
    )
    .expect_err("observer validation should win before unsupported body coverage is reported");
    assert_eq!(
        unsupported_body_with_invalid_observer_error.kind,
        EphemerisErrorKind::InvalidObserver
    );
    assert!(unsupported_body_with_invalid_observer_error
        .message
        .contains("request received invalid observer location"));

    let unsupported_body_batch_error = validate_requests_against_metadata(
        &[
            geocentric_only_request.clone(),
            unsupported_body_with_invalid_observer_request.clone(),
        ],
        &metadata,
    )
    .expect_err(
        "batch metadata should preserve invalid observer precedence over unsupported bodies",
    );
    assert_eq!(
        unsupported_body_batch_error.kind,
        EphemerisErrorKind::InvalidObserver
    );
    assert!(unsupported_body_batch_error
        .message
        .contains("batch request 2: request received invalid observer location"));

    let sidereal_request = EphemerisRequest {
        zodiac_mode: ZodiacMode::Sidereal {
            ayanamsa: pleiades_types::Ayanamsa::FaganBradley,
        },
        frame: CoordinateFrame::Ecliptic,
        apparent: Apparentness::Mean,
        observer: None,
        ..frame_request.clone()
    };
    let error = validate_request_against_metadata(&sidereal_request, &metadata)
        .expect_err("sidereal requests should be rejected when metadata stays tropical-only");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedZodiacMode);
    assert!(error.message.contains("tropical coordinates only"));

    let sidereal_invalid_observer_request = EphemerisRequest {
        observer: Some(ObserverLocation::new(
            Latitude::from_degrees(95.0),
            Longitude::from_degrees(-0.1),
            Some(45.0),
        )),
        ..sidereal_request.clone()
    };
    let error = validate_request_against_metadata(&sidereal_invalid_observer_request, &metadata)
        .expect_err("sidereal routing should win before malformed observer validation");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedZodiacMode);
    assert!(error.message.contains("tropical coordinates only"));
    assert!(!error.message.contains("invalid observer location"));

    let unsupported_body_apparent_request = EphemerisRequest {
        body: CelestialBody::Mars,
        frame: CoordinateFrame::Ecliptic,
        apparent: Apparentness::Apparent,
        ..frame_request.clone()
    };
    let unsupported_body_apparent_error =
        validate_request_against_metadata(&unsupported_body_apparent_request, &metadata)
            .expect_err("apparentness should win before unsupported body coverage");
    assert_eq!(
        unsupported_body_apparent_error.kind,
        EphemerisErrorKind::UnsupportedApparentness
    );
    assert_eq!(unsupported_body_apparent_error.message, "toy backend currently returns mean geometric coordinates only; apparent corrections are not implemented");

    let unsupported_body_apparent_batch_error = validate_requests_against_metadata(
        &[
            geocentric_only_request.clone(),
            unsupported_body_apparent_request.clone(),
        ],
        &metadata,
    )
    .expect_err("batch validation should preserve apparentness precedence before body coverage");
    assert_eq!(
        unsupported_body_apparent_batch_error.kind,
        EphemerisErrorKind::UnsupportedApparentness
    );
    assert_eq!(unsupported_body_apparent_batch_error.message, "batch request 2: toy backend currently returns mean geometric coordinates only; apparent corrections are not implemented");

    let sidereal_metadata = BackendMetadata {
        capabilities: BackendCapabilities {
            native_sidereal: true,
            ..metadata.capabilities.clone()
        },
        ..metadata.clone()
    };
    assert!(validate_request_against_metadata(&sidereal_request, &sidereal_metadata).is_ok());

    let invalid_custom_body_request = EphemerisRequest {
        body: CelestialBody::Custom(pleiades_types::CustomBodyId::new("asteroid", " 433-Eros ")),
        ..frame_request.clone()
    };
    let invalid_custom_body_error = validate_request_against_metadata(
        &invalid_custom_body_request,
        &BackendMetadata {
            body_coverage: vec![CelestialBody::Custom(pleiades_types::CustomBodyId::new(
                "asteroid",
                " 433-Eros ",
            ))],
            ..metadata.clone()
        },
    )
    .expect_err("custom body identifiers should validate before metadata dispatch");
    assert_eq!(
        invalid_custom_body_error.kind,
        EphemerisErrorKind::InvalidRequest
    );
    assert!(invalid_custom_body_error
            .message
            .contains("request body is invalid: custom body id designation must not have leading or trailing whitespace"));

    let invalid_custom_ayanamsa_request = EphemerisRequest {
        zodiac_mode: ZodiacMode::Sidereal {
            ayanamsa: pleiades_types::Ayanamsa::Custom(pleiades_types::CustomAyanamsa {
                name: "  ".to_string(),
                description: Some("local calibration".to_string()),
                epoch: Some(pleiades_types::JulianDay::from_days(2451545.0)),
                offset_degrees: Some(pleiades_types::Angle::from_degrees(24.0)),
            }),
        },
        ..frame_request.clone()
    };
    let invalid_custom_ayanamsa_error =
        validate_request_against_metadata(&invalid_custom_ayanamsa_request, &sidereal_metadata)
            .expect_err("custom ayanamsas should validate before sidereal request dispatch");
    assert_eq!(
        invalid_custom_ayanamsa_error.kind,
        EphemerisErrorKind::InvalidRequest
    );
    assert!(invalid_custom_ayanamsa_error
        .message
        .contains("sidereal ayanamsa is invalid: custom ayanamsa name must not be blank"));

    let error = validate_zodiac_policy(&sidereal_request, "toy backend", &[ZodiacMode::Tropical])
        .expect_err("sidereal requests should be rejected when only tropical output is supported");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedZodiacMode);
    assert!(error.message.contains("tropical coordinates only"));
    let request_policy = current_request_policy_summary();
    assert_eq!(
            request_policy.time_scale,
            "direct backend requests accept TT/TDB; UTC/UT1 inputs require caller-supplied conversion helpers; no built-in Delta T or UTC convenience model"
        );
    assert_eq!(
            request_policy.observer,
            "chart houses use observer locations; chart body observers stay separate; body requests stay geocentric; geocentric-only backends reject observer-bearing requests with UnsupportedObserver; malformed observer coordinates remain InvalidObserver; topocentric body positions remain unsupported"
        );
    assert_eq!(
            request_policy.apparentness,
            "current first-party backends accept mean geometric output only; apparent-place corrections are rejected unless a backend explicitly advertises support"
        );
    assert_eq!(
            request_policy.frame,
            "ecliptic body positions are the default request shape; equatorial output is backend-specific and derived via mean-obliquity transforms when supported; supported equatorial precision is bounded by the shared mean-obliquity frame round-trip envelope; native sidereal backend output remains unsupported unless a backend explicitly advertises it"
        );
    assert_eq!(
        time_scale_policy_summary_for_report().summary_line(),
        request_policy.time_scale
    );
    assert_eq!(
        observer_policy_summary_for_report().summary_line(),
        request_policy.observer
    );
    assert_eq!(
        apparentness_policy_summary_for_report().summary_line(),
        request_policy.apparentness
    );
    assert_eq!(frame_policy_summary_for_report(), request_policy.frame);
    assert_eq!(
        zodiac_policy_summary_for_report(&[ZodiacMode::Tropical]),
        "tropical only"
    );

    let observer_request = EphemerisRequest {
        observer: Some(ObserverLocation::new(
            Latitude::from_degrees(51.5),
            Longitude::from_degrees(-0.1),
            Some(45.0),
        )),
        ..apparent_request.clone()
    };
    let error = validate_observer_policy(&observer_request, "toy backend", false)
        .expect_err("topocentric requests should be rejected when unsupported");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedObserver);
    assert!(error.message.contains("toy backend is geocentric only"));
    assert!(error
        .message
        .contains(&observer_request.observer.as_ref().unwrap().summary_line()));
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

#[test]
fn validate_requests_against_metadata_rejects_unsupported_batch_backends() {
    let metadata = BackendMetadata {
        id: BackendId::new("toy backend"),
        version: "0.1.0".to_string(),
        family: BackendFamily::Algorithmic,
        provenance: BackendProvenance::new("toy backend"),
        nominal_range: TimeRange::new(None, None),
        supported_time_scales: vec![TimeScale::Tt],
        body_coverage: vec![CelestialBody::Sun],
        supported_frames: vec![CoordinateFrame::Ecliptic],
        capabilities: BackendCapabilities {
            batch: false,
            ..BackendCapabilities::default()
        },
        accuracy: AccuracyClass::Approximate,
        deterministic: true,
        offline: true,
    };
    let request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
    );

    let error = validate_requests_against_metadata(&[request], &metadata).expect_err(
        "batch requests should be rejected when the backend does not advertise batch support",
    );
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert_eq!(error.message, "toy backend does not support batch requests");
}

#[test]
fn validate_requests_against_metadata_preserves_mixed_time_scales_and_topocentric_requests_when_supported(
) {
    struct EchoSunBackend;

    impl EphemerisBackend for EchoSunBackend {
        fn metadata(&self) -> BackendMetadata {
            BackendMetadata {
                id: BackendId::new("echo-sun"),
                version: "0.1.0".to_string(),
                family: BackendFamily::Algorithmic,
                provenance: BackendProvenance::new("echo Sun backend"),
                nominal_range: TimeRange::new(None, None),
                supported_time_scales: vec![TimeScale::Tt, TimeScale::Tdb],
                body_coverage: vec![CelestialBody::Sun],
                supported_frames: vec![CoordinateFrame::Ecliptic],
                capabilities: BackendCapabilities {
                    batch: true,
                    apparent: true,
                    topocentric: true,
                    ..BackendCapabilities::default()
                },
                accuracy: AccuracyClass::Approximate,
                deterministic: true,
                offline: true,
            }
        }

        fn supports_body(&self, body: CelestialBody) -> bool {
            body == CelestialBody::Sun
        }

        fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
            Ok(EphemerisResult::new(
                BackendId::new("echo-sun"),
                req.body.clone(),
                req.instant,
                req.frame,
                req.zodiac_mode.clone(),
                req.apparent,
            ))
        }
    }

    let backend = EchoSunBackend;
    let metadata = backend.metadata();

    let geocentric_request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
    );
    let mut topocentric_request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb),
    );
    topocentric_request.observer = Some(ObserverLocation::new(
        Latitude::from_degrees(51.5),
        Longitude::from_degrees(12.5),
        Some(0.0),
    ));

    validate_requests_against_metadata(
        &[geocentric_request.clone(), topocentric_request.clone()],
        &metadata,
    )
    .expect("batch preflight should preserve mixed TT/TDB and topocentric requests when supported");

    let results = backend
        .positions(&[geocentric_request, topocentric_request])
        .expect("batch adapter should preserve the validated request shapes");
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].backend_id.as_str(), "echo-sun");
    assert_eq!(results[1].backend_id.as_str(), "echo-sun");
    assert_eq!(results[0].instant.scale, TimeScale::Tt);
    assert_eq!(results[1].instant.scale, TimeScale::Tdb);
}

#[test]
fn routing_metadata_defers_request_shape_checks_to_the_selected_provider() {
    struct RejectingSunBackend;

    impl EphemerisBackend for RejectingSunBackend {
        fn metadata(&self) -> BackendMetadata {
            BackendMetadata {
                id: BackendId::new("rejecting-sun"),
                version: "0.1.0".to_string(),
                family: BackendFamily::Algorithmic,
                provenance: BackendProvenance::new("rejecting Sun backend"),
                nominal_range: TimeRange::new(None, None),
                supported_time_scales: vec![TimeScale::Tt],
                body_coverage: vec![CelestialBody::Sun],
                supported_frames: vec![CoordinateFrame::Ecliptic],
                capabilities: BackendCapabilities {
                    batch: false,
                    apparent: false,
                    topocentric: false,
                    ..BackendCapabilities::default()
                },
                accuracy: AccuracyClass::Approximate,
                deterministic: true,
                offline: true,
            }
        }

        fn supports_body(&self, body: CelestialBody) -> bool {
            body == CelestialBody::Sun
        }

        fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
            if req.observer.is_some() {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::UnsupportedObserver,
                    "rejecting Sun backend is geocentric only",
                ));
            }

            if req.apparent == Apparentness::Apparent {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    "rejecting Sun backend only returns mean geometric coordinates",
                ));
            }

            Ok(EphemerisResult::new(
                BackendId::new("rejecting-sun"),
                req.body.clone(),
                req.instant,
                req.frame,
                req.zodiac_mode.clone(),
                req.apparent,
            ))
        }
    }

    struct AcceptingSunBackend;

    impl EphemerisBackend for AcceptingSunBackend {
        fn metadata(&self) -> BackendMetadata {
            BackendMetadata {
                id: BackendId::new("accepting-sun"),
                version: "0.1.0".to_string(),
                family: BackendFamily::Algorithmic,
                provenance: BackendProvenance::new("accepting Sun backend"),
                nominal_range: TimeRange::new(None, None),
                supported_time_scales: vec![TimeScale::Tt],
                body_coverage: vec![CelestialBody::Sun],
                supported_frames: vec![CoordinateFrame::Ecliptic],
                capabilities: BackendCapabilities {
                    batch: true,
                    apparent: true,
                    topocentric: true,
                    ..BackendCapabilities::default()
                },
                accuracy: AccuracyClass::Approximate,
                deterministic: true,
                offline: true,
            }
        }

        fn supports_body(&self, body: CelestialBody) -> bool {
            body == CelestialBody::Sun
        }

        fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
            Ok(EphemerisResult::new(
                BackendId::new("accepting-sun"),
                req.body.clone(),
                req.instant,
                req.frame,
                req.zodiac_mode.clone(),
                req.apparent,
            ))
        }
    }

    let routing = RoutingBackend::new(vec![
        Box::new(RejectingSunBackend),
        Box::new(AcceptingSunBackend),
    ]);
    let mut request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
    );
    request.observer = Some(ObserverLocation::new(
        Latitude::from_degrees(51.5),
        Longitude::from_degrees(12.5),
        Some(0.0),
    ));
    request.apparent = Apparentness::Apparent;

    let metadata = routing.metadata();
    assert!(metadata.family.is_routing());
    assert!(!metadata.capabilities.batch);

    validate_requests_against_metadata(&[request.clone()], &metadata)
        .expect("routing metadata should defer request-shape checks to the selected provider");

    let result = routing
        .positions(&[request])
        .expect("routing should recover through the secondary provider");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].backend_id.as_str(), "accepting-sun");
}

#[test]
fn routing_backend_batch_metadata_defers_observer_and_apparentness_checks_to_the_selected_provider()
{
    struct RejectingSunBackend;

    impl EphemerisBackend for RejectingSunBackend {
        fn metadata(&self) -> BackendMetadata {
            BackendMetadata {
                id: BackendId::new("rejecting-sun-batch"),
                version: "0.1.0".to_string(),
                family: BackendFamily::Algorithmic,
                provenance: BackendProvenance::new("rejecting Sun batch backend"),
                nominal_range: TimeRange::new(None, None),
                supported_time_scales: vec![TimeScale::Tt, TimeScale::Tdb],
                body_coverage: vec![CelestialBody::Sun],
                supported_frames: vec![CoordinateFrame::Ecliptic],
                capabilities: BackendCapabilities {
                    batch: false,
                    apparent: false,
                    topocentric: false,
                    ..BackendCapabilities::default()
                },
                accuracy: AccuracyClass::Approximate,
                deterministic: true,
                offline: true,
            }
        }

        fn supports_body(&self, body: CelestialBody) -> bool {
            body == CelestialBody::Sun
        }

        fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
            if req.observer.is_some() {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::UnsupportedObserver,
                    "rejecting Sun batch backend is geocentric only",
                ));
            }

            if req.apparent == Apparentness::Apparent {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    "rejecting Sun batch backend only returns mean geometric coordinates",
                ));
            }

            Ok(EphemerisResult::new(
                BackendId::new("rejecting-sun-batch"),
                req.body.clone(),
                req.instant,
                req.frame,
                req.zodiac_mode.clone(),
                req.apparent,
            ))
        }
    }

    struct AcceptingSunBackend;

    impl EphemerisBackend for AcceptingSunBackend {
        fn metadata(&self) -> BackendMetadata {
            BackendMetadata {
                id: BackendId::new("accepting-sun-batch"),
                version: "0.1.0".to_string(),
                family: BackendFamily::Algorithmic,
                provenance: BackendProvenance::new("accepting Sun batch backend"),
                nominal_range: TimeRange::new(None, None),
                supported_time_scales: vec![TimeScale::Tt, TimeScale::Tdb],
                body_coverage: vec![CelestialBody::Sun],
                supported_frames: vec![CoordinateFrame::Ecliptic],
                capabilities: BackendCapabilities {
                    batch: true,
                    apparent: true,
                    topocentric: true,
                    ..BackendCapabilities::default()
                },
                accuracy: AccuracyClass::Approximate,
                deterministic: true,
                offline: true,
            }
        }

        fn supports_body(&self, body: CelestialBody) -> bool {
            body == CelestialBody::Sun
        }

        fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
            Ok(EphemerisResult::new(
                BackendId::new("accepting-sun-batch"),
                req.body.clone(),
                req.instant,
                req.frame,
                req.zodiac_mode.clone(),
                req.apparent,
            ))
        }
    }

    let routing = RoutingBackend::new(vec![
        Box::new(RejectingSunBackend),
        Box::new(AcceptingSunBackend),
    ]);
    let mut geocentric_request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
    );
    geocentric_request.observer = Some(ObserverLocation::new(
        Latitude::from_degrees(51.5),
        Longitude::from_degrees(12.5),
        Some(0.0),
    ));
    let mut apparent_request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb),
    );
    apparent_request.apparent = Apparentness::Apparent;

    let metadata = routing.metadata();
    assert!(metadata.family.is_routing());
    assert!(!metadata.capabilities.batch);

    validate_requests_against_metadata(
        &[geocentric_request.clone(), apparent_request.clone()],
        &metadata,
    )
    .expect(
        "routing metadata should defer observer and apparentness checks to the selected provider",
    );

    let results = routing
        .positions(&[geocentric_request, apparent_request])
        .expect("routing should recover through the secondary provider for batch requests");
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].backend_id.as_str(), "accepting-sun-batch");
    assert_eq!(results[1].backend_id.as_str(), "accepting-sun-batch");
    assert_eq!(results[0].instant.scale, TimeScale::Tt);
    assert_eq!(results[1].instant.scale, TimeScale::Tdb);
}

#[test]
fn validate_requests_against_metadata_rejects_sidereal_requests_with_batch_index_prefix() {
    let metadata = BackendMetadata {
        id: BackendId::new("toy backend"),
        version: "0.1.0".to_string(),
        family: BackendFamily::Algorithmic,
        provenance: BackendProvenance::new("toy backend"),
        nominal_range: TimeRange::new(None, None),
        supported_time_scales: vec![TimeScale::Tt],
        body_coverage: vec![CelestialBody::Sun],
        supported_frames: vec![CoordinateFrame::Ecliptic],
        capabilities: BackendCapabilities::default(),
        accuracy: AccuracyClass::Approximate,
        deterministic: true,
        offline: true,
    };
    let tropical_request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
    );
    let sidereal_request = EphemerisRequest {
        zodiac_mode: ZodiacMode::Sidereal {
            ayanamsa: pleiades_types::Ayanamsa::FaganBradley,
        },
        ..tropical_request.clone()
    };

    let error =
        validate_requests_against_metadata(&[tropical_request, sidereal_request], &metadata)
            .expect_err("the batch helper should preserve sidereal request failures");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedZodiacMode);
    assert_eq!(
        error.message,
        "batch request 2: toy backend currently exposes tropical coordinates only"
    );
}

#[test]
fn validate_requests_against_metadata_rejects_apparent_requests_with_batch_index_prefix() {
    let metadata = BackendMetadata {
        id: BackendId::new("toy backend"),
        version: "0.1.0".to_string(),
        family: BackendFamily::Algorithmic,
        provenance: BackendProvenance::new("toy backend"),
        nominal_range: TimeRange::new(None, None),
        supported_time_scales: vec![TimeScale::Tt],
        body_coverage: vec![CelestialBody::Sun],
        supported_frames: vec![CoordinateFrame::Ecliptic],
        capabilities: BackendCapabilities {
            apparent: false,
            ..BackendCapabilities::default()
        },
        accuracy: AccuracyClass::Approximate,
        deterministic: true,
        offline: true,
    };
    let mean_request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
    );
    let apparent_request = EphemerisRequest {
        apparent: Apparentness::Apparent,
        ..mean_request.clone()
    };

    let error = validate_requests_against_metadata(&[mean_request, apparent_request], &metadata)
        .expect_err("the batch helper should preserve apparentness failures");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedApparentness);
    assert_eq!(
            error.message,
            "batch request 2: toy backend currently returns mean geometric coordinates only; apparent corrections are not implemented"
        );
}

#[test]
fn validate_requests_against_metadata_rejects_apparent_requests_before_topocentric_checks() {
    let metadata = BackendMetadata {
        id: BackendId::new("toy backend"),
        version: "0.1.0".to_string(),
        family: BackendFamily::Algorithmic,
        provenance: BackendProvenance::new("toy backend"),
        nominal_range: TimeRange::new(None, None),
        supported_time_scales: vec![TimeScale::Tt],
        body_coverage: vec![CelestialBody::Sun],
        supported_frames: vec![CoordinateFrame::Ecliptic],
        capabilities: BackendCapabilities {
            apparent: false,
            ..BackendCapabilities::default()
        },
        accuracy: AccuracyClass::Approximate,
        deterministic: true,
        offline: true,
    };
    let request = EphemerisRequest {
        apparent: Apparentness::Apparent,
        observer: Some(ObserverLocation::new(
            Latitude::from_degrees(51.5),
            Longitude::from_degrees(-0.1),
            Some(45.0),
        )),
        ..EphemerisRequest::new(
            CelestialBody::Sun,
            Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
        )
    };

    let error = validate_requests_against_metadata(&[request], &metadata)
        .expect_err("apparentness should be checked before the observer policy");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedApparentness);
    assert!(error.message.contains(
            "toy backend currently returns mean geometric coordinates only; apparent corrections are not implemented"
        ));
    assert!(!error
        .message
        .contains("topocentric positions are not implemented"));
}

#[test]
fn validate_requests_against_metadata_rejects_apparent_requests_before_topocentric_checks_in_batches(
) {
    let metadata = BackendMetadata {
        id: BackendId::new("toy backend"),
        version: "0.1.0".to_string(),
        family: BackendFamily::Algorithmic,
        provenance: BackendProvenance::new("toy backend"),
        nominal_range: TimeRange::new(None, None),
        supported_time_scales: vec![TimeScale::Tt],
        body_coverage: vec![CelestialBody::Sun],
        supported_frames: vec![CoordinateFrame::Ecliptic],
        capabilities: BackendCapabilities {
            apparent: false,
            ..BackendCapabilities::default()
        },
        accuracy: AccuracyClass::Approximate,
        deterministic: true,
        offline: true,
    };
    let geocentric_request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
    );
    let apparent_topocentric_request = EphemerisRequest {
        apparent: Apparentness::Apparent,
        observer: Some(ObserverLocation::new(
            Latitude::from_degrees(51.5),
            Longitude::from_degrees(-0.1),
            Some(45.0),
        )),
        ..geocentric_request.clone()
    };

    let error = validate_requests_against_metadata(
        &[geocentric_request, apparent_topocentric_request],
        &metadata,
    )
    .expect_err("the batch helper should preserve apparentness failures before observer checks");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedApparentness);
    assert!(error.message.contains(
            "batch request 2: toy backend currently returns mean geometric coordinates only; apparent corrections are not implemented"
        ));
    assert!(!error
        .message
        .contains("topocentric positions are not implemented"));
}

#[test]
fn validate_requests_against_metadata_rejects_apparent_requests_before_sidereal_checks_in_batches()
{
    let metadata = BackendMetadata {
        id: BackendId::new("toy backend"),
        version: "0.1.0".to_string(),
        family: BackendFamily::Algorithmic,
        provenance: BackendProvenance::new("toy backend"),
        nominal_range: TimeRange::new(None, None),
        supported_time_scales: vec![TimeScale::Tt],
        body_coverage: vec![CelestialBody::Sun],
        supported_frames: vec![CoordinateFrame::Ecliptic],
        capabilities: BackendCapabilities {
            apparent: false,
            ..BackendCapabilities::default()
        },
        accuracy: AccuracyClass::Approximate,
        deterministic: true,
        offline: true,
    };
    let geocentric_request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
    );
    let apparent_sidereal_request = EphemerisRequest {
        apparent: Apparentness::Apparent,
        zodiac_mode: ZodiacMode::Sidereal {
            ayanamsa: pleiades_types::Ayanamsa::FaganBradley,
        },
        observer: Some(ObserverLocation::new(
            Latitude::from_degrees(51.5),
            Longitude::from_degrees(-0.1),
            Some(45.0),
        )),
        ..geocentric_request.clone()
    };

    let error = validate_requests_against_metadata(
        &[geocentric_request, apparent_sidereal_request],
        &metadata,
    )
    .expect_err("the batch helper should preserve apparentness failures before sidereal checks");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedApparentness);
    assert!(error.message.contains(
            "batch request 2: toy backend currently returns mean geometric coordinates only; apparent corrections are not implemented"
        ));
    assert!(!error.message.contains("exposes tropical coordinates only"));
    assert!(!error
        .message
        .contains("topocentric positions are not implemented"));
}

#[test]
fn validate_requests_against_metadata_rejects_apparent_requests_before_sidereal_and_observer_checks_in_batches(
) {
    let metadata = BackendMetadata {
        id: BackendId::new("toy backend"),
        version: "0.1.0".to_string(),
        family: BackendFamily::Algorithmic,
        provenance: BackendProvenance::new("toy backend"),
        nominal_range: TimeRange::new(None, None),
        supported_time_scales: vec![TimeScale::Tt],
        body_coverage: vec![CelestialBody::Sun],
        supported_frames: vec![CoordinateFrame::Ecliptic],
        capabilities: BackendCapabilities {
            apparent: false,
            ..BackendCapabilities::default()
        },
        accuracy: AccuracyClass::Approximate,
        deterministic: true,
        offline: true,
    };
    let geocentric_request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
    );
    let apparent_sidereal_topocentric_request = EphemerisRequest {
        apparent: Apparentness::Apparent,
        zodiac_mode: ZodiacMode::Sidereal {
            ayanamsa: pleiades_types::Ayanamsa::FaganBradley,
        },
        observer: Some(ObserverLocation::new(
            Latitude::from_degrees(51.5),
            Longitude::from_degrees(-0.1),
            Some(45.0),
        )),
        ..geocentric_request.clone()
    };

    let error = validate_requests_against_metadata(
            &[geocentric_request, apparent_sidereal_topocentric_request],
            &metadata,
        )
        .expect_err(
            "the batch helper should preserve apparentness failures before sidereal and observer checks",
        );
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedApparentness);
    assert!(error.message.contains(
            "batch request 2: toy backend currently returns mean geometric coordinates only; apparent corrections are not implemented"
        ));
    assert!(!error.message.contains("exposes tropical coordinates only"));
    assert!(!error
        .message
        .contains("topocentric positions are not implemented"));
}

#[test]
fn validate_requests_against_metadata_rejects_unsupported_time_scales_before_apparentness_checks_in_batches(
) {
    let metadata = BackendMetadata {
        id: BackendId::new("toy backend"),
        version: "0.1.0".to_string(),
        family: BackendFamily::Algorithmic,
        provenance: BackendProvenance::new("toy backend"),
        nominal_range: TimeRange::new(None, None),
        supported_time_scales: vec![TimeScale::Tt],
        body_coverage: vec![CelestialBody::Sun],
        supported_frames: vec![CoordinateFrame::Ecliptic],
        capabilities: BackendCapabilities {
            apparent: false,
            ..BackendCapabilities::default()
        },
        accuracy: AccuracyClass::Approximate,
        deterministic: true,
        offline: true,
    };
    let geocentric_request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
    );
    let unsupported_time_scale_apparent_request = EphemerisRequest {
        instant: Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Utc),
        apparent: Apparentness::Apparent,
        ..geocentric_request.clone()
    };

    let error = validate_requests_against_metadata(
        &[geocentric_request, unsupported_time_scale_apparent_request],
        &metadata,
    )
    .expect_err("time-scale failures should be reported before apparentness failures in batches");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedTimeScale);
    assert_eq!(
        error.message,
        "batch request 2: toy backend expects one of [TT] for request instants"
    );
    assert!(!error
        .message
        .contains("apparent corrections are not implemented"));
}

#[test]
fn validate_requests_against_metadata_rejects_unsupported_time_scales_before_apparentness_and_observer_checks_in_batches(
) {
    let metadata = BackendMetadata {
        id: BackendId::new("toy backend"),
        version: "0.1.0".to_string(),
        family: BackendFamily::Algorithmic,
        provenance: BackendProvenance::new("toy backend"),
        nominal_range: TimeRange::new(None, None),
        supported_time_scales: vec![TimeScale::Tt],
        body_coverage: vec![CelestialBody::Sun],
        supported_frames: vec![CoordinateFrame::Ecliptic],
        capabilities: BackendCapabilities {
            apparent: false,
            ..BackendCapabilities::default()
        },
        accuracy: AccuracyClass::Approximate,
        deterministic: true,
        offline: true,
    };
    let geocentric_request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
    );
    let unsupported_time_scale_apparent_observer_request = EphemerisRequest {
        instant: Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Utc),
        apparent: Apparentness::Apparent,
        observer: Some(ObserverLocation::new(
            Latitude::from_degrees(95.0),
            Longitude::from_degrees(-0.1),
            Some(45.0),
        )),
        ..geocentric_request.clone()
    };

    let error = validate_requests_against_metadata(
            &[geocentric_request, unsupported_time_scale_apparent_observer_request],
            &metadata,
        )
        .expect_err(
            "time-scale failures should be reported before apparentness and observer failures in batches",
        );
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedTimeScale);
    assert_eq!(
        error.message,
        "batch request 2: toy backend expects one of [TT] for request instants"
    );
    assert!(!error
        .message
        .contains("apparent corrections are not implemented"));
    assert!(!error
        .message
        .contains("request received invalid observer location"));
}

#[test]
fn validate_requests_against_metadata_rejects_unsupported_time_scales_before_invalid_observer_checks_in_batches(
) {
    let metadata = BackendMetadata {
        id: BackendId::new("toy backend"),
        version: "0.1.0".to_string(),
        family: BackendFamily::Algorithmic,
        provenance: BackendProvenance::new("toy backend"),
        nominal_range: TimeRange::new(None, None),
        supported_time_scales: vec![TimeScale::Tt],
        body_coverage: vec![CelestialBody::Sun],
        supported_frames: vec![CoordinateFrame::Ecliptic],
        capabilities: BackendCapabilities::default(),
        accuracy: AccuracyClass::Approximate,
        deterministic: true,
        offline: true,
    };
    let geocentric_request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
    );
    let unsupported_time_scale_invalid_observer_request = EphemerisRequest {
        instant: Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Utc),
        observer: Some(ObserverLocation::new(
            Latitude::from_degrees(95.0),
            Longitude::from_degrees(-0.1),
            Some(45.0),
        )),
        ..geocentric_request.clone()
    };

    let error = validate_requests_against_metadata(
        &[
            geocentric_request,
            unsupported_time_scale_invalid_observer_request,
        ],
        &metadata,
    )
    .expect_err(
        "time-scale failures should be reported before invalid observer failures in batches",
    );
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedTimeScale);
    assert_eq!(
        error.message,
        "batch request 2: toy backend expects one of [TT] for request instants"
    );
    assert!(!error
        .message
        .contains("request received invalid observer location"));
}

#[test]
fn validate_requests_against_metadata_rejects_topocentric_requests_with_batch_index_prefix() {
    let metadata = BackendMetadata {
        id: BackendId::new("toy backend"),
        version: "0.1.0".to_string(),
        family: BackendFamily::Algorithmic,
        provenance: BackendProvenance::new("toy backend"),
        nominal_range: TimeRange::new(None, None),
        supported_time_scales: vec![TimeScale::Tt],
        body_coverage: vec![CelestialBody::Sun],
        supported_frames: vec![CoordinateFrame::Ecliptic],
        capabilities: BackendCapabilities::default(),
        accuracy: AccuracyClass::Approximate,
        deterministic: true,
        offline: true,
    };
    let geocentric_request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
    );
    let topocentric_request = EphemerisRequest {
        observer: Some(ObserverLocation::new(
            Latitude::from_degrees(51.5),
            Longitude::from_degrees(-0.1),
            Some(45.0),
        )),
        ..geocentric_request.clone()
    };
    let topocentric_summary = topocentric_request
        .observer
        .as_ref()
        .expect("observer should be present")
        .summary_line();

    let error =
        validate_requests_against_metadata(&[geocentric_request, topocentric_request], &metadata)
            .expect_err("the batch helper should preserve observer failures");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedObserver);
    assert!(error
        .message
        .contains("batch request 2: toy backend is geocentric only"));
    assert!(error.message.contains(&topocentric_summary));
}

#[test]
fn validate_requests_against_metadata_rejects_sidereal_requests_before_topocentric_checks_in_batches(
) {
    let metadata = BackendMetadata {
        id: BackendId::new("toy backend"),
        version: "0.1.0".to_string(),
        family: BackendFamily::Algorithmic,
        provenance: BackendProvenance::new("toy backend"),
        nominal_range: TimeRange::new(None, None),
        supported_time_scales: vec![TimeScale::Tt],
        body_coverage: vec![CelestialBody::Sun],
        supported_frames: vec![CoordinateFrame::Ecliptic],
        capabilities: BackendCapabilities::default(),
        accuracy: AccuracyClass::Approximate,
        deterministic: true,
        offline: true,
    };
    let geocentric_request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
    );
    let sidereal_topocentric_request = EphemerisRequest {
        zodiac_mode: ZodiacMode::Sidereal {
            ayanamsa: pleiades_types::Ayanamsa::FaganBradley,
        },
        observer: Some(ObserverLocation::new(
            Latitude::from_degrees(51.5),
            Longitude::from_degrees(-0.1),
            Some(45.0),
        )),
        ..geocentric_request.clone()
    };

    let error = validate_requests_against_metadata(
        &[geocentric_request, sidereal_topocentric_request],
        &metadata,
    )
    .expect_err("the batch helper should preserve sidereal failures before observer checks");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedZodiacMode);
    assert_eq!(
        error.message,
        "batch request 2: toy backend currently exposes tropical coordinates only"
    );
    assert!(!error
        .message
        .contains("topocentric positions are not implemented"));
}

#[test]
fn validate_requests_against_metadata_rejects_sidereal_requests_before_invalid_observer_checks_in_batches(
) {
    let metadata = BackendMetadata {
        id: BackendId::new("toy backend"),
        version: "0.1.0".to_string(),
        family: BackendFamily::Algorithmic,
        provenance: BackendProvenance::new("toy backend"),
        nominal_range: TimeRange::new(None, None),
        supported_time_scales: vec![TimeScale::Tt],
        body_coverage: vec![CelestialBody::Sun],
        supported_frames: vec![CoordinateFrame::Ecliptic],
        capabilities: BackendCapabilities::default(),
        accuracy: AccuracyClass::Approximate,
        deterministic: true,
        offline: true,
    };
    let geocentric_request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
    );
    let sidereal_invalid_observer_request = EphemerisRequest {
        zodiac_mode: ZodiacMode::Sidereal {
            ayanamsa: pleiades_types::Ayanamsa::FaganBradley,
        },
        observer: Some(ObserverLocation::new(
            Latitude::from_degrees(95.0),
            Longitude::from_degrees(-0.1),
            Some(45.0),
        )),
        ..geocentric_request.clone()
    };

    let error = validate_requests_against_metadata(
        &[geocentric_request, sidereal_invalid_observer_request],
        &metadata,
    )
    .expect_err(
        "the batch helper should preserve sidereal failures before invalid observer checks",
    );
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedZodiacMode);
    assert_eq!(
        error.message,
        "batch request 2: toy backend currently exposes tropical coordinates only"
    );
    assert!(!error.message.contains("invalid observer location"));
}

#[test]
fn validate_requests_against_metadata_fails_fast_on_the_first_invalid_request() {
    let metadata = BackendMetadata {
        id: BackendId::new("toy backend"),
        version: "0.1.0".to_string(),
        family: BackendFamily::Algorithmic,
        provenance: BackendProvenance::new("toy backend"),
        nominal_range: TimeRange::new(None, None),
        supported_time_scales: vec![TimeScale::Tt],
        body_coverage: vec![CelestialBody::Sun],
        supported_frames: vec![CoordinateFrame::Ecliptic],
        capabilities: BackendCapabilities::default(),
        accuracy: AccuracyClass::Approximate,
        deterministic: true,
        offline: true,
    };
    let valid_request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
    );
    let invalid_request = EphemerisRequest {
        body: CelestialBody::Mars,
        ..valid_request.clone()
    };

    let error = validate_requests_against_metadata(&[valid_request, invalid_request], &metadata)
        .expect_err("the batch helper should stop at the first unsupported body");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedBody);
    assert_eq!(
        error.message,
        "batch request 2: toy backend does not support Mars"
    );
}

#[test]
fn batch_query_preserves_apparent_request_rejection() {
    struct MeanOnlyBackend {
        calls: AtomicUsize,
    }

    impl EphemerisBackend for MeanOnlyBackend {
        fn metadata(&self) -> BackendMetadata {
            BackendMetadata {
                id: BackendId::new("mean-only"),
                version: "0.1.0".to_string(),
                family: BackendFamily::Algorithmic,
                provenance: BackendProvenance::new("mean-only test backend"),
                nominal_range: TimeRange::new(None, None),
                supported_time_scales: vec![TimeScale::Tt],
                body_coverage: vec![CelestialBody::Sun],
                supported_frames: vec![CoordinateFrame::Ecliptic],
                capabilities: BackendCapabilities::default(),
                accuracy: AccuracyClass::Approximate,
                deterministic: true,
                offline: true,
            }
        }

        fn supports_body(&self, body: CelestialBody) -> bool {
            body == CelestialBody::Sun
        }

        fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
            self.calls.fetch_add(1, Ordering::SeqCst);

            validate_request_policy(
                req,
                "mean-only test backend",
                &[TimeScale::Tt],
                &[CoordinateFrame::Ecliptic],
                true,
                false,
            )?;

            Ok(EphemerisResult::new(
                BackendId::new("mean-only"),
                req.body.clone(),
                req.instant,
                req.frame,
                req.zodiac_mode.clone(),
                req.apparent,
            ))
        }
    }

    let backend = MeanOnlyBackend {
        calls: AtomicUsize::new(0),
    };
    let mean_request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt),
    );
    let apparent_request = EphemerisRequest {
        apparent: Apparentness::Apparent,
        ..mean_request.clone()
    };

    let error = backend
        .positions(&[mean_request, apparent_request])
        .expect_err("batch requests should preserve apparentness rejections");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedApparentness);
    assert_eq!(backend.calls.load(Ordering::SeqCst), 2);
}

#[test]
fn batch_query_preserves_observer_request_rejection() {
    struct GeocentricOnlyBackend {
        calls: AtomicUsize,
    }

    impl EphemerisBackend for GeocentricOnlyBackend {
        fn metadata(&self) -> BackendMetadata {
            BackendMetadata {
                id: BackendId::new("geocentric-only"),
                version: "0.1.0".to_string(),
                family: BackendFamily::Algorithmic,
                provenance: BackendProvenance::new("geocentric-only test backend"),
                nominal_range: TimeRange::new(None, None),
                supported_time_scales: vec![TimeScale::Tt],
                body_coverage: vec![CelestialBody::Sun],
                supported_frames: vec![CoordinateFrame::Ecliptic],
                capabilities: BackendCapabilities::default(),
                accuracy: AccuracyClass::Approximate,
                deterministic: true,
                offline: true,
            }
        }

        fn supports_body(&self, body: CelestialBody) -> bool {
            body == CelestialBody::Sun
        }

        fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
            self.calls.fetch_add(1, Ordering::SeqCst);

            validate_observer_policy(req, "geocentric-only test backend", false)?;

            Ok(EphemerisResult::new(
                BackendId::new("geocentric-only"),
                req.body.clone(),
                req.instant,
                req.frame,
                req.zodiac_mode.clone(),
                req.apparent,
            ))
        }
    }

    let backend = GeocentricOnlyBackend {
        calls: AtomicUsize::new(0),
    };
    let geocentric_request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt),
    );
    let topocentric_request = EphemerisRequest {
        observer: Some(ObserverLocation::new(
            Latitude::from_degrees(51.5),
            Longitude::from_degrees(-0.1),
            Some(45.0),
        )),
        ..geocentric_request.clone()
    };

    let error = backend
        .positions(&[geocentric_request, topocentric_request])
        .expect_err("batch requests should preserve observer rejections");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedObserver);
    assert_eq!(backend.calls.load(Ordering::SeqCst), 2);
}

#[test]
fn batch_query_preserves_mixed_time_scales() {
    struct MixedScaleBackend {
        calls: AtomicUsize,
    }

    impl EphemerisBackend for MixedScaleBackend {
        fn metadata(&self) -> BackendMetadata {
            BackendMetadata {
                id: BackendId::new("mixed-scale"),
                version: "0.1.0".to_string(),
                family: BackendFamily::Algorithmic,
                provenance: BackendProvenance::new("mixed-scale test backend"),
                nominal_range: TimeRange::new(None, None),
                supported_time_scales: vec![TimeScale::Tt, TimeScale::Tdb],
                body_coverage: vec![CelestialBody::Sun],
                supported_frames: vec![CoordinateFrame::Ecliptic],
                capabilities: BackendCapabilities::default(),
                accuracy: AccuracyClass::Approximate,
                deterministic: true,
                offline: true,
            }
        }

        fn supports_body(&self, body: CelestialBody) -> bool {
            body == CelestialBody::Sun
        }

        fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
            self.calls.fetch_add(1, Ordering::SeqCst);

            validate_request_policy(
                req,
                "mixed-scale test backend",
                &[TimeScale::Tt, TimeScale::Tdb],
                &[CoordinateFrame::Ecliptic],
                true,
                false,
            )?;

            Ok(EphemerisResult::new(
                BackendId::new("mixed-scale"),
                req.body.clone(),
                req.instant,
                req.frame,
                req.zodiac_mode.clone(),
                req.apparent,
            ))
        }
    }

    let backend = MixedScaleBackend {
        calls: AtomicUsize::new(0),
    };
    let tt_request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt),
    );
    let tdb_request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tdb),
    );

    let results = backend
        .positions(&[tt_request, tdb_request])
        .expect("batch requests should preserve mixed time-scale labels");
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].instant.scale, TimeScale::Tt);
    assert_eq!(results[1].instant.scale, TimeScale::Tdb);
    assert_eq!(backend.calls.load(Ordering::SeqCst), 2);
}

#[cfg(feature = "serde")]
#[test]
fn serde_roundtrip_preserves_requests_and_results() {
    let request = EphemerisRequest {
        body: CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros")),
        instant: Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt),
        observer: Some(ObserverLocation::new(
            Latitude::from_degrees(51.5),
            Longitude::from_degrees(-0.1),
            Some(45.0),
        )),
        frame: CoordinateFrame::Equatorial,
        zodiac_mode: ZodiacMode::Sidereal {
            ayanamsa: Ayanamsa::Lahiri,
        },
        apparent: Apparentness::Mean,
    };
    let request_roundtrip: EphemerisRequest =
        serde_json::from_value(serde_json::to_value(&request).expect("request should serialize"))
            .expect("request should deserialize");
    assert_eq!(request_roundtrip, request);

    let mut result = EphemerisResult::new(
        BackendId::new("toy"),
        CelestialBody::Moon,
        Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt),
        CoordinateFrame::Ecliptic,
        ZodiacMode::Tropical,
        Apparentness::Apparent,
    );
    result.quality = QualityAnnotation::Interpolated;
    result.ecliptic = Some(EclipticCoordinates::new(
        Longitude::from_degrees(123.0),
        Latitude::from_degrees(2.5),
        Some(1.0),
    ));
    result.motion = Some(Motion::new(Some(0.12), Some(-0.01), None));

    let result_roundtrip: EphemerisResult =
        serde_json::from_value(serde_json::to_value(&result).expect("result should serialize"))
            .expect("result should deserialize");
    assert_eq!(result_roundtrip, result);
}

#[test]
fn ephemeris_result_validation_rejects_invalid_coordinate_and_motion_samples() {
    let mut result = EphemerisResult::new(
        BackendId::new("toy"),
        CelestialBody::Moon,
        Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
        CoordinateFrame::Ecliptic,
        ZodiacMode::Tropical,
        Apparentness::Mean,
    );
    result.ecliptic = Some(EclipticCoordinates::new(
        Longitude::from_degrees(12.5),
        Latitude::from_degrees(2.5),
        Some(1.0),
    ));
    result.equatorial = Some(EquatorialCoordinates::new(
        Angle::from_degrees(f64::NAN),
        Latitude::from_degrees(1.0),
        Some(1.0),
    ));
    result.motion = Some(Motion::new(Some(f64::INFINITY), None, None));

    let error = result
        .validate()
        .expect_err("invalid equatorial coordinates should fail validation");
    assert!(matches!(
        error,
        EphemerisResultValidationError::InvalidEquatorial(
            CoordinateValidationError::NonFiniteValue {
                coordinate: "equatorial",
                field: "right_ascension",
                value,
            }
        ) if value.is_nan()
    ));
    assert!(error
            .to_string()
            .contains("backend result equatorial is invalid: equatorial coordinate field `right_ascension` must be finite"));

    result.equatorial = None;
    let error = result
        .validated_summary_line()
        .expect_err("invalid motion should fail validation");
    assert!(matches!(
        error,
        EphemerisResultValidationError::InvalidMotion(MotionValidationError::NonFiniteSpeed {
            field: "longitude_deg_per_day",
            value,
        }) if value.is_infinite()
    ));
    assert!(error.to_string().contains(
        "backend result motion is invalid: motion field `longitude_deg_per_day` must be finite"
    ));
}

#[test]
fn batch_query_uses_single_query_behavior() {
    let backend = ToyBackend;
    let request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt),
    );

    let result = backend
        .positions(&[request])
        .expect("toy backend should succeed");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].quality, QualityAnnotation::Exact);
    assert!(result[0].ecliptic.is_some());
}

#[test]
fn batch_query_short_circuits_on_the_first_error() {
    struct CountingBackend {
        calls: AtomicUsize,
    }

    impl EphemerisBackend for CountingBackend {
        fn metadata(&self) -> BackendMetadata {
            BackendMetadata {
                id: BackendId::new("counting"),
                version: "0.1.0".to_string(),
                family: BackendFamily::Algorithmic,
                provenance: BackendProvenance::new("counting test backend"),
                nominal_range: TimeRange::new(None, None),
                supported_time_scales: vec![TimeScale::Tt],
                body_coverage: vec![CelestialBody::Sun, CelestialBody::Moon],
                supported_frames: vec![CoordinateFrame::Ecliptic],
                capabilities: BackendCapabilities::default(),
                accuracy: AccuracyClass::Approximate,
                deterministic: true,
                offline: true,
            }
        }

        fn supports_body(&self, _body: CelestialBody) -> bool {
            true
        }

        fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
            self.calls.fetch_add(1, Ordering::SeqCst);

            if req.body == CelestialBody::Moon {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::UnsupportedBody,
                    "Moon requests fail in the counting backend",
                ));
            }

            Ok(EphemerisResult::new(
                BackendId::new("counting"),
                req.body.clone(),
                req.instant,
                req.frame,
                req.zodiac_mode.clone(),
                req.apparent,
            ))
        }
    }

    let backend = CountingBackend {
        calls: AtomicUsize::new(0),
    };
    let requests = [
        EphemerisRequest::new(
            CelestialBody::Sun,
            Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt),
        ),
        EphemerisRequest::new(
            CelestialBody::Moon,
            Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt),
        ),
        EphemerisRequest::new(
            CelestialBody::Sun,
            Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt),
        ),
    ];

    let error = backend
        .positions(&requests)
        .expect_err("batch requests should fail fast on the first error");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedBody);
    assert_eq!(backend.calls.load(Ordering::SeqCst), 2);
}

#[test]
fn metadata_is_reported() {
    let backend = ToyBackend;
    let metadata = backend.metadata();
    assert_eq!(metadata.id.as_str(), "toy");
    assert!(metadata.body_coverage.contains(&CelestialBody::Sun));
}

#[test]
fn composite_backend_routes_by_body() {
    struct MoonBackend;

    impl EphemerisBackend for MoonBackend {
        fn metadata(&self) -> BackendMetadata {
            BackendMetadata {
                id: BackendId::new("moon"),
                version: "0.1.0".to_string(),
                family: BackendFamily::Algorithmic,
                provenance: BackendProvenance::new("moon backend"),
                nominal_range: TimeRange::new(None, None),
                supported_time_scales: vec![TimeScale::Tt],
                body_coverage: vec![CelestialBody::Moon],
                supported_frames: vec![CoordinateFrame::Ecliptic],
                capabilities: BackendCapabilities::default(),
                accuracy: AccuracyClass::Approximate,
                deterministic: true,
                offline: true,
            }
        }

        fn supports_body(&self, body: CelestialBody) -> bool {
            body == CelestialBody::Moon
        }

        fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
            Ok(EphemerisResult::new(
                BackendId::new("moon"),
                req.body.clone(),
                req.instant,
                req.frame,
                req.zodiac_mode.clone(),
                req.apparent,
            ))
        }
    }

    let composite = CompositeBackend::new(ToyBackend, MoonBackend);
    let sun_request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt),
    );
    let moon_request = EphemerisRequest::new(
        CelestialBody::Moon,
        Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt),
    );

    assert_eq!(
        composite
            .position(&sun_request)
            .unwrap()
            .backend_id
            .as_str(),
        "toy"
    );
    assert_eq!(
        composite
            .position(&moon_request)
            .unwrap()
            .backend_id
            .as_str(),
        "moon"
    );
}

#[test]
fn routing_backend_tries_later_providers_and_merges_metadata() {
    struct FailingSunBackend;

    impl EphemerisBackend for FailingSunBackend {
        fn metadata(&self) -> BackendMetadata {
            BackendMetadata {
                id: BackendId::new("fail-sun"),
                version: "0.1.0".to_string(),
                family: BackendFamily::Algorithmic,
                provenance: BackendProvenance::new("failing Sun backend"),
                nominal_range: TimeRange::new(
                    Some(Instant::new(
                        JulianDay::from_days(2_451_545.0),
                        TimeScale::Tt,
                    )),
                    Some(Instant::new(
                        JulianDay::from_days(2_451_546.0),
                        TimeScale::Tt,
                    )),
                ),
                supported_time_scales: vec![TimeScale::Tt],
                body_coverage: vec![CelestialBody::Sun],
                supported_frames: vec![CoordinateFrame::Ecliptic],
                capabilities: BackendCapabilities::default(),
                accuracy: AccuracyClass::Approximate,
                deterministic: true,
                offline: true,
            }
        }

        fn supports_body(&self, body: CelestialBody) -> bool {
            body == CelestialBody::Sun
        }

        fn position(&self, _req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
            Err(EphemerisError::new(
                EphemerisErrorKind::UnsupportedCoordinateFrame,
                "retry with the next provider",
            ))
        }
    }

    struct RecoverySunBackend;

    impl EphemerisBackend for RecoverySunBackend {
        fn metadata(&self) -> BackendMetadata {
            BackendMetadata {
                id: BackendId::new("recovery-sun"),
                version: "0.1.0".to_string(),
                family: BackendFamily::Algorithmic,
                provenance: BackendProvenance::new("recovery Sun backend"),
                nominal_range: TimeRange::new(
                    Some(Instant::new(
                        JulianDay::from_days(2_451_545.5),
                        TimeScale::Tdb,
                    )),
                    Some(Instant::new(
                        JulianDay::from_days(2_451_546.5),
                        TimeScale::Tdb,
                    )),
                ),
                supported_time_scales: vec![TimeScale::Tt],
                body_coverage: vec![CelestialBody::Sun],
                supported_frames: vec![CoordinateFrame::Ecliptic],
                capabilities: BackendCapabilities::default(),
                accuracy: AccuracyClass::Approximate,
                deterministic: true,
                offline: true,
            }
        }

        fn supports_body(&self, body: CelestialBody) -> bool {
            body == CelestialBody::Sun
        }

        fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
            let mut result = EphemerisResult::new(
                BackendId::new("recovery-sun"),
                req.body.clone(),
                req.instant,
                req.frame,
                req.zodiac_mode.clone(),
                req.apparent,
            );
            result.quality = QualityAnnotation::Exact;
            result.ecliptic = Some(EclipticCoordinates::new(
                Longitude::from_degrees(10.0),
                Latitude::from_degrees(0.0),
                Some(1.0),
            ));
            Ok(result)
        }
    }

    struct MoonBackend;

    impl EphemerisBackend for MoonBackend {
        fn metadata(&self) -> BackendMetadata {
            BackendMetadata {
                id: BackendId::new("moon"),
                version: "0.1.0".to_string(),
                family: BackendFamily::Algorithmic,
                provenance: BackendProvenance::new("moon backend"),
                nominal_range: TimeRange::new(None, None),
                supported_time_scales: vec![TimeScale::Tt],
                body_coverage: vec![CelestialBody::Moon],
                supported_frames: vec![CoordinateFrame::Ecliptic],
                capabilities: BackendCapabilities::default(),
                accuracy: AccuracyClass::Approximate,
                deterministic: true,
                offline: true,
            }
        }

        fn supports_body(&self, body: CelestialBody) -> bool {
            body == CelestialBody::Moon
        }

        fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
            Ok(EphemerisResult::new(
                BackendId::new("moon"),
                req.body.clone(),
                req.instant,
                req.frame,
                req.zodiac_mode.clone(),
                req.apparent,
            ))
        }
    }

    let routing = RoutingBackend::new(vec![
        Box::new(FailingSunBackend),
        Box::new(RecoverySunBackend),
        Box::new(MoonBackend),
    ]);
    let sun_request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt),
    );
    let moon_request = EphemerisRequest::new(
        CelestialBody::Moon,
        Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt),
    );

    let metadata = routing.metadata();
    assert!(metadata.body_coverage.contains(&CelestialBody::Sun));
    assert!(metadata.body_coverage.contains(&CelestialBody::Moon));
    assert!(metadata.provenance.summary.contains("3 provider(s)"));
    assert_eq!(metadata.nominal_range.validate(), Ok(()));
    assert_eq!(
        metadata
            .nominal_range
            .start
            .expect("routing range start should exist")
            .scale,
        TimeScale::Tt
    );
    assert_eq!(
        metadata
            .nominal_range
            .end
            .expect("routing range end should exist")
            .scale,
        TimeScale::Tt
    );
    assert_eq!(
        metadata.nominal_range.summary_line(),
        "JD 2451545.5 (TT) → JD 2451546.0 (TT)"
    );

    assert_eq!(
        routing.position(&sun_request).unwrap().backend_id.as_str(),
        "recovery-sun"
    );
    assert_eq!(
        routing.position(&moon_request).unwrap().backend_id.as_str(),
        "moon"
    );
}

#[test]
fn routing_backend_batch_positions_preserve_mixed_time_scales_and_topocentric_observers_after_fallback(
) {
    struct FailingSunBackend;

    impl EphemerisBackend for FailingSunBackend {
        fn metadata(&self) -> BackendMetadata {
            BackendMetadata {
                id: BackendId::new("fail-sun-batch"),
                version: "0.1.0".to_string(),
                family: BackendFamily::Algorithmic,
                provenance: BackendProvenance::new("failing Sun batch backend"),
                nominal_range: TimeRange::new(None, None),
                supported_time_scales: vec![TimeScale::Tt, TimeScale::Tdb],
                body_coverage: vec![CelestialBody::Sun],
                supported_frames: vec![CoordinateFrame::Ecliptic],
                capabilities: BackendCapabilities::default(),
                accuracy: AccuracyClass::Approximate,
                deterministic: true,
                offline: true,
            }
        }

        fn supports_body(&self, body: CelestialBody) -> bool {
            body == CelestialBody::Sun
        }

        fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
            if req.observer.is_some() {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::UnsupportedObserver,
                    "retry with the next provider",
                ));
            }

            Ok(EphemerisResult::new(
                BackendId::new("fail-sun-batch"),
                req.body.clone(),
                req.instant,
                req.frame,
                req.zodiac_mode.clone(),
                req.apparent,
            ))
        }
    }

    struct RecoverySunBackend {
        calls: AtomicUsize,
    }

    impl EphemerisBackend for RecoverySunBackend {
        fn metadata(&self) -> BackendMetadata {
            BackendMetadata {
                id: BackendId::new("recovery-sun-batch"),
                version: "0.1.0".to_string(),
                family: BackendFamily::Algorithmic,
                provenance: BackendProvenance::new("recovery Sun batch backend"),
                nominal_range: TimeRange::new(None, None),
                supported_time_scales: vec![TimeScale::Tt, TimeScale::Tdb],
                body_coverage: vec![CelestialBody::Sun],
                supported_frames: vec![CoordinateFrame::Ecliptic],
                capabilities: BackendCapabilities {
                    topocentric: true,
                    ..BackendCapabilities::default()
                },
                accuracy: AccuracyClass::Approximate,
                deterministic: true,
                offline: true,
            }
        }

        fn supports_body(&self, body: CelestialBody) -> bool {
            body == CelestialBody::Sun
        }

        fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok(EphemerisResult::new(
                BackendId::new("recovery-sun-batch"),
                req.body.clone(),
                req.instant,
                req.frame,
                req.zodiac_mode.clone(),
                req.apparent,
            ))
        }
    }

    let routing = RoutingBackend::new(vec![
        Box::new(FailingSunBackend),
        Box::new(RecoverySunBackend {
            calls: AtomicUsize::new(0),
        }),
    ]);
    let tt_request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt),
    );
    let mut tdb_request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tdb),
    );
    tdb_request.observer = Some(ObserverLocation::new(
        Latitude::from_degrees(51.5),
        Longitude::from_degrees(12.5),
        Some(0.0),
    ));

    let metadata = routing.metadata();
    validate_requests_against_metadata(&[tt_request.clone(), tdb_request.clone()], &metadata)
            .expect("routing metadata should keep mixed TT/TDB and topocentric requests aligned with the selected provider");

    let results = routing
            .positions(&[tt_request.clone(), tdb_request.clone()])
            .expect("routing should preserve mixed batch scales while falling back to the secondary provider");

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].backend_id.as_str(), "fail-sun-batch");
    assert_eq!(results[1].backend_id.as_str(), "recovery-sun-batch");
    assert_eq!(results[0].instant.scale, TimeScale::Tt);
    assert_eq!(results[1].instant.scale, TimeScale::Tdb);
}
