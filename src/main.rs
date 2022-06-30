use clap::Parser;
use crate::cli::Args;
use crate::token::Tokens;

mod cli;
mod token;

fn main() {
	let args = Args::parse();
	let mut tokens = Tokens::new(args.filename.as_str()).unwrap();
	tokens.parse().unwrap();
}
