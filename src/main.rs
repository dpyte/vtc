use clap::Parser;
use vtc::cli::Args;
use vtc::serializer::parser::RParser;
use vtc::serializer::token::Tokens;


fn main() {
	let args = Args::parse();
	let mut tokens = Tokens::new(args.filename.as_str()).unwrap();
	tokens.tokenize().unwrap();

	let mut p_obj = RParser::new(tokens);
	p_obj.generate_ast();
}
