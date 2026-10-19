//! VSOP87B Venus coefficient tables backed by the full public IMCCE/CELMECH
//! source file.
//!
//! Venus now uses a generated binary table derived from the vendored public
//! IMCCE/CELMECH VSOP87B source file, so the backend keeps a reproducible
//! coefficient artifact for the inner planet without relying on a hand-trimmed
//! coefficient slice.

use super::series::{SeriesFidelity, Vsop87SeriesSource};
use super::vsop87b_earth::SphericalLbr;

static VENUS: Vsop87SeriesSource =
    Vsop87SeriesSource::new(include_bytes!("../../data/VSOP87B.ven.bin"));

pub(crate) fn venus_lbr(julian_day_tt: f64, fidelity: SeriesFidelity) -> SphericalLbr {
    VENUS.lbr(julian_day_tt, fidelity)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tables::vsop87b_earth::parse_generated_vsop87b_tables;
    use crate::tables::vsop87b_earth::parse_vsop87b_tables;

    #[test]
    fn parses_full_venus_tables_with_expected_series_counts() {
        let tables = parse_vsop87b_tables(include_str!("../../data/VSOP87B.ven"))
            .expect("Venus source file should parse");
        assert_eq!(tables.longitude.len(), 6);
        assert_eq!(tables.latitude.len(), 6);
        assert_eq!(tables.radius.len(), 6);

        let longitude_terms: Vec<usize> = tables.longitude.iter().map(Vec::len).collect();
        let latitude_terms: Vec<usize> = tables.latitude.iter().map(Vec::len).collect();
        let radius_terms: Vec<usize> = tables.radius.iter().map(Vec::len).collect();

        assert_eq!(longitude_terms, vec![416, 235, 72, 7, 4, 2]);
        assert_eq!(latitude_terms, vec![210, 121, 51, 12, 4, 4]);
        assert_eq!(radius_terms, vec![323, 174, 62, 8, 3, 2]);
    }

    #[test]
    fn parses_generated_venus_table_blob_with_expected_series_counts() {
        let tables = parse_generated_vsop87b_tables(include_bytes!("../../data/VSOP87B.ven.bin"));
        assert_eq!(tables.longitude.len(), 6);
        assert_eq!(tables.latitude.len(), 6);
        assert_eq!(tables.radius.len(), 6);

        let longitude_terms: Vec<usize> = tables.longitude.iter().map(Vec::len).collect();
        let latitude_terms: Vec<usize> = tables.latitude.iter().map(Vec::len).collect();
        let radius_terms: Vec<usize> = tables.radius.iter().map(Vec::len).collect();

        assert_eq!(longitude_terms, vec![416, 235, 72, 7, 4, 2]);
        assert_eq!(latitude_terms, vec![210, 121, 51, 12, 4, 4]);
        assert_eq!(radius_terms, vec![323, 174, 62, 8, 3, 2]);
    }

    #[test]
    fn evaluates_j2000_venus_coordinates_from_the_generated_table_file() {
        let venus = venus_lbr(2_451_545.0, SeriesFidelity::Full);
        assert!((venus.longitude_rad.to_degrees() - 182.602_920_758_433_04).abs() < 1e-12);
        assert!((venus.latitude_rad.to_degrees() - 3.264_615_250_767_216).abs() < 1e-12);
        assert!((venus.radius_au - 0.720_212_924_846_730_5).abs() < 1e-15);
    }
}
