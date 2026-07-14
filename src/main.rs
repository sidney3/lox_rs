use lox_rs::{Config, run};

use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(name = "lox", version, about = "A Lox Runtime")]
struct Cli {
  /// Path to the .lox source file (omit for REPL)
  #[arg(short, long)]
  script: PathBuf,

  #[arg(long, default_value_t = false)]
  disasm: bool,
}

fn main() {
  env_logger::init();

  let cli = Cli::parse();

  let cfg = Config {
    script: cli.script,
    disasm: cli.disasm,
  };

  run(cfg)
}
