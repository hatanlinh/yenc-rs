//! yEnc encoding functionality

use std::io::{Read, Write};

use crate::error::Result;

const ESCAPE_CHAR: u8 = b'=';
const OFFSET: u8 = 42;
const LINE_LENGTH: usize = 128;

const ESCAPING_CHARS: [u8; 7] = [
    0x00, // NULL
    0x09, // TAB (optional, but recommended)
    0x0A, // LF
    0x0D, // CR
    0x20, // SPACE at beginning/end of line
    0x2E, // DOT at beginning of line (for SMTP)
    b'=', // Escape character itself
];

#[inline]
fn needs_escape(byte: u8, encoded: u8) -> bool {
    ESCAPING_CHARS.contains(&encoded) || byte == ESCAPE_CHAR
}

/// Encode a single byte
#[inline]
fn encode_byte(byte: u8) -> u8 {
    byte.wrapping_add(OFFSET)
}

/// Encode data from a reader and write yEnc format to a writer
///
/// # Arguments
/// * `reader` - Input reader containing raw data
/// * `writer` - Output writer for yEnc-encoded data
/// * `filename` - Name to use in the yEnc header
///
/// # Returns
/// Number of bytes read from input
pub fn encode<R: Read, W: Write>(mut reader: R, mut writer: W, filename: &str) -> Result<usize> {
    let mut input_data = Vec::new();
    reader.read_to_end(&mut input_data)?;

    let size = input_data.len();
    writeln!(
        writer,
        "=ybegin line={} size={} name={}",
        LINE_LENGTH, size, filename
    )?;

    let mut line_length = 0;
    for &byte in &input_data {
        let encoded = encode_byte(byte);

        if needs_escape(byte, encoded) {
            writer.write_all(&[ESCAPE_CHAR, encoded.wrapping_add(64)])?;
            line_length += 2;
        } else {
            writer.write_all(&[encoded])?;
            line_length += 1;
        }

        if line_length >= LINE_LENGTH {
            writeln!(writer)?;
            line_length = 0;
        }
    }

    if line_length > 0 {
        writeln!(writer)?;
    }
    writeln!(writer, "=yend size={}", size)?;

    Ok(size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_byte() {
        assert_eq!(encode_byte(0), 42);
        assert_eq!(encode_byte(1), 43);
    }

    #[test]
    fn test_encode_simple() {
        let input = vec![0u8, 1, 2, 3, 4];
        let mut output = Vec::new();

        let size = encode(&input[..], &mut output, "test.bin").unwrap();

        assert_eq!(size, 5);
        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("=ybegin"));
        assert!(output_str.contains("name=test.bin"));
    }
}
