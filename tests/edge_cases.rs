//! Edge case tests

use yenc;

#[test]
fn test_critical_characters() {
    // Test characters that need escaping
    let original = vec![0x00, 0x0A, 0x0D, b'=', 0x09];

    let mut encoded = Vec::new();
    yenc::encode(&original[..], &mut encoded, "critical.bin").unwrap();

    let mut decoded = Vec::new();
    yenc::decode(&encoded[..], &mut decoded).unwrap();

    assert_eq!(decoded, original);
}

#[test]
fn test_long_filename() {
    let original = b"test";
    let long_name = "a".repeat(200) + ".bin";

    let mut encoded = Vec::new();
    yenc::encode(&original[..], &mut encoded, &long_name).unwrap();

    let mut decoded = Vec::new();
    let (header, _, _) = yenc::decode(&encoded[..], &mut decoded).unwrap();

    assert_eq!(header.name, long_name);
}
