use dvma::codec::Base26Codec;
use dvma::core::BinaryPayload;

#[test]
fn test_base26_exhaustive_all_256_bytes() {
    let all_bytes: Vec<u8> = (0..=255).collect();
    let encoded = Base26Codec::encode(&all_bytes);
    assert_eq!(encoded.len(), 512);

    let decoded = Base26Codec::decode(&encoded).expect("Exhaustive Base26 decode failed");
    assert_eq!(all_bytes, decoded);
}

#[test]
fn test_base26_known_examples() {
    // 255 = 9 * 26 + 21 ('J', 'V')
    let encoded = Base26Codec::encode(&[255]);
    assert_eq!(encoded, "JV");

    let decoded = Base26Codec::decode("JV").unwrap();
    assert_eq!(decoded, vec![255]);

    // 0 = 0 * 26 + 0 ('A', 'A')
    let encoded_zero = Base26Codec::encode(&[0]);
    assert_eq!(encoded_zero, "AA");
    assert_eq!(Base26Codec::decode("AA").unwrap(), vec![0]);
}

#[test]
fn test_base26_invalid_inputs() {
    // Odd length
    assert!(Base26Codec::decode("ABC").is_err());

    // Non-uppercase / non-A-Z
    assert!(Base26Codec::decode("A1").is_err());
    assert!(Base26Codec::decode("aA").is_err());

    // Invalid pair exceeding 255
    // Z = 25 -> Z Z = 25 * 26 + 25 = 675 > 255
    assert!(Base26Codec::decode("ZZ").is_err());
}

#[test]
fn test_binary_payload_framing_roundtrip() {
    let nonce = [1u8; 12];
    let ciphertext = vec![10, 20, 30, 40, 50, 60, 70, 80, 90, 100];

    let payload = BinaryPayload::new(nonce, ciphertext.clone());
    let serialized = payload.serialize();

    let parsed = BinaryPayload::deserialize(&serialized).expect("Deserialization failed");
    assert_eq!(payload, parsed);
}
