use std::path::PathBuf;

use clap::Parser;
use lox_rs::{Config, run};

#[derive(Parser)]
#[command(name = "lox", version, about = "A Lox Runtime")]
struct Cli {
  /// Path to the .lox source file (omit for REPL)
  #[arg(short, long)]
  script: PathBuf,
}

fn main() {
  env_logger::init();

  let cli = Cli::parse();

  let cfg = Config { script: cli.script };

  run(cfg).unwrap();
}
