use std::path::PathBuf;

#[derive(Default)]
pub struct Args {
	pub config: Option<PathBuf>,
}

impl Args {
	pub fn from_cli_args() -> Result<Self, lexopt::Error> {
		use lexopt::prelude::*;
		let mut args = Args::default();
		let mut parser = lexopt::Parser::from_env();
		while let Some(arg) = parser.next()? {
			match arg {
				Short('c') | Long("config") => {
					args.config = Some(parser.value()?.parse::<PathBuf>()?);
				}
				_ => {}
			}
		}
		Ok(args)
	}
}
