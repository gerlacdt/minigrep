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
fn from_stdin(_args: Args, re: &Regex) -> Result<(), Box<dyn Error>> {
    let lines = stdin().lock().lines();
    for line in lines {
        if let Ok(l) = line {
            handle_line(&l, &re);
        }
    }
    Ok(())
}

fn from_files(args: Args, re: &Regex) -> Result<(), Box<dyn Error>> {
    for filename in args.filenames {
        if let Ok(lines) = read_lines(filename) {
            for line in lines {
                if let Ok(l) = line {
                    handle_line(&l, &re)
                }
            }
        }
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

fn handle_line(line: &str, re: &Regex) {
    if let Some(result) = re.find(line) {
        let found = result.as_str();
        let line_to_print = line.replace(found, &found.red().bold().to_string());
        println!("{}", line_to_print);
    }
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
    use super::*;

    #[test]
    fn test_filenames() {
        let args = Args {
            insensitive: true,
            query: "foo".to_string(),
            filenames: vec![
                "test_files/poem.txt".to_string(),
                "test_files/foo.txt".to_string(),
            ],
        };

        let _ = grep(args);
    }
}
