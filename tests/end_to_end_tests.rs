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

#[test]
fn test_valt_style_company_document_roundtrip() {
    let document = r#"VALT HARBOR SYSTEMS - CUSTOMER REFERENCE BRIEF
Fictional document for DVMA end-to-end verification

1. COMPANY PROFILE

Valt Harbor Systems is a fictional infrastructure company based in Port Merrow. Founded in 2016 by Imani Okafor-Reyes and Lars Tennyson-Wu, Valt builds privacy-first software for organizations that need to store, review, and verify sensitive records. Its operating principle is Proof Before Promise: each security claim must be independently testable.

2. PRODUCTS

Dockside is a client-side encrypted file locker for small teams. Vaultline provides policy-controlled storage, regional key custody, and append-only audit records for regulated customers. Lantern supports end-to-end encrypted review rooms with configurable retention. Ember computes approved aggregate queries over protected datasets without exposing individual records. Beacon monitors access patterns and explains unusual activity in plain language.

3. OPERATIONS

Valt serves fictional customers including the Northern Shipping Consortium, Alderwick Regional Hospital Network, and the City of Tamsin Falls. Production access uses short-lived credentials, two-person approval, and a recorded review trail. Engineers rotate through support, incident response, and documentation duty so that reliability work remains close to customer experience.

4. GOVERNANCE

The company publishes a quarterly transparency report, maintains a standing security research program, and promises to notify affected customers within twelve hours of confirming an incident. Its 2026 Horizon plan covers post-quantum migration, data sovereignty, efficient computing, and open verification tooling. All names, places, figures, and products in this document are invented for testing.

END OF REFERENCE BRIEF
"#
    .as_bytes();

    let temp_png = NamedTempFile::new().unwrap();
    let creation = create_artifact(document, temp_png.path()).unwrap();

    let mut engine = RecoveryEngine::new();
    engine.ingest_image(temp_png.path()).unwrap();
    engine
        .supply_share_a(&creation.share_a_user_string)
        .unwrap();

    let restored = engine.execute_recovery().unwrap();
    assert_eq!(document, restored.as_slice());
}

#[test]
fn test_operational_report_with_mixed_bytes_roundtrip() {
    let mut document = br#"NORTHSTAR OPERATIONS - INCIDENT REVIEW
Case: NS-2026-041 | Status: Closed | Classification: Internal

Summary:
At 03:14 UTC, a fictional telemetry worker reported delayed audit events in the Merrow region. No customer plaintext was exposed. The response team isolated the queue, rotated the service credential, replayed the signed records, and confirmed that all expected event identifiers were present.

Timeline:
03:14 alert received
03:19 queue isolated
03:27 credential rotated
03:52 replay completed
04:08 customer verification finished

Corrective actions:
1. Add a bounded replay window to the collector.
2. Require an independent approval for emergency credential rotation.
3. Publish a plain-language status note within twelve hours.

Reviewer notes: cafe access logs, Munchen region, and the phrase "proof before promise" were intentionally retained.
"#.to_vec();
    document.extend_from_slice(&[0x00, 0xFF, 0x10, 0x20, 0x7F]);

    let temp_png = NamedTempFile::new().unwrap();
    let creation = create_artifact(&document, temp_png.path()).unwrap();

    let mut engine = RecoveryEngine::new();
    engine.ingest_image(temp_png.path()).unwrap();
    engine
        .supply_share_a(&creation.share_a_user_string)
        .unwrap();

    let restored = engine.execute_recovery().unwrap();
    assert_eq!(document, restored.as_slice());
}
