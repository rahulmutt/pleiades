use super::{FrameTreatmentSummary, FrameTreatmentSummaryValidationError as Error};

const LINE: &str = "geocentric ecliptic of date; no frame bias applied";

#[test]
fn a_compact_line_renders_unchanged() {
    let summary = FrameTreatmentSummary::new(LINE);
    assert_eq!(summary.summary_line(), LINE);
    assert_eq!(summary.to_string(), LINE);
    assert_eq!(summary.validate(), Ok(()));
    assert_eq!(summary.validated_summary_line(), Ok(LINE));
}

#[test]
fn a_line_that_is_not_compact_is_rejected() {
    for (summary, expected) in [
        ("", Error::BlankSummary),
        ("   ", Error::BlankSummary),
        (" padded", Error::WhitespacePaddedSummary),
        ("padded ", Error::WhitespacePaddedSummary),
        ("two\nlines", Error::EmbeddedLineBreak),
        // Checked apart from `\n`: either line break alone is rejected.
        ("two\rlines", Error::EmbeddedLineBreak),
    ] {
        let summary = FrameTreatmentSummary::new(summary);
        assert_eq!(summary.validate(), Err(expected), "{summary:?}");
        assert_eq!(
            summary.validated_summary_line(),
            Err(expected),
            "{summary:?}"
        );
    }
}

#[test]
fn validation_errors_describe_the_defect() {
    for (error, message) in [
        (Error::BlankSummary, "frame-treatment summary is blank"),
        (
            Error::WhitespacePaddedSummary,
            "frame-treatment summary has surrounding whitespace",
        ),
        (
            Error::EmbeddedLineBreak,
            "frame-treatment summary contains a line break",
        ),
    ] {
        assert_eq!(error.to_string(), message);
    }
}
