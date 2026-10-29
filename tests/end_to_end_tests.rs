use dvma::container::{load_dvma_png, save_dvma_png};
use dvma::crypto::{generate_master_key, split_master_key};
use dvma::engine::{create_artifact, RecoveryEngine};
use tempfile::NamedTempFile;

#[test]
fn test_security_invariant_wrong_share_a_fails_closed() {
    let doc = b"Top secret document contents.";

    let temp_png = NamedTempFile::new().unwrap();
    let png_path = temp_png.path();

    let _res = create_artifact(doc, png_path).unwrap();

    // Generate a completely different key and Share A
    let wrong_key = generate_master_key();
    let (wrong_share_a, _) = split_master_key(&wrong_key);
    let wrong_share_a_str = wrong_share_a.to_user_string();

    let mut engine = RecoveryEngine::new();
    engine.ingest_image(png_path).unwrap();

    let supply_res = engine.supply_share_a(&wrong_share_a_str);
    assert!(supply_res.is_err());
    assert!(engine.execute_recovery().is_err());
}

#[test]
fn test_security_invariant_tampered_png_payload_fails_authentication() {
    let doc = b"Confidential financial records";
    let temp_png = NamedTempFile::new().unwrap();
    let png_path = temp_png.path();

    let creation_res = create_artifact(doc, png_path).unwrap();

    // Manually load PNG, tamper with pixels in matrix image, and re-save
    let (mut img, share_b) = load_dvma_png(png_path).unwrap();

    // Flip pixels in payload region
    for y in 30..60 {
        for x in 30..60 {
            let pixel = img.get_pixel_mut(x, y);
            pixel[0] ^= 0xFF;
        }
    }

    save_dvma_png(png_path, &img, &share_b).unwrap();

    // Attempt recovery
    let mut engine = RecoveryEngine::new();
    engine.ingest_image(png_path).unwrap();
    engine
        .supply_share_a(&creation_res.share_a_user_string)
        .unwrap();

    let recovery_res = engine.execute_recovery();
    assert!(
        recovery_res.is_err(),
        "Recovery must fail on tampered matrix image"
    );
}

#[test]
fn test_full_roundtrip_unicode_text_and_binary() {
    let unicode_doc =
        "DVMA 协议测试 🚀 - Micro-notch Matrix Encryption Protocol v1.0. 🔒".as_bytes();

    let temp_png = NamedTempFile::new().unwrap();
    let png_path = temp_png.path();

    let creation = create_artifact(unicode_doc, png_path).unwrap();

    let mut engine = RecoveryEngine::new();
    engine.ingest_image(png_path).unwrap();
    engine
        .supply_share_a(&creation.share_a_user_string)
        .unwrap();

    let restored = engine.execute_recovery().unwrap();
    assert_eq!(unicode_doc, restored.as_slice());
}

#[test]
fn test_large_binary_payload_roundtrip() {
    // Generate 1024 bytes of arbitrary binary data
    let mut large_bin = vec![0u8; 1024];
    for (i, byte) in large_bin.iter_mut().enumerate() {
        *byte = (i % 256) as u8;
    }

    let temp_png = NamedTempFile::new().unwrap();
    let png_path = temp_png.path();

    let creation = create_artifact(&large_bin, png_path).unwrap();

    let mut engine = RecoveryEngine::new();
    engine.ingest_image(png_path).unwrap();
    engine
        .supply_share_a(&creation.share_a_user_string)
        .unwrap();

    let restored = engine.execute_recovery().unwrap();
    assert_eq!(large_bin, restored.as_slice());
}
