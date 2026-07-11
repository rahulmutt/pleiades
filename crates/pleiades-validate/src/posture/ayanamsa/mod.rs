//! Ayanamsa report/summary prose relocated from `pleiades-ayanamsa`
//! (report-surface relocation program, Slice B). Rendering only — the
//! functional crate keeps the structured data and its inherent methods.

// Verbatim relocation of a report-prose surface: some renderers are exercised
// only by this module's own tests or have no current in-crate caller.
#![allow(dead_code)]

use pleiades_ayanamsa::{provenance_summary, AyanamsaProvenanceSummaryValidationError};

/// Returns the release-facing provenance payload after validation.
pub(crate) fn validated_provenance_summary_for_report(
) -> Result<String, AyanamsaProvenanceSummaryValidationError> {
    provenance_summary().validated_summary_line()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validated_provenance_summary_for_report_renders_expected_examples() {
        let rendered = validated_provenance_summary_for_report()
            .expect("ayanamsa provenance summary should validate");

        assert_eq!(
            rendered,
            "representative provenance examples: True Citra — True Citra sidereal mode with the published zero point used by Swiss Ephemeris-style interoperability tables.; True Revati — True-nakshatra mode with the Revati reference point fixed to the Swiss Ephemeris zero date.; True Mula — True-nakshatra mode with the Mula reference point fixed to the Swiss Ephemeris zero date.; True Pushya — True-nakshatra Pushya reference mode exposed by Swiss Ephemeris and anchored to the published zero date.; Udayagiri — Udayagiri sidereal mode treated as the Lahiri/Chitrapaksha/Chitra Paksha 285 CE reference family in the Swiss Ephemeris interoperability catalog.; True Sheoran — True-nakshatra Sheoran reference mode with the Swiss Ephemeris zero point at JD 1789947.090881 (+0188/08/09 14:10:52.11 UT).; Babylonian (Britton) — Babylonian sidereal mode associated with Britton's reconstruction, with the Swiss Ephemeris zero point at JD 1805415.712776 (+0230/12/17 05:06:23.86 UT).; Galactic Center (Rgilbrand) — Galactic-center reference mode attributed to Rgilbrand, with the Swiss Ephemeris zero point at JD 1861740.329525 (+0385/03/03 19:54:30.99 UT).; Babylonian (Kugler 1) — Babylonian sidereal mode associated with Kugler's first reconstruction, with the Swiss Ephemeris zero point at JD 1833923.577692 (+0309/01/05 01:51:52.62 UT).; Galactic Equator — Galactic-equator sidereal reference mode. The true/modern variant is anchored to the 1665728.603158 JD zero point described in the Swiss Ephemeris documentation.; Suryasiddhanta (Mean Sun) — Suryasiddhanta mean-sun variant anchored to the published 514 CE zero point used by Swiss Ephemeris.; Aryabhata (522 CE) — Aryabhata zero-point variant anchored to the published 522 CE reference date.; Valens Moon — Valens Moon sidereal mode, catalogued with the Swiss Ephemeris reference epoch and offset from the header metadata."
        );
        assert_eq!(
            rendered,
            provenance_summary()
                .validated_summary_line()
                .expect("ayanamsa provenance summary should validate")
        );
    }
}
