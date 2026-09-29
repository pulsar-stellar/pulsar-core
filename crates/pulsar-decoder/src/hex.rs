//! Lowercase hex encoding for the `bytes` variant of the wire taxonomy.
//!
//! The Go decoder renders `ScVal` bytes with `hex.EncodeToString`, which is
//! lowercase and unpadded. This module reproduces that exactly, and decodes it
//! back, without pulling in a dependency for two short functions. Decoding is
//! fallible and total: malformed input returns an error rather than panicking,
//! per the no-panic rule for `src/`.

const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";

/// Encodes bytes as a lowercase hex string, two digits per byte.
pub(crate) fn encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(HEX_DIGITS[(b >> 4) as usize] as char);
        out.push(HEX_DIGITS[(b & 0x0f) as usize] as char);
    }
    out
}

/// Decodes a lowercase or uppercase hex string into bytes.
///
/// Returns an error on an odd length or any non-hex byte, so a stored value that
/// has been corrupted is rejected rather than silently truncated.
pub(crate) fn decode(s: &str) -> Result<Vec<u8>, String> {
    if !s.len().is_multiple_of(2) {
        return Err(format!("hex string has odd length: {}", s.len()));
    }
    s.as_bytes()
        .chunks_exact(2)
        .map(|pair| match pair {
            [hi, lo] => Ok((from_hex(*hi)? << 4) | from_hex(*lo)?),
            // chunks_exact yields length-2 slices; the compiler cannot prove it,
            // so this arm is required and never taken.
            _ => Err("hex chunk was not two bytes".to_string()),
        })
        .collect()
}

fn from_hex(c: u8) -> Result<u8, String> {
    match c {
        b'0'..=b'9' => Ok(c - b'0'),
        b'a'..=b'f' => Ok(c - b'a' + 10),
        b'A'..=b'F' => Ok(c - b'A' + 10),
        other => Err(format!("invalid hex digit: {:?}", other as char)),
    }
}

#[cfg(test)]
mod tests {
    use super::{decode, encode};

    #[test]
    fn encode_is_lowercase_and_two_digits_per_byte() {
        assert_eq!(encode(&[0x00, 0x0f, 0xde, 0xad]), "000fdead");
    }

    #[test]
    fn encode_of_empty_is_empty() {
        assert_eq!(encode(&[]), "");
    }

    #[test]
    fn decode_reverses_encode() {
        let bytes = vec![0x00, 0x01, 0xfe, 0xff, 0x42];
        assert_eq!(decode(&encode(&bytes)).expect("round trip decodes"), bytes);
    }

    #[test]
    fn decode_accepts_uppercase() {
        assert_eq!(decode("DEAD").expect("uppercase decodes"), vec![0xde, 0xad]);
    }

    #[test]
    fn decode_rejects_odd_length() {
        assert!(decode("abc").is_err());
    }

    #[test]
    fn decode_rejects_non_hex_digit() {
        assert!(decode("zz").is_err());
    }
}
