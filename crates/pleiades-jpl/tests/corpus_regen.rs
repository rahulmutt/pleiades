//! Gated: regenerates each backend slice from the real kernel and compares to
//! the checked-in CSV within a tight reproducibility tolerance. Skipped unless
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

    // (role, checked-in CSV) for every backend-generated slice.
    let cases: [(SliceRole, &str); 4] = [
        (
            SliceRole::Boundary,
            include_str!("../data/corpus/boundary.csv"),
        ),
        (
            SliceRole::InteriorBackbone,
            include_str!("../data/corpus/interior.csv"),
        ),
        (
            SliceRole::FastCluster,
            include_str!("../data/corpus/fast_clusters.csv"),
        ),
        (
            SliceRole::Holdout,
            include_str!("../data/corpus/holdout.csv"),
        ),
    ];

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

    for (role, checked_in) in cases {
        let regenerated = generate_slice(&backend, role).unwrap();
        let a = parse(&regenerated.csv);
        let b = parse(checked_in);
        assert_eq!(a.len(), b.len(), "row count drift vs checked-in {role:?}");
        for ((ka, va), (kb, vb)) in a.iter().zip(b.iter()) {
            assert_eq!(ka, kb, "epoch/body ordering drift in {role:?}");
            for i in 0..3 {
                assert!(
                    (va[i] - vb[i]).abs() < 1.0,
                    "value drift > 1 km at {ka} in {role:?}"
                );
            }
        }
    }
}

#[test]
fn regenerated_asteroid_reference_matches_checked_in() {
    let (Ok(de), Ok(ast)) = (
        std::env::var("PLEIADES_DE_KERNEL"),
        std::env::var("PLEIADES_AST_KERNEL"),
    ) else {
        eprintln!("skipping: set PLEIADES_DE_KERNEL and PLEIADES_AST_KERNEL to run");
        return;
    };
    use pleiades_jpl::spk::corpus_spec::SliceRole;
    use pleiades_jpl::{generate_slice, SpkBackend};

    let backend = SpkBackend::builder()
        .add_kernel(&de)
        .unwrap()
        .add_kernel(&ast)
        .unwrap()
        .build();

    let regenerated = generate_slice(&backend, SliceRole::AsteroidReference).unwrap();
    let checked_in = include_str!("../data/corpus/asteroid_reference.csv");

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
    assert_eq!(a.len(), b.len(), "asteroid_reference row count drift");
    for ((ka, va), (kb, vb)) in a.iter().zip(b.iter()) {
        assert_eq!(ka, kb, "asteroid_reference epoch/body ordering drift");
        for i in 0..3 {
            assert!(
                (va[i] - vb[i]).abs() < 1.0,
                "asteroid_reference {ka} coord {i} drift"
            );
        }
    }
}
