
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
	#[clap(short, long, value_parser)]
	pub filename: String,
}
