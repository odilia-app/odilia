use clap::Parser;

#[derive(Parser, Debug)]
#[clap(about, version)]
pub struct Args {}

pub fn parse() -> Args {
    Args::parse()
}
