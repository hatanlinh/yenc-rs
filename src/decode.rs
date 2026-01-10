//! yEnc decoding functionality

use std::io::{BufRead, BufReader, Read, Write};

use crc32fast::Hasher;

use crate::consts::{ESCAPE_CHAR, ESCAPE_OFFSET, ESCAPING_CHARS, OFFSET};
use crate::error::{Result, YencError};
use crate::header::{YencHeader, YencTrailer};

/// Decode a single yEnc-encoded byte
#[inline]
fn decode_byte(byte: u8) -> u8 {
    byte.wrapping_sub(OFFSET)
}

/// Trim whitespaces at the beginning and end of a byte slice
#[inline]
fn trim_bytes(line: &[u8]) -> &[u8] {
    let is_ws = |b: &u8| b" \t\r\n".contains(b);
    let start = line.iter().position(|b| !is_ws(b)).unwrap_or(line.len());
    let end = line
        .iter()
        .rposition(|b| !is_ws(b))
        .map(|i| i + 1)
        .unwrap_or(0);
    &line[start..end]
}

/// Decoder with configurable options
#[derive(Debug, Clone)]
pub struct Decoder {
    strict: bool,
    validate_crc: bool,
}

impl Default for Decoder {
    fn default() -> Self {
        Self {
            strict: false,
            validate_crc: true,
        }
    }
}

impl Decoder {
    /// Create a new decoder with default settings
    ///
    /// Default settings:
    /// - Lenient mode (accepts any escaped character)
    /// - CRC validation enabled
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable strict validation of escape sequences
    ///
    /// When enabled, only characters that should be escaped according to
    /// the yEnc spec are accepted. Invalid escape sequences will cause an error.
    pub fn strict(mut self) -> Self {
        self.strict = true;
        self
    }

    /// Disable CRC validation
    ///
    /// By default, CRC32 checksums are validated if present in the trailer.
    pub fn no_crc_check(mut self) -> Self {
        self.validate_crc = false;
        self
    }

    /// Decode yEnc data from a reader and write to a writer
    ///
    /// # Arguments
    /// * `reader` - Input reader containing yEnc-encoded data
    /// * `writer` - Output writer for decoded data
    ///
    /// # Returns
    /// A tuple of (header, trailer, bytes_written)
    ///
    /// # Example
    /// ```
    /// use yenc::Decoder;
    ///
    /// let input = b"=ybegin line=128 size=5 name=test.bin\nABCDE\n=yend size=5\n";
    /// let mut output = Vec::new();
    ///
    /// let (header, trailer, size) = Decoder::new()
    ///     .strict()
    ///     .decode(&input[..], &mut output)
    ///     .unwrap();
    /// ```
    pub fn decode<R: Read, W: Write>(
        &self,
        mut reader: R,
        mut writer: W,
    ) -> Result<(YencHeader, Option<YencTrailer>, usize)> {
        let mut buf_reader = BufReader::new(&mut reader);
        let mut line = Vec::new();

        let header = loop {
            line.clear();
            let bytes_read = buf_reader.read_until(b'\n', &mut line)?;
            if bytes_read == 0 {
                return Err(YencError::InvalidHeader("No header found".to_string()));
            }

            let trimmed = trim_bytes(&line);
            if trimmed.starts_with(b"=ybegin ") {
                if let Ok(header_text) = std::str::from_utf8(trimmed) {
                    break YencHeader::parse(header_text)?;
                } else {
                    return Err(YencError::InvalidHeader("Invalid header".to_string()));
                }
            }
        };

        line.clear();
        let bytes_read = buf_reader.read_until(b'\n', &mut line)?;
        if bytes_read == 0 {
            return Err(YencError::InvalidData("No data found".to_string()));
        }

        let trimmed = trim_bytes(&line);
        if trimmed.starts_with(b"=ypart ") {
            // TODO: Handle multi-parts
            line.clear();
            let bytes_read = buf_reader.read_until(b'\n', &mut line)?;
            if bytes_read == 0 {
                return Err(YencError::InvalidData("No data found".to_string()));
            }
        }

        // Initialize CRC32 hasher if validation is enabled
        let mut crc_hasher = if self.validate_crc {
            Some(Hasher::new())
        } else {
            None
        };

        let mut bytes_written = 0;
        let mut escaped = false;
        loop {
            let trimmed = trim_bytes(&line);
            if trimmed.starts_with(b"=yend ") {
                if let Ok(trailer_text) = std::str::from_utf8(trimmed) {
                    let trailer = YencTrailer::parse(trailer_text)?;

                    if let Some(hasher) = crc_hasher {
                        let computed_crc = hasher.finalize();

                        // For single-part files, validate against crc32 field
                        // For multi-part files, validate against pcrc32 field (TODO: handle multi-parts)
                        if let Some(expected_crc) = trailer.crc32 {
                            if computed_crc != expected_crc {
                                return Err(YencError::CrcMismatch {
                                    expected: expected_crc,
                                    actual: computed_crc,
                                });
                            }
                        }
                        // Note: CRC is optional, so if not present we don't fail
                    }

                    return Ok((header, Some(trailer), bytes_written));
                } else {
                    return Err(YencError::InvalidData("Invalid trailer".to_string()));
                }
            }

            for &byte in trimmed {
                if byte == ESCAPE_CHAR {
                    escaped = true;
                    continue;
                }

                let decoded = if escaped {
                    escaped = false;
                    let result = decode_byte(byte.wrapping_sub(ESCAPE_OFFSET));

                    if self.strict && !ESCAPING_CHARS.contains(&result) {
                        return Err(YencError::InvalidData(format!(
                            "Invalid escape sequence: ={:02x}",
                            byte
                        )));
                    }
                    result
                } else {
                    decode_byte(byte)
                };

                // Update CRC if validation is enabled
                if let Some(ref mut hasher) = crc_hasher {
                    hasher.update(&[decoded]);
                }

                writer.write_all(&[decoded])?;
                bytes_written += 1;
            }

            line.clear();
            let bytes_read = buf_reader.read_until(b'\n', &mut line)?;
            if bytes_read == 0 {
                break;
            }
        }

        if escaped {
            return Err(YencError::InvalidData(
                "File ended with incomplete escape sequence".to_string(),
            ));
        }

        Ok((header, None, bytes_written))
    }
}

