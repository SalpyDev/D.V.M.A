use dvma::engine::{create_artifact, RecoveryEngine, RecoveryState};
use tempfile::NamedTempFile;

#[test]
fn test_end_to_end_creation_and_recovery_pipeline() {
    let original_doc = b"Top secret document contents with binary bytes \x00\xFF\x10\x20.";

    let temp_png = NamedTempFile::new().unwrap();
    let png_path = temp_png.path();

    let creation_result =
        create_artifact(original_doc, png_path).expect("Artifact creation failed");

    let share_a_str = creation_result.share_a_user_string;

    let mut engine = RecoveryEngine::new();
    assert_eq!(engine.state, RecoveryState::Init);

    engine
        .ingest_image(png_path)
        .expect("Image ingestion failed");
    assert_eq!(engine.state, RecoveryState::AwaitingShareA);

    engine
        .supply_share_a(&share_a_str)
        .expect("Supplying Share A failed");

    let restored_doc = engine.execute_recovery().expect("Recovery failed");
    assert_eq!(engine.state, RecoveryState::RecoverySucceeded);

    assert_eq!(original_doc, restored_doc.as_slice());
}

#[test]
fn test_recovery_gate_blocks_without_share_a() {
    let original_doc = b"Secret payload";
    let temp_png = NamedTempFile::new().unwrap();
    let png_path = temp_png.path();

    let _res = create_artifact(original_doc, png_path).unwrap();

    let mut engine = RecoveryEngine::new();
    engine.ingest_image(png_path).unwrap();

    // Try executing recovery before supplying Share A
    let res = engine.execute_recovery();
    assert!(res.is_err());
    assert_eq!(engine.state, RecoveryState::RecoveryFailed);
}
