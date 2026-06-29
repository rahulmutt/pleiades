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
            sha256: "8ee059d7ae4a63e4d568843f320034e8681236b07dd04bb8fe6a3d0a10c847e3",
            source_label: "jpl-sbdb-spk:2060",
            request: "Horizons EPHEM_TYPE=SPK COMMAND='DES=2002060;' START=1899-12-01 STOP=2100-02-01",
        },
        ObjectSpk {
            body_designation: "asteroid:5145-Pholus",
            naif_id: 2_005_145,
            sha256: "d746b35eac636c827466c4a6ddba0495f2fbc93fb43cc1e1c769ab0e24d51468",
            source_label: "jpl-sbdb-spk:5145",
            request: "Horizons EPHEM_TYPE=SPK COMMAND='DES=2005145;' START=1899-12-01 STOP=2100-02-01",
        },
        ObjectSpk {
            body_designation: "asteroid:7066-Nessus",
            naif_id: 2_007_066,
            sha256: "6819f13ee0ebd1df54f1acfe1780d7a2c72cce53bfa1c6704f1b967160c9b0ae",
            source_label: "jpl-sbdb-spk:7066",
            request: "Horizons EPHEM_TYPE=SPK COMMAND='DES=2007066;' START=1899-12-01 STOP=2100-02-01",
        },
        ObjectSpk {
            body_designation: "asteroid:10199-Chariklo",
            naif_id: 2_010_199,
            sha256: "3ed8a859848728446649e579aad6a54fddac0a6d4402008c24afc70d508841a1",
            source_label: "jpl-sbdb-spk:10199",
            request: "Horizons EPHEM_TYPE=SPK COMMAND='DES=2010199;' START=1899-12-01 STOP=2100-02-01",
        },
        ObjectSpk {
            body_designation: "asteroid:8405-Asbolus",
            naif_id: 2_008_405,
            sha256: "3a751a602acf4fbc8ad07133008a4bac6afc6645a83a253ba54650fedce8c7e7",
            source_label: "jpl-sbdb-spk:8405",
            request: "Horizons EPHEM_TYPE=SPK COMMAND='DES=2008405;' START=1899-12-01 STOP=2100-02-01",
        },
        ObjectSpk {
            body_designation: "asteroid:1221-Amor",
            naif_id: 2_001_221,
            sha256: "a54eabd556edb738661cf2763502123ad92dc45c8acd81cbc5ade8a9ddf17fff",
            source_label: "jpl-sbdb-spk:1221",
            request: "Horizons EPHEM_TYPE=SPK COMMAND='DES=2001221;' START=1899-12-01 STOP=2100-02-01",
        },
        ObjectSpk {
            body_designation: "asteroid:1181-Lilith",
            naif_id: 2_001_181,
            sha256: "ca1fb954a11320721ac0491bca3e786bfc56e37f180ab35a1bab48f94ce05c2c",
            source_label: "jpl-sbdb-spk:1181",
            request: "Horizons EPHEM_TYPE=SPK COMMAND='DES=2001181;' START=1899-12-01 STOP=2100-02-01",
        },
        ObjectSpk {
            body_designation: "asteroid:944-Hidalgo",
            naif_id: 2_000_944,
            sha256: "df68b48935a98b8505e9f6da2609a218043f8082a3933c13113b9692424d4150",
            source_label: "jpl-sbdb-spk:944",
            request: "Horizons EPHEM_TYPE=SPK COMMAND='DES=2000944;' START=1899-12-01 STOP=2100-02-01",
        },
        ObjectSpk {
            body_designation: "asteroid:1566-Icarus",
            naif_id: 2_001_566,
            sha256: "6b0cc6f7411d09919629847183893ad170300721b3aae18711f3158b1850ef69",
            source_label: "jpl-sbdb-spk:1566",
            request: "Horizons EPHEM_TYPE=SPK COMMAND='DES=2001566;' START=1899-12-01 STOP=2100-02-01",
        },
        ObjectSpk {
            body_designation: "asteroid:1685-Toro",
            naif_id: 2_001_685,
            sha256: "492ad8aec40e908be8b0c8d5b04c2010aa3bc5bd2ccbafce1e2b2c554683edc8",
            source_label: "jpl-sbdb-spk:1685",
            request: "Horizons EPHEM_TYPE=SPK COMMAND='DES=2001685;' START=1899-12-01 STOP=2100-02-01",
        },
        ObjectSpk {
            body_designation: "asteroid:1862-Apollo",
            naif_id: 2_001_862,
            sha256: "8d4fd8c093c5638538b78d7b8b8e3b22674cf72a413d7301dffaa0be0d7602dc",
            source_label: "jpl-sbdb-spk:1862",
            request: "Horizons EPHEM_TYPE=SPK COMMAND='DES=2001862;' START=1899-12-01 STOP=2100-02-01",
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
                o.sha256.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
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
            assert!(o.source_label.starts_with("jpl-sbdb-spk:"), "{}", o.body_designation);
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
