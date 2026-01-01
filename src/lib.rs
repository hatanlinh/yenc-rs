use std::path::PathBuf;

pub fn decode(input: PathBuf, output: PathBuf, debug_mode: bool) -> bool {
    if debug_mode {
        println!(
            "Decoding from {} to {}",
            input.to_string_lossy(),
            output.to_string_lossy()
        )
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
