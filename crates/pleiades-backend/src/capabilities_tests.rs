use crate::*;

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
