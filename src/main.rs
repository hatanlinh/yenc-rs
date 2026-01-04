use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "yenc")]
#[command(author, version, about = "SIMD-accelerated yEnc encoder/decoder", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Command {
    /// Decode a yEnc-encoded file
    Decode {
        /// Input file (yEnc-encoded)
        #[arg(short, long, value_name = "FILE")]
        input: PathBuf,

        /// Output file (decoded binary)
        #[arg(short, long, value_name = "FILE")]
        output: PathBuf,
    },
    /// Encode a file to yEnc format
    Encode {
        /// Input file (binary)
        #[arg(short, long, value_name = "FILE")]
        input: PathBuf,

        /// Output file (yEnc-encoded)
        #[arg(short, long, value_name = "FILE")]
        output: PathBuf,

        /// Filename to use in yEnc header (defaults to input filename)
        #[arg(short, long, value_name = "NAME")]
        name: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Command::Decode { input, output } => {
            if cli.verbose {
                println!("Decoding: {} -> {}", input.display(), output.display());
            }

            match yenc::decode_file(&input, &output) {
                Ok((header, trailer, bytes)) => {
                    println!("> Decoded {} bytes", bytes);
                    if cli.verbose {
                        println!("  File: {}", header.name);
                        println!("  Size: {} bytes", header.size);
                        if let Some(t) = trailer {
                            if let Some(crc) = t.crc32 {
                                println!("  CRC32: {:#x}", crc);
                            }
                        }
                    }
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }
        Command::Encode {
            input,
            output,
            name,
        } => {
            if cli.verbose {
                println!("Encoding: {} -> {}", input.display(), output.display());
            }

            match yenc::encode_file(&input, &output, name.as_deref()) {
                Ok(bytes) => {
                    println!("> Encoded {} bytes", bytes);
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
