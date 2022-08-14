use colored::Colorize;
use regex::RegexBuilder;
use std::{error::Error, io::stdin};

pub fn grep(args: Args) -> Result<(), Box<dyn Error>> {
    // TODO decide between stdin or filenames
    if args.filenames.is_empty() {
        return from_stdin(args);
    } else {
        println!("use filenames");
    }
    Ok(())
}

fn from_stdin(args: Args) -> Result<(), Box<dyn Error>> {
    let lines = stdin().lines();
    let re = match RegexBuilder::new(&args.query)
        .case_insensitive(args.insensitive)
        .build()
    {
        Ok(regex) => regex,
        Err(e) => panic!("Error parsing given regexp: {}", e),
    };

    for line in lines {
        let l = &line.unwrap();
        if let Some(result) = re.find(l) {
            let found = result.as_str();
            let line_to_print = l.replace(found, &found.red().to_string());
            println!("{}", line_to_print);
        }
    }
    Ok(())
}

#[derive(clap::Parser, Debug)]
#[clap(name = "minigrep")]
pub struct Args {
    #[clap(short, long, value_parser)]
    insensitive: bool,

    #[clap(short, long, value_parser)]
    query: String,

    #[clap(value_parser)]
    filenames: Vec<String>,
}

#[cfg(test)]
mod tests {
    // use super::*;
}
