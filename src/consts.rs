//! Common constants and utilities for yEnc encoding/decoding

/// Offset value for yEnc encoding
pub(crate) const OFFSET: u8 = 42;

/// Offset value for escaping chars
pub(crate) const ESCAPE_OFFSET: u8 = 64;

/// The escape character used in yEnc encoding
pub(crate) const ESCAPE_CHAR: u8 = b'=';

/// Default line length for encoded output
pub(crate) const LINE_LENGTH: usize = 128;

/// Characters that are valid to escape according to yEnc spec
pub(crate) const ESCAPING_CHARS: [u8; 7] = [
    0x00, // NULL
    0x09, // TAB
    0x0A, // LF
    0x0D, // CR
    0x20, // SPACE
    0x2E, // DOT
    0x3D, // EQUAL - escape character itself
];
