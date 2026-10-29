use crate::core::error::{DvmaError, Result};

pub struct Base26Codec;

impl Base26Codec {
    /// Encodes a slice of bytes into an uppercase A-Z string.
    /// Each byte produces exactly 2 letters:
    /// q = byte / 26
    /// r = byte % 26
    /// letter1 = 'A' + q
    /// letter2 = 'A' + r
    pub fn encode(input: &[u8]) -> String {
        let mut output = String::with_capacity(input.len() * 2);
        for &byte in input {
            let q = byte / 26;
            let r = byte % 26;
            let char1 = (b'A' + q) as char;
            let char2 = (b'A' + r) as char;
            output.push(char1);
            output.push(char2);
        }
        output
    }

    /// Decodes an uppercase A-Z string back into bytes.
    /// Must contain an even number of characters, strictly A-Z.
    /// Rejects any letter pair where (u * 26 + v) > 255.
    pub fn decode(input: &str) -> Result<Vec<u8>> {
        if !input.len().is_multiple_of(2) {
            return Err(DvmaError::Base26DecodingError(
                "Encoded Base26 string must have an even length".into(),
            ));
        }

        let chars: Vec<char> = input.chars().collect();
        let mut output = Vec::with_capacity(chars.len() / 2);

        for chunk in chars.chunks(2) {
            let c1 = chunk[0];
            let c2 = chunk[1];

            if !c1.is_ascii_uppercase() || !c2.is_ascii_uppercase() {
                return Err(DvmaError::Base26DecodingError(format!(
                    "Invalid characters in Base26 input: '{}', '{}'. Must be uppercase A-Z",
                    c1, c2
                )));
            }

            let u = (c1 as u8) - b'A';
            let v = (c2 as u8) - b'A';

            let val = (u as u16) * 26 + (v as u16);
            if val > 255 {
                return Err(DvmaError::Base26DecodingError(format!(
                    "Base26 pair ('{}','{}') represents value {} which exceeds 255",
                    c1, c2, val
                )));
            }

            output.push(val as u8);
        }

        Ok(output)
    }
}
