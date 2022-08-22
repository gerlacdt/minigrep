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
            println!("{}", filename.purple());
        }
        if let Ok(lines) = read_lines(filename) {
            for line in lines.enumerate() {
                if let (linenumber, Ok(l)) = line {
                    handle_line(&l, linenumber, &re, &args)
                }
            }
        }
        println!(); // newline delimiter for every file
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
    let matches = re.find_iter(line);
    let mut offset = 0;
    let mut found = false;
    for (i, m) in matches.enumerate() {
        found = true; // marker that we have a match in this line

        // print all before match
        if i == 0 && args.linenumber {
            print!("{}:", linenumber);
        }
        print!("{}", &line[offset..m.start()]);

        // print match
        if args.color {
            print!("{}", m.as_str().bold().red());
        } else {
            print!("{}", m.as_str());
        }

        // advance position to after match
        offset = m.end();
    }

    // only print line if there was a match
    if found {
        // print all after last match
        println!("{}", &line[offset..]);
    }
}

#[derive(clap::Parser, Debug)]
#[clap(name = "minigrep")]
pub struct Args {
    /// enable case insensitive search
    #[clap(short, long, value_parser)]
    insensitive: bool,

    /// regex to search for
    #[clap(short, long, value_parser)]
    query: String,

    /// enable showing filenames
    #[clap(short = 'H', long = "with-filenames", value_parser)]
    names: bool,

    /// enable show linenumbers
    #[clap(short = 'n', value_parser)]
    linenumber: bool,

    /// enable highlighting a match
    #[clap(short = 'c', long, value_parser)]
    color: bool,

    /// list of filenames to search in
    #[clap(value_parser)]
    filenames: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_files_with_names_with_color() {
        let args = Args {
            insensitive: true,
            query: "foo".to_string(),
            filenames: vec![
                "test_files/poem.txt".to_string(),
                "test_files/foo.txt".to_string(),
            ],
            names: true,
            linenumber: true,
            color: true,
        };

        let _ = grep(args);
    }

    #[test]
    fn test_files_with_names_no_color() {
        let args = Args {
            insensitive: true,
            query: "foo".to_string(),
            filenames: vec![
                "test_files/poem.txt".to_string(),
                "test_files/foo.txt".to_string(),
            ],
            names: true,
            linenumber: true,
            color: false,
        };

        let _ = grep(args);
    }

    #[test]
    fn test_correct_coloring_ending() {
        let args = Args {
            insensitive: true,
            query: "foo$".to_string(),
            filenames: vec!["test_files/poem.txt".to_string()],
            names: true,
            linenumber: true,
            color: true,
        };

        let _ = grep(args);
        println!("done");
    }

    #[test]
    fn test_correct_coloring_start() {
        let args = Args {
            insensitive: true,
            query: "^foo".to_string(),
            filenames: vec!["test_files/poem.txt".to_string()],
            names: true,
            linenumber: true,
            color: true,
        };

        let _ = grep(args);
        println!("done");
    }

    #[test]
    fn test_string_build() {
        let s = "foobarbaz";

        print!("{}", &s[0..0]);
        print!("{}", &s[0..3].bold().blue());
        print!("{}", &s[3..6]);
        print!("{}", &s[6..9].bold().green());

        println!();
    }
}
