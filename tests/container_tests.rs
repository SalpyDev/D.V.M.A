use dvma::container::{load_dvma_png, save_dvma_png};
use dvma::crypto::{generate_master_key, split_master_key};
use dvma::geometry::{render_matrix, MatrixConfig};
use tempfile::NamedTempFile;

#[test]
fn test_png_save_and_load_with_custom_chunk() {
    let key = generate_master_key();
    let (_, share_b) = split_master_key(&key);

    let matrix_img = render_matrix("ABCDEFGHIJKLMNOPQRSTUVWXYZ", &MatrixConfig::default())
        .expect("Render matrix failed");

    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let temp_path = temp_file.path();

    save_dvma_png(temp_path, &matrix_img, &share_b).expect("Failed to save DVMA PNG");

    let (loaded_img, loaded_share_b) = load_dvma_png(temp_path).expect("Failed to load DVMA PNG");

    assert_eq!(matrix_img.dimensions(), loaded_img.dimensions());
    assert_eq!(share_b, loaded_share_b);
}
