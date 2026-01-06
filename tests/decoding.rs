//! Integration tests for decoding

use yenc::{YencError, decode};

#[test]
fn test_decode_simple() {
    let input = b"=ybegin line=128 size=5 name=test.bin\n\
                  ABCDE\n\
                  =yend size=5\n";

    let mut output = Vec::new();
    let (header, trailer, size) = decode(&input[..], &mut output).unwrap();

    assert_eq!(header.name, "test.bin");
    assert_eq!(header.size, 5);
    assert_eq!(size, 5);
    assert_eq!(output, vec![23, 24, 25, 26, 27]);
    assert!(trailer.is_some());
}

#[test]
fn test_decode_with_escapes() {
    // All escaped characters
    let input = b"=ybegin line=128 size=7 name=test.bin\n\
                  =@=I=J=M=`=n=}\n\
                  =yend size=7\n";

    let mut output = Vec::new();
    decode(&input[..], &mut output).unwrap();

    assert_eq!(output.len(), 7);
    assert_eq!(output, vec![0xD6, 0xDF, 0xE0, 0xE3, 0xF6, 0x04, 0x13]);
}

#[test]
fn test_decode_missing_header() {
    let input = b"VWXYZ\n=yend size=5\n";

    let mut output = Vec::new();
    let result = decode(&input[..], &mut output);

    assert!(matches!(result, Err(YencError::InvalidHeader(_))));
}
