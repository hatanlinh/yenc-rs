//! yEnc decoding functionality

use std::io::{BufRead, BufReader, Read, Write};

use crate::error::{Result, YencError};
use crate::header::{YencHeader, YencTrailer};

const ESCAPE_CHAR: u8 = b'=';
const OFFSET: u8 = 42;

/// Decode a single yEnc-encoded byte
#[inline]
fn decode_byte(byte: u8) -> u8 {
    byte.wrapping_sub(OFFSET)
}

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

/// Decode yEnc data from a reader and write to a writer
///
/// # Arguments
/// * `reader` - Input reader containing yEnc-encoded data
/// * `writer` - Output writer for decoded data
///
/// # Returns
/// A tuple of (header, trailer, bytes_written)
pub fn decode<R: Read, W: Write>(
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

    let mut bytes_written = 0;
    let mut escaped = false;
    loop {
        let trimmed = trim_bytes(&line);
        if trimmed.starts_with(b"=yend ") {
            if let Ok(trailer_text) = std::str::from_utf8(trimmed) {
                let trailer = YencTrailer::parse(trailer_text)?;
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
                decode_byte(byte.wrapping_sub(64))
            } else {
                decode_byte(byte)
            };

            writer.write_all(&[decoded])?;
            bytes_written += 1;
        }

        line.clear();
        let bytes_read = buf_reader.read_until(b'\n', &mut line)?;
        if bytes_read == 0 {
            break;
        }
    }

    Ok((header, None, bytes_written))
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
}
