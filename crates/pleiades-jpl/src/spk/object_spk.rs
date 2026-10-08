//! Per-object pinned SPK manifest for Tier-A asteroids absent from the bundled
//! `sb441-n373s` perturber kernel (centaurs, personal/minor/NEA bodies). Each
//! `.bsp` is sourced once from JPL Horizons over 1900–2100 and pinned by SHA;
//! the files are uncommitted (like de440/sb441-n373s) — this manifest is the
//! committed provenance. The regen path loads them from `PLEIADES_OBJECT_SPK_DIR`.
//!
//! Note: JPL's SPK uses the 8-digit NAIF scheme internally (20_000_000 + n);
//! the resolver (chain.rs) tries both schemes, so the 7-digit id used here is
//! the on-disk filename key.

/// One pinned per-object SPK.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObjectSpk {
    /// Roster `Custom` designation, e.g. `"asteroid:2060-Chiron"`.
    pub body_designation: &'static str,
    /// NAIF id (`2_000_000 + minor-planet number`).
    pub naif_id: i32,
    /// Lowercase 64-hex SHA-256 of the pinned `.bsp`.
    pub sha256: &'static str,
    /// Claim evidence source label (`"jpl-sbdb-spk:<number>"`).
    pub source_label: &'static str,
    /// Exact Horizons/SBDB request used to generate the SPK (provenance).
    pub request: &'static str,
}

/// The committed per-object SPK manifest, in roster order.
pub fn object_spk_manifest() -> &'static [ObjectSpk] {
    &[
        ObjectSpk {
            body_designation: "asteroid:2060-Chiron",
            naif_id: 2_002_060,
            sha256: "92eb11beda1a809a0bf89ef3591aaa5ea63da0330f8b72675e9d341ad9e82e82",
            source_label: "jpl-sbdb-spk:2060",
            request:
                "Horizons EPHEM_TYPE=SPK COMMAND='DES=2002060;' START=1899-12-01 STOP=2100-02-01",
        },
        ObjectSpk {
            body_designation: "asteroid:5145-Pholus",
            naif_id: 2_005_145,
            sha256: "da3d5addaa0aac7d5b21564bbda20dd71d1904aeb87eedbd98454e9f10a921ae",
            source_label: "jpl-sbdb-spk:5145",
            request:
                "Horizons EPHEM_TYPE=SPK COMMAND='DES=2005145;' START=1899-12-01 STOP=2100-02-01",
        },
        ObjectSpk {
            body_designation: "asteroid:7066-Nessus",
            naif_id: 2_007_066,
            sha256: "048b4d3b9f37101d81d9a374ff52c1afd1db1bf11215a3acaa5ef1d0d7c084fb",
            source_label: "jpl-sbdb-spk:7066",
            request:
                "Horizons EPHEM_TYPE=SPK COMMAND='DES=2007066;' START=1899-12-01 STOP=2100-02-01",
        },
        ObjectSpk {
            body_designation: "asteroid:10199-Chariklo",
            naif_id: 2_010_199,
            sha256: "3f3a41d7bfd57f9d13ec4c9a2a0c02f2002f076374e6f2e881575fd0fe172931",
            source_label: "jpl-sbdb-spk:10199",
            request:
                "Horizons EPHEM_TYPE=SPK COMMAND='DES=2010199;' START=1899-12-01 STOP=2100-02-01",
        },
        ObjectSpk {
            body_designation: "asteroid:8405-Asbolus",
            naif_id: 2_008_405,
            sha256: "bce77731972233a0b71bdc7e7174040f879e992f774be2a63dd65d4cc4cf88be",
            source_label: "jpl-sbdb-spk:8405",
            request:
                "Horizons EPHEM_TYPE=SPK COMMAND='DES=2008405;' START=1899-12-01 STOP=2100-02-01",
        },
        ObjectSpk {
            body_designation: "asteroid:1221-Amor",
            naif_id: 2_001_221,
            sha256: "039cd11324d48f0ddf0ef46090e8eea2207f765fb5d8f3797f6864f4d3d4d096",
            source_label: "jpl-sbdb-spk:1221",
            request:
                "Horizons EPHEM_TYPE=SPK COMMAND='DES=2001221;' START=1899-12-01 STOP=2100-02-01",
        },
        ObjectSpk {
            body_designation: "asteroid:1181-Lilith",
            naif_id: 2_001_181,
            sha256: "7c86c230a4a037c4a7292e982cc431b7506dfda9facf086c2c129e4d46fdf06c",
            source_label: "jpl-sbdb-spk:1181",
            request:
                "Horizons EPHEM_TYPE=SPK COMMAND='DES=2001181;' START=1899-12-01 STOP=2100-02-01",
        },
        ObjectSpk {
            body_designation: "asteroid:944-Hidalgo",
            naif_id: 2_000_944,
            sha256: "40d6e5c13b182654a7c5dfd681133c0489c3ada6c1c5b5863b2ec6a1babdf575",
            source_label: "jpl-sbdb-spk:944",
            request:
                "Horizons EPHEM_TYPE=SPK COMMAND='DES=2000944;' START=1899-12-01 STOP=2100-02-01",
        },
        ObjectSpk {
            body_designation: "asteroid:1566-Icarus",
            naif_id: 2_001_566,
            sha256: "266fbb4eb5521c469866c5f28078c522a21f50e0af704dfaa85ac925191f12dc",
            source_label: "jpl-sbdb-spk:1566",
            request:
                "Horizons EPHEM_TYPE=SPK COMMAND='DES=2001566;' START=1899-12-01 STOP=2100-02-01",
        },
        ObjectSpk {
            body_designation: "asteroid:1685-Toro",
            naif_id: 2_001_685,
            sha256: "2a4fc1d3550a5af57d36f1a9cdfa3b4cbbb93152ba0d5b9ee2fa537a743cfc61",
            source_label: "jpl-sbdb-spk:1685",
            request:
                "Horizons EPHEM_TYPE=SPK COMMAND='DES=2001685;' START=1899-12-01 STOP=2100-02-01",
        },
        ObjectSpk {
            body_designation: "asteroid:1862-Apollo",
            naif_id: 2_001_862,
            sha256: "c3b9cb7a0da9ac97b31f16c9c48e3a94f9559cbe5cf30cf5b21de529769f55ad",
            source_label: "jpl-sbdb-spk:1862",
            request:
                "Horizons EPHEM_TYPE=SPK COMMAND='DES=2001862;' START=1899-12-01 STOP=2100-02-01",
        },
    ]
}

