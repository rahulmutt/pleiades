//! Gated: regenerates each slice from the real kernel and compares to the
//! checked-in CSV within a tight reproducibility tolerance. Skipped unless
//! PLEIADES_DE_KERNEL points at de440.bsp.

#[test]
fn regenerated_corpus_matches_checked_in() {
    let Ok(kernel) = std::env::var("PLEIADES_DE_KERNEL") else {
        eprintln!("skipping: set PLEIADES_DE_KERNEL to run");
        return;
    };
    use pleiades_jpl::spk::corpus_spec::SliceRole;
    use pleiades_jpl::{generate_slice, SpkBackend};

    let backend = SpkBackend::builder().add_kernel(&kernel).unwrap().build();
    let regenerated = generate_slice(&backend, SliceRole::Boundary).unwrap();
    let checked_in = include_str!("../data/corpus/boundary.csv");

    // Compare data rows numerically within tolerance (not byte-exact, to allow
    // float formatting differences).
    let parse = |csv: &str| -> Vec<(String, [f64; 3])> {
        csv.lines()
            .filter(|l| !l.starts_with('#') && !l.is_empty())
            .map(|l| {
                let f: Vec<&str> = l.split(',').collect();
                (
                    format!("{},{}", f[0], f[1]),
                    [
                        f[2].parse().unwrap(),
                        f[3].parse().unwrap(),
                        f[4].parse().unwrap(),
                    ],
                )
            })
            .collect()
    };
    let a = parse(&regenerated.csv);
    let b = parse(checked_in);
    assert_eq!(a.len(), b.len(), "row count drift vs checked-in corpus");
    for ((ka, va), (kb, vb)) in a.iter().zip(b.iter()) {
        assert_eq!(ka, kb, "epoch/body ordering drift");
        for i in 0..3 {
            assert!((va[i] - vb[i]).abs() < 1.0, "value drift > 1 km at {ka}");
        }
    }
}
