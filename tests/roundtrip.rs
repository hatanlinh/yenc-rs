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

#[test]
fn test_roundtrip_multipart_encode_decode() {
    // Encode a file in two parts and decode them back
    let full_data = vec![0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9];

    // Part 1: bytes 1-5 (indices 0-4)
    let part1_data = &full_data[0..5];
    let part1_info = yenc::MultiPartInfo::new(1, 2, 1, 5, 10);

    // Part 2: bytes 6-10 (indices 5-9)
    let part2_data = &full_data[5..10];
    let part2_info = yenc::MultiPartInfo::new(2, 2, 6, 10, 10);

    let mut encoded_part1 = Vec::new();
    let mut encoded_part2 = Vec::new();

    yenc::Encoder::new()
        .encode_part(&part1_data[..], &mut encoded_part1, "data.bin", &part1_info)
        .unwrap();
    yenc::Encoder::new()
        .encode_part(&part2_data[..], &mut encoded_part2, "data.bin", &part2_info)
        .unwrap();

    // Decode both parts
    let mut decoded_part1 = Vec::new();
    let mut decoded_part2 = Vec::new();

    let (header1, p1_info, trailer1, _) = yenc::decode(&encoded_part1[..], &mut decoded_part1).unwrap();
    let (header2, p2_info, trailer2, _) = yenc::decode(&encoded_part2[..], &mut decoded_part2).unwrap();

    // Verify headers
    assert_eq!(header1.name, "data.bin");
    assert_eq!(header1.size, 10); // Full file size
    assert_eq!(header1.part, Some(1));
    assert_eq!(header1.total, Some(2));
    assert_eq!(header2.part, Some(2));

    // Verify part information
    let p1 = p1_info.unwrap();
    let p2 = p2_info.unwrap();
    assert_eq!(p1.begin, 1);
    assert_eq!(p1.end, 5);
    assert_eq!(p2.begin, 6);
    assert_eq!(p2.end, 10);

    // Verify trailers
    let t1 = trailer1.unwrap();
    let t2 = trailer2.unwrap();
    assert_eq!(t1.size, 5); // Part size
    assert_eq!(t2.size, 5);
    assert_eq!(t1.part, Some(1));
    assert_eq!(t2.part, Some(2));
    assert!(t1.pcrc32.is_some()); // Part CRC computed
    assert!(t2.pcrc32.is_some());

    // Verify decoded data
    assert_eq!(decoded_part1, vec![0, 1, 2, 3, 4]);
    assert_eq!(decoded_part2, vec![5, 6, 7, 8, 9]);

    // Assemble and verify full file
    let mut reassembled = Vec::new();
    reassembled.extend_from_slice(&decoded_part1);
    reassembled.extend_from_slice(&decoded_part2);

    assert_eq!(reassembled, full_data);
}

#[test]
fn test_roundtrip_multipart_with_full_crc() {
    // Test multi-part with full file CRC in last part
    let full_data = vec![0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9];

    // First encode the full file to get the full CRC
    let mut full_encoded = Vec::new();
    yenc::encode(&full_data[..], &mut full_encoded, "temp.bin").unwrap();

    let full_enc_str = String::from_utf8_lossy(&full_encoded);
    let full_crc_str = full_enc_str.split("crc32=").nth(1).unwrap().split_whitespace().next().unwrap();
    let full_crc = u32::from_str_radix(full_crc_str, 16).unwrap();

    // Now encode as multi-part with full CRC in last part
    let part1_data = &full_data[0..5];
    let part1_info = yenc::MultiPartInfo::new(1, 2, 1, 5, 10);

    let part2_data = &full_data[5..10];
    let part2_info = yenc::MultiPartInfo::new(2, 2, 6, 10, 10)
        .with_full_crc(full_crc); // Include full file CRC in last part

    let mut encoded_part1 = Vec::new();
    let mut encoded_part2 = Vec::new();

    yenc::encode_part(&part1_data[..], &mut encoded_part1, "data.bin", &part1_info).unwrap();
    yenc::encode_part(&part2_data[..], &mut encoded_part2, "data.bin", &part2_info).unwrap();

    // Decode and verify
    let mut decoded_part1 = Vec::new();
    let mut decoded_part2 = Vec::new();

    yenc::decode(&encoded_part1[..], &mut decoded_part1).unwrap();
    let (_, _, trailer2, _) = yenc::decode(&encoded_part2[..], &mut decoded_part2).unwrap();

    // Verify last part has both pcrc32 and crc32
    let t2 = trailer2.unwrap();
    assert!(t2.pcrc32.is_some()); // Part CRC
    assert_eq!(t2.crc32, Some(full_crc)); // Full file CRC

    // Assemble and verify
    let mut reassembled = Vec::new();
    reassembled.extend_from_slice(&decoded_part1);
    reassembled.extend_from_slice(&decoded_part2);
    assert_eq!(reassembled, full_data);
}
