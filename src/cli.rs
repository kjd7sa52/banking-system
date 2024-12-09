use clap::Parser;
use std::path::PathBuf;

/// Simple Banking System
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Provide log messages
    #[arg(short, long)]
    pub log: bool,

    /// Print database to stderr for debugging purposes
    #[arg(short, long)]
    pub printdb: bool,

    /// Input file
    #[arg()]
    pub transactions: PathBuf,
}
