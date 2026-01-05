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
    let input = b"=ybegin line=128 size=3 name=test.bin\n\
                  =j=m=o\n\
                  =yend size=3\n";

    let mut output = Vec::new();
    decode(&input[..], &mut output).unwrap();

    assert_eq!(output.len(), 3);
    assert_eq!(output, vec![0, 3, 5]);
}

#[test]
fn test_decode_missing_header() {
    let input = b"ABCDE\n=yend size=5\n";

    let mut output = Vec::new();
    let result = decode(&input[..], &mut output);

    assert!(matches!(result, Err(YencError::InvalidHeader(_))));
}
