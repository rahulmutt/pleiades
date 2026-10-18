//! VSOP87B Uranus coefficient tables backed by the full public IMCCE/CELMECH
//! source file.
//!
//! Uranus now uses a generated binary table derived from the vendored public
//! IMCCE/CELMECH VSOP87B source file, mirroring the Earth/Sun, Mercury, Venus,
//! Mars, Jupiter, and Saturn paths so the backend keeps a source-backed path
//! for every supported VSOP87 major planet while Pluto remains outside the
//! VSOP87 major-planet files.

use super::series::{SeriesFidelity, Vsop87SeriesSource};
use super::vsop87b_earth::SphericalLbr;

static URANUS: Vsop87SeriesSource =
    Vsop87SeriesSource::new(include_bytes!("../../data/VSOP87B.ura.bin"));

pub(crate) fn uranus_lbr(julian_day_tt: f64, fidelity: SeriesFidelity) -> SphericalLbr {
    URANUS.lbr(julian_day_tt, fidelity)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tables::vsop87b_earth::parse_generated_vsop87b_tables;

    #[test]
    fn parses_generated_uranus_table_blob_with_expected_series_counts() {
        let tables = parse_generated_vsop87b_tables(include_bytes!("../../data/VSOP87B.ura.bin"));
        assert_eq!(tables.longitude.len(), 6);
        assert_eq!(tables.latitude.len(), 6);
        assert_eq!(tables.radius.len(), 6);

        let longitude_terms: Vec<usize> = tables.longitude.iter().map(Vec::len).collect();
        let latitude_terms: Vec<usize> = tables.latitude.iter().map(Vec::len).collect();
        let radius_terms: Vec<usize> = tables.radius.iter().map(Vec::len).collect();

        assert_eq!(longitude_terms, vec![1441, 655, 259, 69, 8, 0]);
        assert_eq!(latitude_terms, vec![311, 130, 39, 15, 0, 0]);
        assert_eq!(radius_terms, vec![1387, 625, 249, 69, 12, 0]);
    }

    #[test]
    fn evaluates_j2000_uranus_coordinates_from_the_full_source_file() {
        let uranus = uranus_lbr(2_451_545.0, SeriesFidelity::Full);
        assert!((uranus.longitude_rad.to_degrees() - 316.418_722_907_632_7).abs() < 1e-12);
        assert!((uranus.latitude_rad.to_degrees() + 0.684_844_296_148_084_3).abs() < 1e-12);
        assert!((uranus.radius_au - 19.924_047_895_208_606).abs() < 1e-12);
    }
}
