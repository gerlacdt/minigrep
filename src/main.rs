use clap::Parser;
use minigrep::Args;
use std::{io::stdout, process};

fn main() {
    let args = Args::parse();
    let mut writer = stdout().lock();
    if let Err(e) = minigrep::grep(args, &mut writer) {
        eprintln!("Application Error: {}", e);
        process::exit(1);
    }
}
