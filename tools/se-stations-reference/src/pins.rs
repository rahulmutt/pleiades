//! The Swiss Ephemeris data files the `--asteroids` mode reads, pinned by
//! SHA-256 and verified fail-closed before any row is written. The files are
//! gitignored (`data/`); these constants are the committed provenance. Source:
//! https://raw.githubusercontent.com/aloistr/swisseph/master/ephe/ (the same
//! source and the same sepl/semo digests as `tools/se-nodaps-reference`).

/// `(file name, SHA-256)`. `seas_18` holds the asteroids; a geocentric
/// asteroid also needs the Earth, which SE derives from the Earth–Moon
/// barycentre (`sepl_18`) and the Moon (`semo_18`).
pub const PINNED_FILES: [(&str, &str); 3] = [
    (
        "seas_18.se1",
        "a2cd8fc33807c78ca9a700c91c2e042258b12fc4796519e00781440b5ad8b2e2",
    ),
    (
        "sepl_18.se1",
        "ca1393ceab3a44fbc895887cf789c68819ae6a1cbc9b22225872dbe4ccd99a66",
    ),
    (
        "semo_18.se1",
        "1ca07bd67c24374d77226180c20a4f9996cba013697894810518e7eb582ca4f7",
    ),
];

/// Checks every pinned file in `ephe_dir`. `Err` names the first missing or
/// mismatched file and how to fetch it.
pub fn verify_swieph_files(ephe_dir: &str) -> Result<(), String> {
    for (name, want) in PINNED_FILES {
        let path = format!("{ephe_dir}/{name}");
        let bytes = std::fs::read(&path).map_err(|e| {
            format!(
                "cannot read {path}: {e}\nDownload: curl -fLo {path} \
                 https://raw.githubusercontent.com/aloistr/swisseph/master/ephe/{name}"
            )
        })?;
        let got = sha256_hex(&bytes);
        if got != want {
            return Err(format!(
                "SHA-256 mismatch for {path}: got {got}, pinned {want}"
            ));
        }
    }
    Ok(())
}

/// FNV-1a 64-bit over the CSV text, the scheme every corpus manifest uses.
pub fn fnv1a64(text: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0001_0000_01b3);
    }
    hash
}

/// Minimal embedded SHA-256 (FIPS 180-4), public-domain-style, no deps.
/// Used only to pin the two SWIEPH data files; unit-tested below against the
/// canonical b"abc" digest.
pub fn sha256_hex(data: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let bit_len = (data.len() as u64).wrapping_mul(8);
    let mut msg = data.to_vec();
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());
    for chunk in msg.chunks_exact(64) {
        let mut w = [0u32; 64];
        for (i, word) in w.iter_mut().take(16).enumerate() {
            *word = u32::from_be_bytes([
                chunk[4 * i],
                chunk[4 * i + 1],
                chunk[4 * i + 2],
                chunk[4 * i + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = h;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ (!e & g);
            let t1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        for (hi, v) in h.iter_mut().zip([a, b, c, d, e, f, g, hh]) {
            *hi = hi.wrapping_add(v);
        }
    }
    h.iter().map(|x| format!("{x:08x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_matches_fips_vectors() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn fnv1a64_matches_the_repo_scheme() {
        // fnv1a64("") is the offset basis. The repo's prime is 0x1_0000_01b3
        // (not the textbook 0x100_0000_01b3), so "a" differs from the published
        // FNV-1a vector 0xaf63dc4c8601ec8c; the manifests are built with this one.
        assert_eq!(fnv1a64(""), 0xcbf2_9ce4_8422_2325);
        assert_eq!(fnv1a64("a"), 0x1162_bb90_8601_ec8c);
    }

    #[test]
    fn verification_fails_closed_on_a_missing_or_wrong_file() {
        let dir = std::env::temp_dir().join(format!("se-pins-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let dir_str = dir.to_str().unwrap();
        assert!(verify_swieph_files(dir_str).is_err(), "missing files must fail");
        for (name, _) in PINNED_FILES {
            std::fs::write(dir.join(name), b"not the real file").unwrap();
        }
        let err = verify_swieph_files(dir_str).unwrap_err();
        assert!(err.contains("SHA-256 mismatch"), "{err}");
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
