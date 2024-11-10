use std::path::PathBuf;

use clap::Parser;

/// Clap based CLI argument parsing.
///
/// Currently supports only reading from a file.
#[derive(Debug, Parser)]
#[command(version, author)]
pub struct Cli {
    /// The file to read the IR signals from.
    #[clap(short, long)]
    pub file: PathBuf,
    #[clap(short, long)]
    pub output_dir: PathBuf,
}
