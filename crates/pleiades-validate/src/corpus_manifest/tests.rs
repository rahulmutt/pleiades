use super::*;

#[test]
fn a_slice_line_gives_its_rows_and_checksum() {
    let manifest = "# header\nslice x file=x.csv role=x rows=12 checksum=345\n";
    assert_eq!(slice_entry(manifest), Ok((12, 345)));
}

#[test]
fn a_slice_manifest_fails_closed() {
    for (manifest, want) in [
        ("rows=1 checksum=2\n", ManifestError::NoSliceLine),
        (
            "slice x checksum=2\n",
            ManifestError::Missing {
                field: "rows=",
                line: None,
            },
        ),
        (
            "slice x rows=1\n",
            ManifestError::Missing {
                field: "checksum=",
                line: None,
            },
        ),
    ] {
        assert_eq!(slice_entry(manifest), Err(want), "{manifest}");
    }
    for (manifest, field) in [
        ("slice x rows=-1 checksum=2\n", "rows"),
        ("slice x rows=1 checksum=0x2\n", "checksum"),
    ] {
        assert!(
            matches!(
                slice_entry(manifest),
                Err(ManifestError::Invalid { field: got, .. }) if got == field
            ),
            "{manifest}"
        );
    }
}

#[test]
fn file_lines_give_an_entry_per_file() {
    let manifest = "corpus: a+b\nfile: a.csv rows=1 checksum=2\nfile: b.csv rows=3 checksum=4\n";
    let entries = file_entries(manifest).expect("entries");
    assert_eq!(entries.len(), 2);
    assert_eq!(entries["a.csv"], (1, 2));
    assert_eq!(entries["b.csv"], (3, 4));
}

#[test]
fn a_file_manifest_fails_closed() {
    assert_eq!(file_entries("corpus: a\n"), Err(ManifestError::NoFileLines));
    assert_eq!(
        file_entries("file: a.csv rows=1\n"),
        Err(ManifestError::MalformedFileLine(
            "file: a.csv rows=1".into()
        ))
    );
    assert_eq!(
        file_entries("file: a.csv rows=1 sha=2\n"),
        Err(ManifestError::Missing {
            field: "checksum=",
            line: Some("file: a.csv rows=1 sha=2".into()),
        })
    );
    assert!(matches!(
        file_entries("file: a.csv rows=x checksum=2\n"),
        Err(ManifestError::Invalid { field: "rows", .. })
    ));
}

#[test]
fn a_labelled_manifest_reads_a_rows_line_and_a_checksum_token() {
    assert_eq!(
        labelled_entry("corpus: c\nrows: 329\nchecksum=77\n"),
        Ok((329, 77))
    );
    assert_eq!(
        labelled_entry("checksum=77\n"),
        Err(ManifestError::Missing {
            field: "rows:",
            line: None,
        })
    );
    assert_eq!(
        labelled_entry("rows: 3\n"),
        Err(ManifestError::Missing {
            field: "checksum=",
            line: None,
        })
    );
}

#[test]
fn messages_match_the_per_gate_copies() {
    assert_eq!(ManifestError::NoSliceLine.to_string(), "no slice line");
    assert_eq!(
        ManifestError::NoFileLines.to_string(),
        "no `file:` lines found in manifest"
    );
    assert_eq!(
        ManifestError::Missing {
            field: "rows=",
            line: None
        }
        .to_string(),
        "rows= missing"
    );
    let invalid = slice_entry("slice x rows=x checksum=1").unwrap_err();
    assert!(invalid.to_string().starts_with("rows: "), "{invalid}");
}

// Every committed corpus manifest parses with the layout its gate reads.
#[test]
fn every_committed_manifest_parses() {
    let data = concat!(env!("CARGO_MANIFEST_DIR"), "/data");
    let read = |corpus: &str| {
        std::fs::read_to_string(format!("{data}/{corpus}/manifest.txt"))
            .unwrap_or_else(|e| panic!("{corpus}: {e}"))
    };
    for corpus in [
        "aspects-corpus",
        "ayanamsa-apparent-corpus",
        "ayanamsa-corpus",
        "helio-position-corpus",
        "lilith-corpus",
        "mean-lunar-corpus",
        "sidereal-position-corpus",
        "stations-corpus",
        "true-node-corpus",
    ] {
        slice_entry(&read(corpus)).unwrap_or_else(|e| panic!("{corpus}: {e}"));
    }
    for corpus in [
        "eclipses-local-corpus",
        "fictitious-corpus",
        "nod-aps-corpus",
        "occultations-corpus",
        "pheno-corpus",
        "rise-trans-corpus",
    ] {
        file_entries(&read(corpus)).unwrap_or_else(|e| panic!("{corpus}: {e}"));
    }
    labelled_entry(&read("crossings-corpus")).expect("crossings-corpus");
}
