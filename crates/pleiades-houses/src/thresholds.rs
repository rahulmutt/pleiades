//! Per-formula-family numeric ceilings for the house-system gate.
//!
//! Mirrors `pleiades-data/src/thresholds.rs`: a small struct of ceilings plus a
//! lookup keyed by the abstract family, and a release-facing summary line.
#![forbid(unsafe_code)]

use crate::catalog::HouseFormulaFamily;

/// Arcsecond ceilings for one formula family.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HouseFamilyCeiling {
    /// Max allowed |residual| on any house cusp, arcseconds.
    pub cusp_arcsec: f64,
    /// Max allowed |residual| on Ascendant/Midheaven, arcseconds.
    pub angle_arcsec: f64,
}

/// Returns the measured-derived arcsecond ceilings for a formula family.
///
/// Ceilings are set to `ceil(measured_max × 2)` — a 2× safety margin over the
/// maximum observed SE-vs-pleiades residual across the committed corpus
/// (`crates/pleiades-validate/data/houses-corpus/cusps.csv`).  A floor of 1.0″
/// is applied so families whose measured residual is exactly 0 still carry a
/// finite positive ceiling.
///
/// Measured maxima (corpus, 60 rows, 4 corpus-validated families):
///
/// | Family               | max cusp  | max angle |
/// |----------------------|-----------|-----------|
/// | Equal                | 0.4921″   | 0.4921″   |
/// | WholeSign            | 0.0000″   | 0.4921″   |
/// | Quadrant             | 5.7145″   | 0.4921″   |
/// | EquatorialProjection | 0.4921″   | 0.4921″   |
///
/// Families not exercised by the committed corpus (GreatCircle, SolarArc,
/// Sector, Custom, Unknown) have no SE-vs-pleiades baseline; their ceilings are
/// kept at generous conservative values and are NOT corpus-validated.  They will
/// be tightened once corpus rows are added for those families.
pub fn house_family_ceiling(family: HouseFormulaFamily) -> HouseFamilyCeiling {
    match family {
        // Equal: measured max cusp 0.4921″ → ceil(0.9842) = 1 → 1.0″ (floor).
        // WholeSign: measured max cusp 0.0000″ → floor → 1.0″.
        // Angle: measured max 0.4921″ → ceil(0.9842) = 1 → 1.0″ (floor).
        HouseFormulaFamily::Equal
        | HouseFormulaFamily::WholeSign => HouseFamilyCeiling { cusp_arcsec: 1.0, angle_arcsec: 1.0 },

        // Quadrant (Placidus/Koch/Porphyry/Alcabitius/Topocentric):
        // measured max cusp 5.7145″ (Koch at lat 66°) → ceil(11.429) = 12.0″.
        // Angle measured max 0.4921″ → ceil(0.9842) = 1 → 2.0″ (small extra margin for angles).
        HouseFormulaFamily::Quadrant => HouseFamilyCeiling { cusp_arcsec: 12.0, angle_arcsec: 2.0 },

        // EquatorialProjection (Regiomontanus/Campanus/Meridian/Axial/Morinus):
        // measured max cusp 0.4921″ → ceil(0.9842) = 1 → 1.0″ (floor).
        // Angle measured max 0.4921″ → 1.0″ (floor).
        HouseFormulaFamily::EquatorialProjection => HouseFamilyCeiling { cusp_arcsec: 1.0, angle_arcsec: 1.0 },

        // NOT corpus-validated — generous conservative values retained until
        // SE baseline rows are added for these families.
        HouseFormulaFamily::GreatCircle => HouseFamilyCeiling { cusp_arcsec: 15.0, angle_arcsec: 5.0 },
        HouseFormulaFamily::SolarArc
        | HouseFormulaFamily::Sector
        | HouseFormulaFamily::Custom
        | HouseFormulaFamily::Unknown => HouseFamilyCeiling { cusp_arcsec: 60.0, angle_arcsec: 10.0 },
    }
}

/// Compact release-facing summary of the family ceilings.
pub fn house_thresholds_summary_for_report() -> String {
    let equal = house_family_ceiling(HouseFormulaFamily::Equal);
    let quad = house_family_ceiling(HouseFormulaFamily::Quadrant);
    format!(
        "House ceilings: space-division {:.1}\u{2033} cusp, quadrant {:.1}\u{2033} cusp",
        equal.cusp_arcsec, quad.cusp_arcsec
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn space_division_is_tighter_than_quadrant() {
        let equal = house_family_ceiling(HouseFormulaFamily::Equal);
        let quad = house_family_ceiling(HouseFormulaFamily::Quadrant);
        assert!(equal.cusp_arcsec <= quad.cusp_arcsec);
        assert!(equal.cusp_arcsec > 0.0);
    }

    #[test]
    fn every_family_has_finite_positive_ceilings() {
        for family in [
            HouseFormulaFamily::Equal,
            HouseFormulaFamily::WholeSign,
            HouseFormulaFamily::Quadrant,
            HouseFormulaFamily::EquatorialProjection,
            HouseFormulaFamily::GreatCircle,
            HouseFormulaFamily::SolarArc,
            HouseFormulaFamily::Sector,
            HouseFormulaFamily::Custom,
            HouseFormulaFamily::Unknown,
        ] {
            let c = house_family_ceiling(family);
            assert!(c.cusp_arcsec.is_finite() && c.cusp_arcsec > 0.0);
            assert!(c.angle_arcsec.is_finite() && c.angle_arcsec > 0.0);
        }
    }
}
