use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "generate_parser", version, about = "SLR1 parser generator")]
struct Cli {
  /// path to the .lparse file
  #[arg(short, long)]
  grammar: PathBuf,

  /// path to the output file
  #[arg(short, long)]
  out: PathBuf,
}

pub fn main() {
  env_logger::init();

  let cli = Cli::parse();

  parse::generate_parser(&cli.grammar, &cli.out).expect("Failed to generate parser");
}
