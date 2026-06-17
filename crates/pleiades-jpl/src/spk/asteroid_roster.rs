//! Curated core of astrologically-relevant minor planets, tagged by sourcing
//! tier and dynamical class. The classical four are named `CelestialBody`
//! variants; all others use the `asteroid:`/`tno:` `Custom` catalog so the
//! shared body enum does not balloon. The unbounded long tail is reachable on
//! demand via `Custom` ids + `pleiades_jpl::ingest`; only this core is
//! committed as corpus data.

use pleiades_backend::CelestialBody;
use pleiades_types::CustomBodyId;

/// How a body's reference positions are sourced.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AsteroidTier {
    /// In `sb441-n16.bsp`: reproducible from the pinned kernel.
    PinnedKernel,
    /// Not in any fixed kernel: Horizons-sourced, provenance-validated only.
    Constrained,
}

/// Dynamical class, used to keep evidence separated in reports.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AsteroidClass {
    MainBelt,
    Centaur,
    Tno,
}

impl AsteroidClass {
    /// Coarse, speed-appropriate sampling cadence (TDB days) over the asteroid
    /// window. Asteroids and centaurs move slowly; TNOs barely move, so they
    /// are sampled sparsely to keep the committed corpus bounded.
    pub fn max_gap_days(self) -> f64 {
        match self {
            AsteroidClass::MainBelt => 180.0,
            AsteroidClass::Centaur => 365.0,
            AsteroidClass::Tno => 1_825.0, // ~5 yr
        }
    }
}

/// One curated-core minor planet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AsteroidEntry {
    pub body: CelestialBody,
    pub tier: AsteroidTier,
    pub class: AsteroidClass,
}

fn ast(designation: &str) -> CelestialBody {
    CelestialBody::Custom(CustomBodyId::new("asteroid", designation))
}

fn tno(designation: &str) -> CelestialBody {
    CelestialBody::Custom(CustomBodyId::new("tno", designation))
}

/// The committed curated core. Order is stable (checksums/reports depend on it).
pub fn asteroid_core_roster() -> &'static [AsteroidEntry] {
    use AsteroidClass::*;
    use AsteroidTier::*;
    use std::sync::OnceLock;
    static ROSTER: OnceLock<Vec<AsteroidEntry>> = OnceLock::new();
    ROSTER
        .get_or_init(|| {
            let e = |body, tier, class| AsteroidEntry { body, tier, class };
            vec![
                // Classical four — in sb441-n16, Tier A.
                e(CelestialBody::Ceres, PinnedKernel, MainBelt),
                e(CelestialBody::Pallas, PinnedKernel, MainBelt),
                e(CelestialBody::Juno, PinnedKernel, MainBelt),
                e(CelestialBody::Vesta, PinnedKernel, MainBelt),
                // Other massive main-belt members of sb441-n16 used in astrology.
                e(ast("10-Hygiea"), PinnedKernel, MainBelt),
                e(ast("16-Psyche"), PinnedKernel, MainBelt),
                e(ast("7-Iris"), PinnedKernel, MainBelt),
                // Centaurs — Tier B (not in sb441-n16).
                e(ast("2060-Chiron"), Constrained, Centaur),
                e(ast("5145-Pholus"), Constrained, Centaur),
                e(ast("7066-Nessus"), Constrained, Centaur),
                e(ast("10199-Chariklo"), Constrained, Centaur),
                e(ast("8405-Asbolus"), Constrained, Centaur),
                // Personal / "goddess" asteroids — Tier B.
                e(ast("433-Eros"), Constrained, MainBelt),
                e(ast("80-Sappho"), Constrained, MainBelt),
                e(ast("1221-Amor"), Constrained, MainBelt),
                e(ast("1181-Lilith"), Constrained, MainBelt),
                e(ast("5-Astraea"), Constrained, MainBelt),
                e(ast("6-Hebe"), Constrained, MainBelt),
                e(ast("8-Flora"), Constrained, MainBelt),
                e(ast("9-Metis"), Constrained, MainBelt),
                e(ast("19-Fortuna"), Constrained, MainBelt),
                e(ast("944-Hidalgo"), Constrained, MainBelt),
                e(ast("1566-Icarus"), Constrained, MainBelt),
                e(ast("1685-Toro"), Constrained, MainBelt),
                e(ast("1862-Apollo"), Constrained, MainBelt),
                // TNOs / dwarf planets — Tier B.
                e(tno("136199-Eris"), Constrained, Tno),
                e(tno("90377-Sedna"), Constrained, Tno),
                e(tno("136108-Haumea"), Constrained, Tno),
                e(tno("136472-Makemake"), Constrained, Tno),
                e(tno("50000-Quaoar"), Constrained, Tno),
                e(tno("90482-Orcus"), Constrained, Tno),
                e(tno("28978-Ixion"), Constrained, Tno),
                e(tno("20000-Varuna"), Constrained, Tno),
                e(tno("225088-Gonggong"), Constrained, Tno),
            ]
        })
        .as_slice()
}

/// Bodies sourced from the pinned kernel (Tier A), in roster order.
pub fn tier_a_bodies() -> Vec<CelestialBody> {
    asteroid_core_roster()
        .iter()
        .filter(|e| e.tier == AsteroidTier::PinnedKernel)
        .map(|e| e.body.clone())
        .collect()
}

/// Horizons-sourced constrained bodies (Tier B), in roster order.
pub fn tier_b_bodies() -> Vec<CelestialBody> {
    asteroid_core_roster()
        .iter()
        .filter(|e| e.tier == AsteroidTier::Constrained)
        .map(|e| e.body.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spk::chain::naif_ids;

    #[test]
    fn roster_has_curated_core() {
        let roster = asteroid_core_roster();
        // ~35-body curated core: classical 4 + centaurs + personal + TNOs.
        assert!(roster.len() >= 33 && roster.len() <= 38, "got {}", roster.len());
    }

    #[test]
    fn tiers_are_disjoint_and_cover_roster() {
        let a = tier_a_bodies();
        let b = tier_b_bodies();
        assert_eq!(a.len() + b.len(), asteroid_core_roster().len());
        for body in &a {
            assert!(!b.contains(body), "{body:?} in both tiers");
        }
    }

    #[test]
    fn classical_four_are_tier_a_main_belt() {
        for body in [
            CelestialBody::Ceres,
            CelestialBody::Pallas,
            CelestialBody::Juno,
            CelestialBody::Vesta,
        ] {
            let e = asteroid_core_roster()
                .iter()
                .find(|e| e.body == body)
                .expect("classical asteroid present");
            assert_eq!(e.tier, AsteroidTier::PinnedKernel);
            assert_eq!(e.class, AsteroidClass::MainBelt);
        }
    }

    #[test]
    fn chiron_is_constrained_centaur() {
        let e = asteroid_core_roster()
            .iter()
            .find(|e| matches!(&e.body, CelestialBody::Custom(c) if c.designation == "2060-Chiron"))
            .expect("Chiron present");
        assert_eq!(e.tier, AsteroidTier::Constrained);
        assert_eq!(e.class, AsteroidClass::Centaur);
    }

    #[test]
    fn every_roster_body_resolves_to_a_naif_id() {
        for e in asteroid_core_roster() {
            assert!(
                !naif_ids(&e.body).is_empty(),
                "{:?} has no NAIF id candidates",
                e.body
            );
        }
    }
}
