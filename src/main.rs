use clap::Parser;
use minigrep::Args;
use std::process;

fn main() {
    let args = Args::parse();

    if let Err(e) = minigrep::grep(args) {
        eprintln!("Application Error: {}", e);
        process::exit(1);
    }
}
