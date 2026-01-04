//! yEnc header and trailer parsing

use crate::error::{Result, YencError};

/// yEnc header
#[derive(Debug, Clone, PartialEq)]
pub struct YencHeader {
    pub name: String,
    pub size: usize,
    pub line_len: Option<usize>,
    pub part: Option<usize>,
    pub total: Option<usize>,
}

impl YencHeader {
    /// Parse a yEnc header line (e.g., "=ybegin line=128 size=123456 name=file.bin")
    pub fn parse(line: &str) -> Result<Self> {
        if !line.starts_with("=ybegin ") {
            return Err(YencError::InvalidHeader(
                "Header must start with '=ybegin'".to_string(),
            ));
        }

        let mut name = None;
        let mut size = None;
        let mut line_len = None;
        let mut part = None;
        let mut total = None;

        for token in line[8..].split_whitespace() {
            if let Some((key, value)) = token.split_once('=') {
                match key {
                    "name" => name = Some(value.to_string()),
                    "size" => size = value.parse().ok(),
                    "line" => line_len = value.parse().ok(),
                    "part" => part = value.parse().ok(),
                    "total" => total = value.parse().ok(),
                    _ => {} // Ignore unknown fields
                }
            }
        }

        Ok(YencHeader {
            name: name.ok_or_else(|| YencError::MissingField("name".to_string()))?,
            size: size.ok_or_else(|| YencError::MissingField("size".to_string()))?,
            line_len,
            part,
            total,
        })
    }
}

/// yEnc trailer
#[derive(Debug, Clone, PartialEq)]
pub struct YencTrailer {
    pub size: usize,
    pub part: Option<usize>,
    pub pcrc32: Option<u32>,
    pub crc32: Option<u32>,
}

impl YencTrailer {
    /// Parse a yEnc trailer line (e.g., "=yend size=123456 crc32=abcd1234")
    pub fn parse(line: &str) -> Result<Self> {
        if !line.starts_with("=yend ") {
            return Err(YencError::InvalidHeader(
                "Trailer must start with '=yend'".to_string(),
            ));
        }

        let mut size = None;
        let mut part = None;
        let mut pcrc32 = None;
        let mut crc32 = None;

        for token in line[6..].split_whitespace() {
            if let Some((key, value)) = token.split_once('=') {
                match key {
                    "size" => size = value.parse().ok(),
                    "part" => part = value.parse().ok(),
                    "pcrc32" => pcrc32 = u32::from_str_radix(value, 16).ok(),
                    "crc32" => crc32 = u32::from_str_radix(value, 16).ok(),
                    _ => {}
                }
            }
        }

        Ok(YencTrailer {
            size: size.ok_or_else(|| YencError::MissingField("size".to_string()))?,
            part,
            pcrc32,
            crc32,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_header() {
        let line = "=ybegin line=128 size=123456 name=testfile.bin";
        let header = YencHeader::parse(line).unwrap();
        assert_eq!(header.name, "testfile.bin");
        assert_eq!(header.size, 123456);
        assert_eq!(header.line_len, Some(128));
    }

    #[test]
    fn test_parse_trailer() {
        let line = "=yend size=123456 crc32=abcd1234";
        let trailer = YencTrailer::parse(line).unwrap();
        assert_eq!(trailer.size, 123456);
        assert_eq!(trailer.crc32, Some(0xabcd1234));
    }
}
