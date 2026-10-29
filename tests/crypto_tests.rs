use dvma::crypto::{
    decrypt_document, encrypt_document, generate_master_key, reconstruct_master_key,
    split_master_key, KeyShare,
};

#[test]
fn test_encryption_decryption_roundtrip() {
    let key = generate_master_key();
    let plaintext =
        b"Hello, DVMA Protocol World! Sensitive binary document contents \x00\x01\x02\xFF.";

    let payload = encrypt_document(&key, plaintext).expect("Encryption failed");
    let decrypted = decrypt_document(&key, &payload).expect("Decryption failed");

    assert_eq!(plaintext, decrypted.as_slice());
}

#[test]
fn test_encryption_authentication_failure_on_tamper() {
    let key = generate_master_key();
    let plaintext = b"Secret document contents";

    let mut payload = encrypt_document(&key, plaintext).expect("Encryption failed");

    // Tamper with ciphertext
    if let Some(byte) = payload.ciphertext_and_tag.first_mut() {
        *byte ^= 0xFF;
    }

    let result = decrypt_document(&key, &payload);
    assert!(
        result.is_err(),
        "Decryption should fail on tampered ciphertext"
    );
}

#[test]
fn test_shamir_secret_sharing_reconstruction() {
    let key = generate_master_key();
    let (share_a, share_b) = split_master_key(&key);

    assert_ne!(share_a.data, key.to_vec());
    assert_ne!(share_b.data, key.to_vec());
    assert_ne!(share_a.data, share_b.data);

    let reconstructed =
        reconstruct_master_key(&share_a, &share_b).expect("Key reconstruction failed");

    assert_eq!(key.as_slice(), reconstructed.as_slice());
}

#[test]
fn test_shamir_mismatched_share_set_rejected() {
    let key1 = generate_master_key();
    let key2 = generate_master_key();

    let (share_a1, _) = split_master_key(&key1);
    let (_, share_b2) = split_master_key(&key2);

    let result = reconstruct_master_key(&share_a1, &share_b2);
    assert!(result.is_err(), "Mismatched shares must be rejected");
}

#[test]
fn test_share_a_user_string_roundtrip() {
    let key = generate_master_key();
    let (share_a, _) = split_master_key(&key);

    let user_str = share_a.to_user_string();
    assert!(user_str.starts_with("DVMA-SHARE-A-"));

    let parsed_share_a =
        KeyShare::from_user_string(&user_str).expect("Failed to parse Share A string");

    assert_eq!(share_a, parsed_share_a);
}
