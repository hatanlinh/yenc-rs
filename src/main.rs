use std::path::PathBuf;

use clap::{Parser, Subcommand};

use yenc::{decode, encode};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,

    /// Turn debugging information on
    #[arg(short, long)]
    debug: bool,
}

#[derive(Subcommand)]
enum Command {
    /// Decode
    Decode {
        /// Input path
        #[arg(short, long, required = true)]
        input: Option<PathBuf>,

        /// Output path
        #[arg(short, long, required = true)]
        output: Option<PathBuf>,
    },

    /// Encode
    Encode {
        /// Input path
        #[arg(short, long, required = true)]
        input: Option<PathBuf>,

        /// Output path
        #[arg(short, long, required = true)]
        output: Option<PathBuf>,
    },
}

fn main() {
    let args = Args::parse();

    if args.debug {
        println!("Debug mode is on");
    } else {
        println!("Debug mode is off");
    }

    match &args.command {
        Some(Command::Decode { input, output }) => {
            match decode(
                input.as_ref().unwrap().to_path_buf(),
                output.as_ref().unwrap().to_path_buf(),
                args.debug,
            ) {
                true => {
                    println!("Decoded successfully")
                }
                false => {
                    println!("Decode failed")
                }
            }
        }
        Some(Command::Encode { input, output }) => {
            match encode(
                input.as_ref().unwrap().to_path_buf(),
                output.as_ref().unwrap().to_path_buf(),
                args.debug,
            ) {
                true => {
                    println!("Encoded successfully")
                }
                false => {
                    println!("Encode failed")
                }
            }
        }
        None => {
            println!("No command specified!")
        }
    }
}
