use crate::*;

#[test]
fn packaged_artifact_roundtrips_through_codec() {
    let artifact = packaged_artifact();
    let encoded = artifact.encode().expect("packaged artifact should encode");
    let fixture = CompressedArtifact::decode(PACKAGED_ARTIFACT_FIXTURE)
        .expect("packaged artifact fixture should decode");
    assert_eq!(
        fixture.header.generation_label,
        artifact.header.generation_label
    );
    assert_eq!(fixture.bodies, artifact.bodies);
    let decoded = CompressedArtifact::decode(&encoded).expect("packaged artifact should decode");
    assert_eq!(decoded.header.generation_label, ARTIFACT_LABEL);
    assert_eq!(decoded.bodies.len(), packaged_bodies().len());
    assert_eq!(decoded.checksum, artifact.checksum().unwrap());
}

#[test]
fn packaged_backend_from_artifact_uses_supplied_metadata() {
    let mut artifact = packaged_artifact().clone();
    artifact.header.source = "external packaged artifact".to_string();

    let backend = PackagedDataBackend::from_artifact(artifact);
    let metadata = backend.metadata();

    assert_eq!(metadata.provenance.summary, "external packaged artifact");
    assert!(metadata.body_coverage.contains(&CelestialBody::Sun));
    assert!(metadata
        .supported_frames
        .contains(&CoordinateFrame::Equatorial));
}

#[cfg(feature = "packaged-artifact-path")]
#[test]
fn packaged_backend_from_path_loads_a_file_artifact() {
    let path = std::env::temp_dir().join(format!(
        "pleiades-data-packaged-artifact-{}-{}.bin",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after the Unix epoch")
            .as_nanos()
    ));
    std::fs::write(&path, PACKAGED_ARTIFACT_FIXTURE).expect("test artifact should be writable");

    let backend = PackagedDataBackend::from_path(&path)
        .expect("packaged artifact path should load successfully");
    let metadata = backend.metadata();

    assert_eq!(metadata.id.as_str(), PACKAGE_NAME);
    assert!(metadata.offline);
    assert!(metadata.body_coverage.contains(&CelestialBody::Sun));

    let _ = std::fs::remove_file(&path);
}

#[cfg(feature = "packaged-artifact-path")]
#[test]
fn packaged_artifact_from_path_rejects_corrupted_artifact() {
    let path = std::env::temp_dir().join(format!(
        "pleiades-data-packaged-artifact-corrupt-{}-{}.bin",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after the Unix epoch")
            .as_nanos()
    ));
    std::fs::write(&path, b"not a valid packaged artifact")
        .expect("corrupt artifact should be writable");

    let error = packaged_artifact_from_path(&path)
        .expect_err("corrupted packaged artifact should fail to decode");
    let error_text = error.to_string();

    match error {
        PackagedArtifactLoadError::Decode(_) => {}
        other => panic!("expected decode failure, got {other}"),
    }
    assert!(error_text.contains("failed to decode packaged artifact"));

    let _ = std::fs::remove_file(&path);
}

#[test]
fn packaged_artifact_decode_rejects_checksum_corruption() {
    let mut encoded = PACKAGED_ARTIFACT_FIXTURE.to_vec();
    let last_index = encoded.len() - 1;
    encoded[last_index] ^= 0x01;

    let error = CompressedArtifact::decode(&encoded)
        .expect_err("tampered packaged artifact should fail to decode");

    assert_eq!(
        error.kind,
        pleiades_compression::CompressionErrorKind::ChecksumMismatch
    );
}

#[test]
fn packaged_artifact_kernel_free_regeneration_decodes_the_committed_fixture() {
    // Kernel-free regeneration now decodes the committed bytes (runtime decode is
    // the only kernel-free path). The decoded artifact must validate and match
    // the fixture's decoded structure, and the kernel-free bytes accessor must
    // return the committed fixture byte-for-byte (these are exactly the bytes the
    // gated WRITE path would emit).
    let generated = regenerate_packaged_artifact();
    generated
        .validate()
        .expect("decoded packaged artifact should validate");

    // Kernel-free bytes are the committed fixture, byte-identical.
    assert_eq!(
        regenerate_packaged_artifact_bytes(),
        PACKAGED_ARTIFACT_FIXTURE
    );

    // The decoded artifact matches the fixture's decoded structure.
    let fixture = CompressedArtifact::decode(PACKAGED_ARTIFACT_FIXTURE)
        .expect("committed packaged artifact fixture should decode");
    assert_eq!(generated.header.generation_label, ARTIFACT_LABEL);
    assert_eq!(generated.bodies, fixture.bodies);
    assert_eq!(generated.checksum, fixture.checksum);
    assert_eq!(generated.bodies.len(), packaged_bodies().len());
    assert_eq!(
        generated.residual_segment_count() > 0,
        !generated.residual_bodies().is_empty()
    );
}

#[test]
fn snapshot_reconstruction_covers_only_constrained_asteroids() {
    use pleiades_jpl::reference_snapshot;
    let snapshot = reference_snapshot();
    let artifact =
        crate::regenerate::try_regenerate_packaged_artifact_from_snapshot(snapshot).unwrap();
    // Major bodies are no longer reconstructed from the snapshot; only the
    // constrained asteroid (Eros) is present.
    let bodies: Vec<_> = artifact.bodies.iter().map(|b| b.body.clone()).collect();
    assert!(bodies.iter().any(|b| matches!(b, pleiades_backend::CelestialBody::Custom(_))));
    assert!(
        !bodies.contains(&pleiades_backend::CelestialBody::Sun),
        "major bodies must not come from the snapshot path"
    );
}

#[test]
fn packaged_artifact_generation_rejects_tampered_reference_snapshot_inputs() {
    let mut snapshot = reference_snapshot().to_vec();
    snapshot[0].x_km += 1.0;

    let error = try_regenerate_packaged_artifact_from_snapshot(&snapshot)
        .expect_err("tampered reference snapshot inputs should be rejected");

    assert!(
        error
            .to_string()
            .contains("packaged artifact regeneration snapshot input at index 0 does not match the checked-in reference snapshot"),
        "unexpected validation error: {error}"
    );
}

#[test]
fn packaged_artifact_generation_validates_phase1_source_inputs() {
    validate_packaged_artifact_phase1_source_inputs()
        .expect("phase-1 source inputs should validate before regeneration");
}
