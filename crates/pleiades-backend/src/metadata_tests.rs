use crate::*;

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
