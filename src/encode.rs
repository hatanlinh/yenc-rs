//! yEnc encoding functionality

use std::io::{Read, Write};

use crc32fast::Hasher;

use crate::consts::{ESCAPE_CHAR, ESCAPE_OFFSET, ESCAPING_CHARS, LINE_LENGTH, OFFSET};
use crate::error::{Result, YencError};

#[inline]
fn needs_escape(byte: u8, encoded: u8) -> bool {
    ESCAPING_CHARS.contains(&encoded) || byte == ESCAPE_CHAR
}

/// Encode a single byte
#[inline]
fn encode_byte(byte: u8) -> u8 {
    byte.wrapping_add(OFFSET)
}

/// Multi-part encoding configuration
#[derive(Debug, Clone)]
pub struct MultiPartInfo {
    /// Part number (1-based)
    pub part: usize,
    /// Total number of parts
    pub total: usize,
    /// Starting byte position in original file (1-based, inclusive)
    pub begin: usize,
    /// Ending byte position in original file (1-based, inclusive)
    pub end: usize,
    /// Full file size (not just this part)
    pub full_size: usize,
    /// Optional: Full file CRC32 (typically included in last part only)
    pub full_crc: Option<u32>,
}

impl MultiPartInfo {
    /// Create a new multi-part configuration
    ///
    /// # Arguments
    /// * `part` - Part number (1-based)
    /// * `total` - Total number of parts
    /// * `begin` - Starting byte position (1-based, inclusive)
    /// * `end` - Ending byte position (1-based, inclusive)
    /// * `full_size` - Total file size
    pub fn new(part: usize, total: usize, begin: usize, end: usize, full_size: usize) -> Self {
        Self {
            part,
            total,
            begin,
            end,
            full_size,
            full_crc: None,
        }
    }

    /// Set the full file CRC32 (typically for last part)
    pub fn with_full_crc(mut self, crc: u32) -> Self {
        self.full_crc = Some(crc);
        self
    }

    /// Calculate expected part size (end - begin + 1)
    pub fn expected_size(&self) -> usize {
        self.end - self.begin + 1
    }
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

        // Compute CRC32 of original data if enabled
        let crc32 = if self.compute_crc {
            let mut hasher = Hasher::new();
            hasher.update(&input_data);
            Some(hasher.finalize())
        } else {
            None
        };

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

        // Write trailer with CRC32 if computed
        if let Some(crc) = crc32 {
            writeln!(writer, "=yend size={} crc32={:08x}", size, crc)?;
        } else {
            writeln!(writer, "=yend size={}", size)?;
        }

        Ok(size)
    }

    /// Encode a single part of a multi-part file
    ///
    /// # Arguments
    /// * `reader` - Input reader containing raw data for this part
    /// * `writer` - Output writer for yEnc-encoded data
    /// * `filename` - Name to use in the yEnc header
    /// * `part_info` - Multi-part configuration
    ///
    /// # Returns
    /// Number of bytes read from input
    ///
    /// # Errors
    /// Returns error if the input size doesn't match the expected part size
    ///
    /// # Example
    /// ```
    /// use yenc::{Encoder, MultiPartInfo};
    ///
    /// // Encode part 1 of 2 (bytes 1-5 of a 10-byte file)
    /// let data = vec![0u8, 1, 2, 3, 4];
    /// let mut output = Vec::new();
    ///
    /// let part_info = MultiPartInfo::new(1, 2, 1, 5, 10);
    ///
    /// Encoder::new()
    ///     .encode_part(&data[..], &mut output, "file.bin", &part_info)
    ///     .unwrap();
    /// ```
    pub fn encode_part<R: Read, W: Write>(
        &self,
        mut reader: R,
        mut writer: W,
        filename: &str,
        part_info: &MultiPartInfo,
    ) -> Result<usize> {
        let mut input_data = Vec::new();
        reader.read_to_end(&mut input_data)?;

        let part_size = input_data.len();
        let expected_size = part_info.expected_size();

        // Validate that input size matches expected part size
        if part_size != expected_size {
            return Err(YencError::InvalidData(format!(
                "Part size mismatch: expected {} bytes (from begin={} end={}), but got {} bytes",
                expected_size, part_info.begin, part_info.end, part_size
            )));
        }

        // Compute part CRC32 if enabled
        let part_crc = if self.compute_crc {
            let mut hasher = Hasher::new();
            hasher.update(&input_data);
            Some(hasher.finalize())
        } else {
            None
        };

        // Write multi-part header
        writeln!(
            writer,
            "=ybegin part={} total={} line={} size={} name={}",
            part_info.part, part_info.total, self.line_length, part_info.full_size, filename
        )?;

        // Write part line
        writeln!(
            writer,
            "=ypart begin={} end={}",
            part_info.begin, part_info.end
        )?;

        // Encode data
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

        // Write trailer
        write!(writer, "=yend size={} part={}", part_size, part_info.part)?;

        // Add part CRC if computed
        if let Some(pcrc) = part_crc {
            write!(writer, " pcrc32={:08x}", pcrc)?;
        }

        // Add full file CRC if provided
        if let Some(full_crc) = part_info.full_crc {
            write!(writer, " crc32={:08x}", full_crc)?;
        }

        writeln!(writer)?;

        Ok(part_size)
    }
}

