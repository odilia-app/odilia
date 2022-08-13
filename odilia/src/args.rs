//! Command-line argument parsing with [`clap`].

use clap::Parser;

/// Command-line arguments
#[derive(Parser, Debug)]
#[clap(about, version)]
pub struct Args {}

/// Parse command-line arguments from [`std::env::args_os`]. This function exists so we don't have
/// to import [`clap::Parser`] in main.rs.
pub fn parse() -> Args {
    Args::parse()
}
