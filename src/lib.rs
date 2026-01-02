use std::{
    fs::OpenOptions,
    io::{BufRead, BufReader},
    path::PathBuf,
};

fn parse_header_line(_line: &str) -> bool {
    // TODO: Implement parse header line
    return true;
}

fn decode_line(_line: &str) -> bool {
    // TODO: Implement decode line
    return true;
}

pub fn decode(input: PathBuf, output: PathBuf, debug_mode: bool) -> bool {
    if debug_mode {
        println!(
            "Decoding from {} to {}",
            input.to_string_lossy(),
            output.to_string_lossy()
        )
    }

    let input_file = OpenOptions::new()
        .read(true)
        .open(input)
        .unwrap_or_else(|_| panic!("Invalid input!"));
    let input_reader = BufReader::new(input_file);

    let mut yenc_start_found = false;
    for line in input_reader.lines() {
        let line = line.unwrap();
        if line.starts_with("=ybegin ") {
            yenc_start_found = true;
            parse_header_line(&line);
        } else if yenc_start_found {
            decode_line(&&line);
        }
    }

    true
}

pub fn encode(input: PathBuf, output: PathBuf, debug_mode: bool) -> bool {
    if debug_mode {
        println!(
            "Decoding from {} to {}",
            input.to_string_lossy(),
            output.to_string_lossy()
        )
    }

    true
}
