use crate::core::error::{DvmaError, Result};
use std::f64::consts::PI;

pub const ALPHABET_SIZE: usize = 26;
pub const DEGREES_PER_LETTER: f64 = 360.0 / 26.0; // 13.846153846153847 degrees

/// Converts a letter 'A'-'Z' to its canonical micro-notch angle in degrees [0, 360).
/// A = 0.0°, B = 13.8462°, N = 180.0°, Z = 346.1538°
pub fn letter_to_angle(letter: char) -> Result<f64> {
    if !letter.is_ascii_uppercase() {
        return Err(DvmaError::GeometryError(format!(
            "Invalid character '{}' for alphabet angle lookup",
            letter
        )));
    }
    let index = (letter as u8 - b'A') as usize;
    Ok((index as f64) * DEGREES_PER_LETTER)
}

/// Converts a letter index 0..25 to canonical angle in degrees.
pub fn index_to_angle(index: usize) -> f64 {
    ((index % ALPHABET_SIZE) as f64) * DEGREES_PER_LETTER
}

/// Converts a measured angle in degrees [0, 360) to the nearest letter 'A'-'Z'.
/// Verifies that the angular deviation is within `max_tolerance_degrees`.
pub fn angle_to_letter(measured_angle_deg: f64, max_tolerance_deg: f64) -> Result<char> {
    let mut norm_angle = measured_angle_deg % 360.0;
    if norm_angle < 0.0 {
        norm_angle += 360.0;
    }

    let raw_index = norm_angle / DEGREES_PER_LETTER;
    let index = raw_index.round() as usize % ALPHABET_SIZE;
    let canonical_angle = index_to_angle(index);

    let diff = (norm_angle - canonical_angle).abs();
    let diff = diff.min(360.0 - diff);

    if diff > max_tolerance_deg {
        return Err(DvmaError::GeometryError(format!(
            "Ambiguous notch angle {:.2}°: deviation {:.2}° exceeds tolerance {:.2}°",
            norm_angle, diff, max_tolerance_deg
        )));
    }

    Ok((b'A' + index as u8) as char)
}

/// Helper to convert degrees to radians
pub fn deg_to_rad(deg: f64) -> f64 {
    deg * PI / 180.0
}

/// Helper to convert radians to degrees [0, 360)
pub fn rad_to_deg(rad: f64) -> f64 {
    let mut deg = rad * 180.0 / PI;
    deg %= 360.0;
    if deg < 0.0 {
        deg += 360.0;
    }
    deg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alphabet_angle_mapping() {
        assert_eq!(letter_to_angle('A').unwrap(), 0.0);
        assert!((letter_to_angle('B').unwrap() - 13.8461538).abs() < 1e-5);
        assert_eq!(letter_to_angle('N').unwrap(), 180.0);
        assert!((letter_to_angle('Z').unwrap() - 346.153846).abs() < 1e-5);
    }

    #[test]
    fn test_angle_to_letter_rounding_and_tolerance() {
        assert_eq!(angle_to_letter(0.1, 5.0).unwrap(), 'A');
        assert_eq!(angle_to_letter(359.9, 5.0).unwrap(), 'A');
        assert_eq!(angle_to_letter(13.8, 5.0).unwrap(), 'B');
        assert_eq!(angle_to_letter(180.2, 5.0).unwrap(), 'N');

        // Outside tolerance
        assert!(angle_to_letter(6.9, 2.0).is_err());
    }
}
