use crate::metadata::BackendMetadataValidationError;
use core::fmt;

pub(crate) fn validate_non_blank(
    field: &'static str,
    value: &str,
) -> Result<(), BackendMetadataValidationError> {
    if value.trim().is_empty() || value.trim() != value {
        Err(BackendMetadataValidationError::BlankField { field })
    } else {
        Ok(())
    }
}

pub(crate) fn validate_unique_entries<T: fmt::Display + PartialEq>(
    field: &'static str,
    values: &[T],
) -> Result<(), BackendMetadataValidationError> {
    for (index, value) in values.iter().enumerate() {
        if values[..index].iter().any(|prior| prior == value) {
            return Err(BackendMetadataValidationError::DuplicateEntry {
                field,
                value: value.to_string(),
            });
        }
    }

    Ok(())
}

pub(crate) fn validate_non_empty_unique<T: fmt::Display + PartialEq>(
    field: &'static str,
    values: &[T],
) -> Result<(), BackendMetadataValidationError> {
    if values.is_empty() {
        return Err(BackendMetadataValidationError::EmptyField { field });
    }

    validate_unique_entries(field, values)
}
