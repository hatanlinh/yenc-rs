//! yEnc encoding functionality

use std::io::{Read, Write};

use crate::consts::{ESCAPE_CHAR, ESCAPE_OFFSET, ESCAPING_CHARS, LINE_LENGTH, OFFSET};
use crate::error::Result;

#[inline]
fn needs_escape(byte: u8, encoded: u8) -> bool {
    ESCAPING_CHARS.contains(&encoded) || byte == ESCAPE_CHAR
}

/// Encode a single byte
#[inline]
fn encode_byte(byte: u8) -> u8 {
    byte.wrapping_add(OFFSET)
}

/// Encoder with configurable options
#[derive(Debug, Clone)]
pub struct Encoder {
    line_length: usize,
    compute_crc: bool,
}

impl Default for Encoder {
    fn default() -> Self {
        Self {
            line_length: LINE_LENGTH,
            compute_crc: true,
        }
    }
}

impl Encoder {
    /// Create a new encoder with default settings
    ///
    /// Default settings:
    /// - Line length: 128 characters
    /// - CRC32 computation enabled
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the line length for encoded output
    ///
    /// Standard yEnc uses 128 characters per line.
    pub fn line_length(mut self, length: usize) -> Self {
        self.line_length = length;
        self
    }

    /// Disable CRC32 computation in the trailer
    pub fn no_crc(mut self) -> Self {
        self.compute_crc = false;
        self
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
    pub fn encode<R: Read, W: Write>(
        &self,
        mut reader: R,
        mut writer: W,
        filename: &str,
    ) -> Result<usize> {
        let mut input_data = Vec::new();
        reader.read_to_end(&mut input_data)?;

        let size = input_data.len();
        writeln!(
            writer,
            "=ybegin line={} size={} name={}",
            self.line_length, size, filename
        )?;

        let mut line_length = 0;
        for &byte in &input_data {
            let encoded = encode_byte(byte);

            if needs_escape(byte, encoded) {
                writer.write_all(&[ESCAPE_CHAR, encoded.wrapping_add(ESCAPE_OFFSET)])?;
                line_length += 2;
            } else {
                writer.write_all(&[encoded])?;
                line_length += 1;
            }

            if line_length >= self.line_length {
                writeln!(writer)?;
                line_length = 0;
            }
        }

        if line_length > 0 {
            writeln!(writer)?;
        }

        if self.compute_crc {
            // TODO: Compute actual CRC32
            writeln!(writer, "=yend size={}", size)?;
        } else {
            writeln!(writer, "=yend size={}", size)?;
        }

        Ok(size)
    }
}

/// Encode data with default settings
///
/// This is a convenience function equivalent to `Encoder::new().encode(reader, writer, filename)`
pub fn encode<R: Read, W: Write>(reader: R, writer: W, filename: &str) -> Result<usize> {
    Encoder::new().encode(reader, writer, filename)
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