/// Encode data with default settings
///
/// This is a convenience function equivalent to `Encoder::new().encode(reader, writer, filename)`
pub fn encode<R: Read, W: Write>(reader: R, writer: W, filename: &str) -> Result<usize> {
    Encoder::new().encode(reader, writer, filename)
}

/// Encode a part with default encoder settings
///
/// This is a convenience function equivalent to:
/// `Encoder::new().encode_part(reader, writer, filename, part_info)`
pub fn encode_part<R: Read, W: Write>(
    reader: R,
    writer: W,
    filename: &str,
    part_info: &MultiPartInfo,
) -> Result<usize> {
    Encoder::new().encode_part(reader, writer, filename, part_info)
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

        // Verify CRC32 is present (default behavior)
        assert!(output_str.contains("crc32="));
        let crc_line = output_str.lines().last().unwrap();
        assert!(crc_line.starts_with("=yend"));

        // CRC32 for [0, 1, 2, 3, 4] is 0x515ad3cc
        assert!(crc_line.contains("crc32=515ad3cc"));
    }

    #[test]
    fn test_encode_no_crc() {
        let input = vec![0u8, 1, 2, 3, 4];
        let mut output = Vec::new();

        Encoder::new().no_crc().encode(&input[..], &mut output, "test.bin").unwrap();

        let output_str = String::from_utf8(output).unwrap();
        assert!(!output_str.contains("crc32="));
    }

    #[test]
    fn test_encode_multipart_basic() {
        let data = vec![0u8, 1, 2, 3, 4];
        let mut output = Vec::new();

        let part_info = MultiPartInfo::new(1, 2, 1, 5, 10);

        Encoder::new()
            .encode_part(&data[..], &mut output, "test.bin", &part_info)
            .unwrap();

        let output_str = String::from_utf8(output).unwrap();

        // Verify header
        assert!(output_str.contains("=ybegin part=1 total=2"));
        assert!(output_str.contains("size=10")); // Full file size
        assert!(output_str.contains("name=test.bin"));

        // Verify part line
        assert!(output_str.contains("=ypart begin=1 end=5"));

        // Verify trailer
        let trailer = output_str.lines().last().unwrap();
        assert!(trailer.starts_with("=yend size=5 part=1")); // Part size
        assert!(trailer.contains("pcrc32=515ad3cc")); // Part CRC
    }

    #[test]
    fn test_encode_multipart_with_full_crc() {
        let data = vec![5u8, 6, 7, 8, 9];
        let mut output = Vec::new();

        let part_info = MultiPartInfo::new(2, 2, 6, 10, 10)
            .with_full_crc(0x12345678); // Full file CRC

        Encoder::new()
            .encode_part(&data[..], &mut output, "test.bin", &part_info)
            .unwrap();

        let output_str = String::from_utf8(output).unwrap();
        let trailer = output_str.lines().last().unwrap();

        assert!(trailer.contains("pcrc32=")); // Part CRC
        assert!(trailer.contains("crc32=12345678")); // Full file CRC
    }

    #[test]
    fn test_encode_multipart_size_mismatch() {
        let data = vec![0u8, 1, 2]; // Only 3 bytes
        let mut output = Vec::new();

        // Says it should be 5 bytes (begin=1 end=5)
        let part_info = MultiPartInfo::new(1, 2, 1, 5, 10);

        let result = Encoder::new()
            .encode_part(&data[..], &mut output, "test.bin", &part_info);

        assert!(result.is_err());
        match result.unwrap_err() {
            YencError::InvalidData(msg) => {
                assert!(msg.contains("Part size mismatch"));
            }
            other => panic!("Expected InvalidData, got {:?}", other),
        }
    }

    #[test]
    fn test_encode_multipart_no_crc() {
        let data = vec![0u8, 1, 2, 3, 4];
        let mut output = Vec::new();

        let part_info = MultiPartInfo::new(1, 1, 1, 5, 5);

        Encoder::new()
            .no_crc()
            .encode_part(&data[..], &mut output, "test.bin", &part_info)
            .unwrap();

        let output_str = String::from_utf8(output).unwrap();
        assert!(!output_str.contains("pcrc32=")); // No CRC computed
    }

    #[test]
    fn test_multipart_info_expected_size() {
        let info = MultiPartInfo::new(1, 10, 1, 100, 1000);
        assert_eq!(info.expected_size(), 100);

        let info = MultiPartInfo::new(2, 10, 101, 200, 1000);
        assert_eq!(info.expected_size(), 100);

        let info = MultiPartInfo::new(5, 10, 400001, 500000, 500000);
        assert_eq!(info.expected_size(), 100000);
    }
}
