use pleiades_backend::{EphemerisError, EphemerisErrorKind};
use pleiades_houses::HouseError;
use pleiades_types::{CustomDefinitionValidationError, ObserverLocationValidationError};

pub(super) fn map_observer_location_error(
    error: ObserverLocationValidationError,
) -> EphemerisError {
    EphemerisError::new(EphemerisErrorKind::InvalidObserver, error.to_string())
}

pub(super) fn map_house_error(error: HouseError) -> EphemerisError {
    let kind = match error.kind {
        pleiades_houses::HouseErrorKind::UnsupportedHouseSystem => {
            EphemerisErrorKind::InvalidRequest
        }
        pleiades_houses::HouseErrorKind::InvalidLatitude
        | pleiades_houses::HouseErrorKind::InvalidLongitude
        | pleiades_houses::HouseErrorKind::InvalidElevation => EphemerisErrorKind::InvalidObserver,
        pleiades_houses::HouseErrorKind::InvalidObliquity => EphemerisErrorKind::InvalidRequest,
        pleiades_houses::HouseErrorKind::NumericalFailure => EphemerisErrorKind::NumericalFailure,
        _ => EphemerisErrorKind::InvalidRequest,
    };

    EphemerisError::new(kind, error.message)
}

pub(super) fn map_custom_definition_error(
    subject: &'static str,
    error: CustomDefinitionValidationError,
) -> EphemerisError {
    EphemerisError::new(
        EphemerisErrorKind::InvalidRequest,
        format!("{subject} is invalid: {error}"),
    )
}
