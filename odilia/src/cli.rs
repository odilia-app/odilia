use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, author)]
pub struct CliArgs {
	/// Specify a custom Odilia configuration path
	#[arg(short, long, value_name = "FILE")]
	pub config: Option<PathBuf>,
}
