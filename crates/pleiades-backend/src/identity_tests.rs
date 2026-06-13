use crate::*;

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
