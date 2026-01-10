//! Multi-part encoding/decoding tests

use yenc;

#[test]
fn test_multipart_decode_single_part() {
    // Decode a single part from a multi-part file
    let input = b"=ybegin part=1 total=3 line=128 size=15 name=test.bin\n\
                  =ypart begin=1 end=5\n\
                  *+,-=n\n\
                  =yend size=5 part=1 pcrc32=515ad3cc\n";
    let mut output = Vec::new();

    let (header, part, trailer, size) = yenc::decode(&input[..], &mut output).unwrap();

    // Verify header
    assert_eq!(header.name, "test.bin");
    assert_eq!(header.size, 15); // Total file size
    assert_eq!(header.part, Some(1));
    assert_eq!(header.total, Some(3));

    // Verify part info
    let part = part.expect("Should have part info for multi-part file");
    assert_eq!(part.begin, 1);
    assert_eq!(part.end, 5);
    assert_eq!(part.size(), 5);

    // Verify trailer
    let trailer = trailer.expect("Should have trailer");
    assert_eq!(trailer.size, 5); // Part size
    assert_eq!(trailer.part, Some(1));
    assert_eq!(trailer.pcrc32, Some(0x515ad3cc));

    // Verify decoded data
    assert_eq!(size, 5);
    assert_eq!(output, vec![0, 1, 2, 3, 4]);
}

#[test]
fn test_multipart_validation_errors() {
    // Test that part size validation works
    let input = b"=ybegin part=1 total=2 line=128 size=100 name=test.bin\n\
                  =ypart begin=1 end=10\n\
                  *+,-=n\n\
                  =yend size=5 part=1 pcrc32=515ad3cc\n"; // Wrong: should be size=10

    let mut output = Vec::new();
    let result = yenc::decode(&input[..], &mut output);

    assert!(result.is_err());
    match result.unwrap_err() {
        yenc::YencError::InvalidData(msg) => {
            assert!(msg.contains("Part size mismatch"));
        }
        other => panic!("Expected InvalidData, got {:?}", other),
    }
}

#[test]
fn test_multipart_large_byte_offsets() {
    // Test with realistic large file offsets (e.g., part 100 of a large file)
    let input = b"=ybegin part=100 total=200 line=128 size=10485760 name=large.bin\n\
                  =ypart begin=5242881 end=5294080\n\
                  *+,-=n\n\
                  =yend size=51200 part=100 pcrc32=ffffffff\n";

    let mut output = Vec::new();
    let (header, part, trailer, _) = yenc::Decoder::new()
        .no_crc_check()
        .decode(&input[..], &mut output)
        .unwrap();

    assert_eq!(header.size, 10485760); // 10 MB file
    assert_eq!(header.part, Some(100));
    assert_eq!(header.total, Some(200));

    let part = part.unwrap();
    assert_eq!(part.begin, 5242881);
    assert_eq!(part.end, 5294080);
    assert_eq!(part.size(), 51200); // 50 KB part

    let trailer = trailer.unwrap();
    assert_eq!(trailer.size, 51200);
}

#[test]
fn test_multipart_with_real_crc_validation() {
    // Test multi-part decoding with actual CRC32 validation
    // This uses real encoded data with correct CRC values

    // First, encode the parts to get correct CRC values
    let data_part1 = vec![0u8, 1, 2, 3, 4];
    let data_part2 = vec![5u8, 6, 7, 8, 9];

    let mut encoded_part1 = Vec::new();
    let mut encoded_part2 = Vec::new();

    yenc::encode(&data_part1[..], &mut encoded_part1, "temp.bin").unwrap();
    yenc::encode(&data_part2[..], &mut encoded_part2, "temp.bin").unwrap();

    // Extract CRC values from encoded output
    let enc1_str = String::from_utf8_lossy(&encoded_part1);
    let enc2_str = String::from_utf8_lossy(&encoded_part2);

    let crc1 = enc1_str.split("crc32=").nth(1).unwrap().split_whitespace().next().unwrap();
    let crc2 = enc2_str.split("crc32=").nth(1).unwrap().split_whitespace().next().unwrap();

    // Now create multi-part format with correct CRCs
    let part1 = format!(
        "=ybegin part=1 total=2 line=128 size=10 name=real.bin\n\
         =ypart begin=1 end=5\n\
         *+,-=n\n\
         =yend size=5 part=1 pcrc32={}\n",
        crc1
    );

    let part2 = format!(
        "=ybegin part=2 total=2 line=128 size=10 name=real.bin\n\
         =ypart begin=6 end=10\n\
         /0123\n\
         =yend size=5 part=2 pcrc32={}\n",
        crc2
    );

    let mut decoded_part1 = Vec::new();
    let mut decoded_part2 = Vec::new();

    // Decode with CRC validation enabled (default)
    let (header1, part_info1, trailer1, _) = yenc::decode(part1.as_bytes(), &mut decoded_part1).unwrap();
    let (header2, part_info2, trailer2, _) = yenc::decode(part2.as_bytes(), &mut decoded_part2).unwrap();

    // Verify both parts
    assert_eq!(header1.name, "real.bin");
    assert_eq!(header1.size, 10);
    assert_eq!(header1.part, Some(1));
    assert_eq!(header2.part, Some(2));

    // Verify part info
    let p1 = part_info1.unwrap();
    let p2 = part_info2.unwrap();
    assert_eq!(p1.begin, 1);
    assert_eq!(p1.end, 5);
    assert_eq!(p2.begin, 6);
    assert_eq!(p2.end, 10);

    // Verify CRC values are present
    let t1 = trailer1.unwrap();
    let t2 = trailer2.unwrap();
    assert!(t1.pcrc32.is_some());
    assert!(t2.pcrc32.is_some());

    // Verify decoded data
    assert_eq!(decoded_part1, vec![0, 1, 2, 3, 4]);
    assert_eq!(decoded_part2, vec![5, 6, 7, 8, 9]);

    // Assemble full file
    let mut full_file = Vec::new();
    full_file.extend_from_slice(&decoded_part1);
    full_file.extend_from_slice(&decoded_part2);
    assert_eq!(full_file, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
}
