//! Integration tests for encoding

use yenc::encode;

#[test]
fn test_encode_simple() {
    let input = vec![0u8, 1, 2, 3, 4];
    let mut output = Vec::new();

    let size = encode(&input[..], &mut output, "test.bin").unwrap();

    assert_eq!(size, 5);

    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("=ybegin"));
    assert!(output_str.contains("name=test.bin"));
    assert!(output_str.contains("size=5"));
    assert!(output_str.contains("=yend"));

    // TODO: Check encoded data
}

#[test]
fn test_encode_binary_data() {
    // All possible byte values
    let input: Vec<u8> = (0..=255).collect();
    let mut output = Vec::new();

    encode(&input[..], &mut output, "binary.bin").unwrap();

    // Should have header, data, and trailer
    assert!(output.len() > input.len());

    // TODO: Validate more
}

#[test]
fn test_encode_empty() {
    let input: Vec<u8> = Vec::new();
    let mut output = Vec::new();

    let size = encode(&input[..], &mut output, "empty.bin").unwrap();

    assert_eq!(size, 0);
}
