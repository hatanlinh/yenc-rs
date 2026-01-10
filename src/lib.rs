//! # yenc
//!
//! A SIMD-accelerated Rust implementation of the yEnc binary encoding format.
//!
//! ## Example
//!
//! ```rust
//! use yenc::{decode, encode};
//!
//! // Encoding
//! let input = b"Hello, World!";
//! let mut encoded = Vec::new();
//! yenc::encode(&input[..], &mut encoded, "hello.txt").unwrap();
//!
//! // Decoding
//! let mut decoded = Vec::new();
//! let (header, _, _, _) = yenc::decode(&encoded[..], &mut decoded).unwrap();
//! assert_eq!(decoded, b"Hello, World!");
//! assert_eq!(header.name, "hello.txt");
//! ```
//!
//! ## Advanced Usage
//!
//! ```rust
//! use yenc::{Decoder, Encoder};
//!
//! // Custom decoder options
//! let mut output = Vec::new();
//! let input = b"=ybegin line=128 size=5 name=test.bin\nKLMNO\n=yend size=5\n";
//!
//! let (_header, _part, _trailer, _size) = Decoder::new()
//!     .strict()
//!     .no_crc_check()
//!     .decode(&input[..], &mut output)
//!     .unwrap();
//!
//! // Custom encoder options
//! let data = b"Hello";
//! let mut encoded = Vec::new();
//! Encoder::new()
//!     .line_length(64)
//!     .no_crc()
//!     .encode(&data[..], &mut encoded, "file.bin")
//!     .unwrap();
//! ```

mod consts;
mod decode;
mod encode;
pub mod error;
pub mod header;

pub use decode::{Decoder, decode};
pub use encode::{Encoder, encode};
pub use error::{Result, YencError};
pub use header::{YencHeader, YencPart, YencTrailer};

use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

/// Decode a yEnc file
///
/// Opens files and decodes yEnc data.
///
/// # Arguments
/// * `input_path` - Path to the yEnc-encoded file
/// * `output_path` - Path where decoded data will be written
///
/// # Returns
/// A tuple of (header, part, trailer, bytes_written)
/// - For single-part files: part will be None
/// - For multi-part files: part contains begin/end byte positions
pub fn decode_file<P: AsRef<Path>>(
    input_path: P,
    output_path: P,
) -> Result<(YencHeader, Option<YencPart>, Option<YencTrailer>, usize)> {
    let input = BufReader::new(File::open(input_path)?);
    let output = BufWriter::new(File::create(output_path)?);
    decode(input, output)
}

/// Encode a file to yEnc format
///
/// Opens files and encodes data to yEnc.
///
/// # Arguments
/// * `input_path` - Path to the file to encode
/// * `output_path` - Path where yEnc-encoded data will be written
/// * `filename` - Filename to use in the yEnc header (defaults to input filename)
///
/// # Returns
/// Number of bytes encoded
pub fn encode_file<P: AsRef<Path>>(
    input_path: P,
    output_path: P,
    filename: Option<&str>,
) -> Result<usize> {
    let input = BufReader::new(File::open(&input_path)?);
    let output = BufWriter::new(File::create(output_path)?);

    let name = filename.unwrap_or_else(|| {
        input_path
            .as_ref()
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file.bin")
    });

    encode(input, output, name)
}
