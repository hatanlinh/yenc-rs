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

#[test]
fn test_roundtrip_with_crc_corruption() {
    // Test that corrupted CRC is detected
    let original = b"Test data for CRC validation";

    let mut encoded = Vec::new();
    yenc::encode(&original[..], &mut encoded, "test.bin").unwrap();

    // Find and corrupt the CRC value in the trailer line
    // Look for "crc32=" and replace the hex value with "ffffffff"
    let trailer_start = encoded.windows(6).position(|w| w == b"crc32=").unwrap();
    let crc_value_start = trailer_start + 6;
    // Replace the 8-byte hex CRC value with "ffffffff"
    encoded[crc_value_start..crc_value_start + 8].copy_from_slice(b"ffffffff");

    let mut decoded = Vec::new();
    let result = yenc::decode(&encoded[..], &mut decoded);

    // Should fail with CRC mismatch
    assert!(result.is_err());
    match result.unwrap_err() {
        yenc::YencError::CrcMismatch { .. } => {
            // Expected error
        }
        other => panic!("Expected CrcMismatch, got {:?}", other),
    }
}

#[test]
fn test_roundtrip_no_crc_validation() {
    // Test that corrupted CRC is ignored when validation is disabled
    let original = b"Test data";

    let mut encoded = Vec::new();
    yenc::encode(&original[..], &mut encoded, "test.bin").unwrap();

    // Find and corrupt the CRC value in the trailer line
    let trailer_start = encoded.windows(6).position(|w| w == b"crc32=").unwrap();
    let crc_value_start = trailer_start + 6;
    // Replace the 8-byte hex CRC value with "ffffffff"
    encoded[crc_value_start..crc_value_start + 8].copy_from_slice(b"ffffffff");

    let mut decoded = Vec::new();
    let result = yenc::Decoder::new()
        .no_crc_check()
        .decode(&encoded[..], &mut decoded);

    // Should succeed even with wrong CRC when validation is disabled
    assert!(result.is_ok());
    assert_eq!(decoded, original);
}
