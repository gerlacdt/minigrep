use colored::Colorize;
use regex::{Regex, RegexBuilder};
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::{error::Error, io::stdin};

pub fn grep(args: Args) -> Result<(), Box<dyn Error>> {
    let re = create_regex(&args);
    if args.filenames.is_empty() {
        return from_stdin(args, &re);
    }
    from_files(args, &re)
}

fn create_regex(args: &Args) -> Regex {
    match RegexBuilder::new(&args.query)
        .case_insensitive(args.insensitive)
        .build()
    {
        Ok(regex) => regex,
        Err(e) => panic!("Error parsing given regexp: {}", e),
    }
}

// use trait for stdin/stdout
// https://stackoverflow.com/questions/28370126/how-can-i-test-stdin-and-stdout
fn from_stdin(args: Args, re: &Regex) -> Result<(), Box<dyn Error>> {
    let lines = stdin().lock().lines();
    for line in lines.enumerate() {
        if let (linenumber, Ok(l)) = line {
            handle_line(&l, linenumber, &re, &args);
        }
    }
    Ok(())
}

fn from_files(args: Args, re: &Regex) -> Result<(), Box<dyn Error>> {
    for filename in &args.filenames {
        if args.names {
            println!("{}:", filename);
        }
        if let Ok(lines) = read_lines(filename) {
            for line in lines.enumerate() {
                if let (linenumber, Ok(l)) = line {
                    handle_line(&l, linenumber, &re, &args)
                }
            }
        }
        println!();
    }
    Ok(())
}

// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn handle_line(line: &str, linenumber: usize, re: &Regex, args: &Args) {
    if let Some(result) = re.find(line) {
        let found = result.as_str();
        let line_to_print = line.replace(found, &found.red().bold().to_string());
        if args.linenumber {
            println!("{}:{}", linenumber, line_to_print);
        } else {
            println!("{}", line_to_print);
        }
    }
}

#[derive(clap::Parser, Debug)]
#[clap(name = "minigrep")]
pub struct Args {
    #[clap(short, long, value_parser)]
    insensitive: bool,

    #[clap(short, long, value_parser)]
    query: String,

    #[clap(long, value_parser)]
    names: bool,

    #[clap(long, value_parser)]
    linenumber: bool,

    #[clap(value_parser)]
    filenames: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_files_with_names() {
        let args = Args {
            insensitive: true,
            query: "foo".to_string(),
            filenames: vec![
                "test_files/poem.txt".to_string(),
                "test_files/foo.txt".to_string(),
            ],
            names: true,
            linenumber: true,
        };

        let _ = grep(args);
    }
}