/// Decode yEnc data with default settings (lenient mode, CRC validation enabled)
///
/// This is a convenience function equivalent to `Decoder::new().decode(reader, writer)`
///
/// # Example
/// ```
/// use yenc::decode;
///
/// let input = b"=ybegin line=128 size=5 name=test.bin\nKLMNO\n=yend size=5\n";
/// let mut output = Vec::new();
///
/// let (header, trailer, size) = decode(&input[..], &mut output).unwrap();
/// ```
pub fn decode<R: Read, W: Write>(
    reader: R,
    writer: W,
) -> Result<(YencHeader, Option<YencTrailer>, usize)> {
    Decoder::default().decode(reader, writer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_byte() {
        assert_eq!(decode_byte(b'*'), 0);
        assert_eq!(decode_byte(b'+'), 1);
        assert_eq!(decode_byte(b','), 2);
    }

    #[test]
    fn test_decode_simple() {
        let input = b"=ybegin line=128 size=5 name=test.bin\nKLMNO\n=yend size=5\n";
        let mut output = Vec::new();

        let (header, _, size) = decode(&input[..], &mut output).unwrap();

        assert_eq!(header.name, "test.bin");
        assert_eq!(header.size, 5);
        assert_eq!(size, 5);
        assert_eq!(output, vec![33, 34, 35, 36, 37]);
    }

    #[test]
    fn test_decoder_builder() {
        let input = b"=ybegin line=128 size=5 name=test.bin\nKLMNO\n=yend size=5\n";
        let mut output = Vec::new();

        // Using builder
        let (header, _, _) = Decoder::new()
            .strict()
            .no_crc_check()
            .decode(&input[..], &mut output)
            .unwrap();

        assert_eq!(header.name, "test.bin");
    }

    #[test]
    fn test_invalid_escape_sequence() {
        let input = b"=ybegin line=128 size=1 name=test.bin\n=a\n=yend size=1\n";
        let mut output = Vec::new();

        let result = Decoder::new().strict().decode(&input[..], &mut output);

        assert!(result.is_err());
        match result.unwrap_err() {
            YencError::InvalidData(msg) => {
                assert!(msg.contains("Invalid escape sequence"));
            }
            other => panic!("Expected InvalidData, got {:?}", other),
        }
    }

    #[test]
    fn test_decode_with_valid_crc() {
        // Encoded data with valid CRC32
        let input = b"=ybegin line=128 size=5 name=test.bin\n*+,-=n\n=yend size=5 crc32=515ad3cc\n";
        let mut output = Vec::new();

        let (header, trailer, size) = decode(&input[..], &mut output).unwrap();

        assert_eq!(header.name, "test.bin");
        assert_eq!(size, 5);
        assert_eq!(output, vec![0, 1, 2, 3, 4]);
        assert_eq!(trailer.unwrap().crc32, Some(0x515ad3cc));
    }

    #[test]
    fn test_decode_with_invalid_crc() {
        // Encoded data with incorrect CRC32
        let input = b"=ybegin line=128 size=5 name=test.bin\n*+,-=n\n=yend size=5 crc32=ffffffff\n";
        let mut output = Vec::new();

        let result = decode(&input[..], &mut output);

        assert!(result.is_err());
        match result.unwrap_err() {
            YencError::CrcMismatch { expected, actual } => {
                assert_eq!(expected, 0xffffffff);
                assert_eq!(actual, 0x515ad3cc);
            }
            other => panic!("Expected CrcMismatch, got {:?}", other),
        }
    }

    #[test]
    fn test_decode_no_crc_check() {
        // Even with wrong CRC, should pass when validation is disabled
        let input = b"=ybegin line=128 size=5 name=test.bin\n*+,-=n\n=yend size=5 crc32=ffffffff\n";
        let mut output = Vec::new();

        let result = Decoder::new()
            .no_crc_check()
            .decode(&input[..], &mut output);

        assert!(result.is_ok());
        assert_eq!(output, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_decode_without_crc_in_trailer() {
        // No CRC in trailer - should pass even with validation enabled
        let input = b"=ybegin line=128 size=5 name=test.bin\n*+,-=n\n=yend size=5\n";
        let mut output = Vec::new();

        let result = decode(&input[..], &mut output);

        assert!(result.is_ok());
        assert_eq!(output, vec![0, 1, 2, 3, 4]);
    }
}
