use dvma::geometry::{
    angle_to_letter, decode_matrix, letter_to_angle, render_matrix, MatrixConfig, ALPHABET_SIZE,
    DEGREES_PER_LETTER,
};

#[test]
fn test_all_26_alphabet_angles() {
    for i in 0..ALPHABET_SIZE {
        let ch = (b'A' + i as u8) as char;
        let expected_angle = (i as f64) * DEGREES_PER_LETTER;
        let angle = letter_to_angle(ch).expect("Letter angle conversion failed");
        assert!(
            (angle - expected_angle).abs() < 1e-5,
            "Mismatch for letter {}: expected {}, got {}",
            ch,
            expected_angle,
            angle
        );

        let recovered_ch = angle_to_letter(angle, 2.0).expect("Angle to letter failed");
        assert_eq!(ch, recovered_ch);
    }
}

#[test]
fn test_matrix_rendering_and_decoding_roundtrip() {
    let config = MatrixConfig::default();
    let original_payload = "ABCDEFGHIJKLMNOPQRSTUVWXYZABCDEF";

    let img = render_matrix(original_payload, &config).expect("Render matrix failed");
    let decoded_payload = decode_matrix(&img, &config).expect("Decode matrix failed");

    assert_eq!(original_payload, decoded_payload);
}
