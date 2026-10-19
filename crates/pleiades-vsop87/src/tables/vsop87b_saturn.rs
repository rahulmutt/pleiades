//! VSOP87B Saturn coefficient tables backed by the full public IMCCE/CELMECH
//! source file.
//!
//! Saturn now uses a generated binary table derived from the vendored public
//! IMCCE/CELMECH VSOP87B source file, mirroring the Earth/Sun, Mercury, Venus,
//! Mars, and Jupiter paths so the backend keeps a source-backed path for every
//! supported VSOP87 major planet while Pluto remains outside the VSOP87
//! major-planet files.

use super::series::{SeriesFidelity, Vsop87SeriesSource};
use super::vsop87b_earth::SphericalLbr;

static SATURN: Vsop87SeriesSource =
    Vsop87SeriesSource::new(include_bytes!("../../data/VSOP87B.sat.bin"));

pub(crate) fn saturn_lbr(julian_day_tt: f64, fidelity: SeriesFidelity) -> SphericalLbr {
    SATURN.lbr(julian_day_tt, fidelity)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tables::vsop87b_earth::parse_generated_vsop87b_tables;

    #[test]
    fn parses_generated_saturn_table_blob_with_expected_series_counts() {
        let tables = parse_generated_vsop87b_tables(include_bytes!("../../data/VSOP87B.sat.bin"));
        assert_eq!(tables.longitude.len(), 6);
        assert_eq!(tables.latitude.len(), 6);
        assert_eq!(tables.radius.len(), 6);

        let longitude_terms: Vec<usize> = tables.longitude.iter().map(Vec::len).collect();
        let latitude_terms: Vec<usize> = tables.latitude.iter().map(Vec::len).collect();
        let radius_terms: Vec<usize> = tables.radius.iter().map(Vec::len).collect();

        assert_eq!(longitude_terms, vec![1437, 817, 438, 192, 85, 30]);
        assert_eq!(latitude_terms, vec![500, 247, 111, 54, 24, 11]);
        assert_eq!(radius_terms, vec![1208, 627, 338, 154, 65, 27]);
    }

    #[test]
    fn evaluates_j2000_saturn_coordinates_from_the_full_source_file() {
        let saturn = saturn_lbr(2_451_545.0, SeriesFidelity::Full);
        assert!((saturn.longitude_rad.to_degrees() - 45.722_254_745_568_414).abs() < 1e-12);
        assert!((saturn.latitude_rad.to_degrees() + 2.303_199_518_162_094).abs() < 1e-12);
        assert!((saturn.radius_au - 9.183_848_288_052_635).abs() < 1e-12);
    }
}