/// Looks up the pinned SPK for a roster designation.
pub fn object_spk_for(designation: &str) -> Option<&'static ObjectSpk> {
    object_spk_manifest()
        .iter()
        .find(|o| o.body_designation == designation)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_sha_is_lowercase_64_hex() {
        for o in object_spk_manifest() {
            assert_eq!(o.sha256.len(), 64, "{} sha length", o.body_designation);
            assert!(
                o.sha256
                    .chars()
                    .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
                "{} sha not lowercase hex",
                o.body_designation
            );
        }
    }

    #[test]
    fn naif_ids_match_designation_number() {
        for o in object_spk_manifest() {
            let n: i32 = o
                .body_designation
                .split([':', '-'])
                .find_map(|s| s.parse().ok())
                .expect("designation has a number");
            assert_eq!(o.naif_id, 2_000_000 + n, "{} naif id", o.body_designation);
        }
    }

    #[test]
    fn source_labels_and_requests_are_present() {
        for o in object_spk_manifest() {
            assert!(
                o.source_label.starts_with("jpl-sbdb-spk:"),
                "{}",
                o.body_designation
            );
            assert!(!o.request.is_empty(), "{}", o.body_designation);
        }
    }

    #[test]
    fn lookup_round_trips() {
        for o in object_spk_manifest() {
            assert_eq!(
                object_spk_for(o.body_designation).map(|x| x.naif_id),
                Some(o.naif_id)
            );
        }
    }
}
