use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(version, about, author)]
pub struct Args {
	/// Specify a custom Odilia configuration path
	#[arg(short, long, value_name = "FILE")]
	pub config: Option<PathBuf>,
}
