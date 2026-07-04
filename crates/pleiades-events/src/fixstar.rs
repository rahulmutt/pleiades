//! Curated fixed-star apparent place. A bounded set of astrologically-used
//! stars, astrometry matched to Swiss Ephemeris `sefstars.txt`, parsed once from
//! a committed CSV at build time.

use crate::error::EventError;

/// One catalog row: J2000 ICRS position + space motion.
#[derive(Clone, Copy, Debug)]
pub struct FixedStarEntry {
    /// SE-compatible star name.
    pub name: &'static str,
    /// Right ascension at J2000, degrees.
    pub ra_j2000_deg: f64,
    /// Declination at J2000, degrees.
    pub dec_j2000_deg: f64,
    /// Proper motion in RA (mas/yr, already `·cosδ`-folded per sefstars convention).
    pub pm_ra_mas_yr: f64,
    /// Proper motion in Dec (mas/yr).
    pub pm_dec_mas_yr: f64,
    /// Parallax, milliarcseconds.
    pub parallax_mas: f64,
    /// Radial velocity, km/s.
    pub rv_km_s: f64,
}

const RAW: &str = include_str!("../data/fixstars-catalog.csv");

// Parsed at first use; the file is tiny (~30 rows). Build a static via a parse fn
// invoked in a `LazyLock` (std, no extra deps).
use std::sync::LazyLock;
pub(crate) static CATALOG: LazyLock<Vec<FixedStarEntry>> = LazyLock::new(parse_catalog);

fn parse_catalog() -> Vec<FixedStarEntry> {
    RAW.lines()
        .filter(|l| !l.starts_with('#') && !l.starts_with("name") && !l.trim().is_empty())
        .map(|l| {
            let f: Vec<&str> = l.split(',').collect();
            FixedStarEntry {
                name: Box::leak(f[0].to_string().into_boxed_str()),
                ra_j2000_deg: f[1].parse().unwrap(),
                dec_j2000_deg: f[2].parse().unwrap(),
                pm_ra_mas_yr: f[3].parse().unwrap(),
                pm_dec_mas_yr: f[4].parse().unwrap(),
                parallax_mas: f[5].parse().unwrap(),
                rv_km_s: f[6].parse().unwrap(),
            }
        })
        .collect()
}

/// Looks up a curated fixed star by SE-compatible name (case-insensitive).
///
/// Returns [`EventError::UnknownFixedStar`] (fail closed, no placeholder) when the
/// name is not present in the curated catalog.
pub fn fixed_star_entry(name: &str) -> Result<&'static FixedStarEntry, EventError> {
    CATALOG
        .iter()
        .find(|e| e.name.eq_ignore_ascii_case(name))
        .ok_or_else(|| EventError::UnknownFixedStar {
            name: name.to_string(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_star_resolves() {
        let e = fixed_star_entry("Aldebaran").unwrap();
        assert!((e.ra_j2000_deg - 68.98).abs() < 0.1);
        assert!((e.dec_j2000_deg - 16.51).abs() < 0.1);
    }

    #[test]
    fn unknown_star_fails_closed() {
        let err = fixed_star_entry("Nonesuch").unwrap_err();
        assert!(matches!(err, EventError::UnknownFixedStar { .. }));
    }

    #[test]
    fn catalog_is_nonempty_and_finite() {
        assert!(CATALOG.len() >= 25);
        for e in CATALOG.iter() {
            assert!(e.ra_j2000_deg.is_finite() && (0.0..360.0).contains(&e.ra_j2000_deg));
            assert!((-90.0..=90.0).contains(&e.dec_j2000_deg));
        }
    }
}
