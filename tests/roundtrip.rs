//! Roundtrip tests (encode then decode)

use yenc;

#[test]
fn test_roundtrip_text() {
    let original = b"The quick brown fox jumps over the lazy dog";

    let mut encoded = Vec::new();
    yenc::encode(&original[..], &mut encoded, "test.txt").unwrap();

    let mut decoded = Vec::new();
    yenc::decode(&encoded[..], &mut decoded).unwrap();

    assert_eq!(decoded, original);
}

#[test]
fn test_roundtrip_binary() {
    // All byte values
    let original: Vec<u8> = (0..=255).collect();

    let mut encoded = Vec::new();
    yenc::encode(&original[..], &mut encoded, "binary.bin").unwrap();

    let mut decoded = Vec::new();
    yenc::decode(&encoded[..], &mut decoded).unwrap();

    assert_eq!(decoded, original);
}

#[test]
fn test_roundtrip_random_data() {
    // Pseudo-random data
    let original: Vec<u8> = (0..1000).map(|i| (i * 7 + 13) as u8).collect();

    let mut encoded = Vec::new();
    yenc::encode(&original[..], &mut encoded, "random.bin").unwrap();

    let mut decoded = Vec::new();
    yenc::decode(&encoded[..], &mut decoded).unwrap();

    assert_eq!(decoded, original);
}
