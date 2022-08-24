use colored::Colorize;
use regex::{Regex, RegexBuilder};
use std::fs::File;
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::{error::Error, io::stdin, io::stdout};

pub fn grep(args: Args) -> Result<(), Box<dyn Error>> {
    let re = create_regex(&args);
    if args.filenames.is_empty() {
        let io = Io {
            input: stdin().lock(),
            output: &mut stdout(),
        };
        from_stdin(io, args, &re)?;
        return Ok(());
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

struct Io<'a, I: BufRead, O: Write> {
    input: I,
    output: &'a mut O,
}

// use trait for stdin/stdout
// https://stackoverflow.com/questions/28370126/how-can-i-test-stdin-and-stdout
fn from_stdin<I: BufRead, O: Write>(
    mut io: Io<I, O>,
    args: Args,
    re: &Regex,
) -> Result<(), Box<dyn Error>> {
    let lines = io.input.lines();
    for line in lines.enumerate() {
        if let (linenumber, Ok(l)) = line {
            handle_line(&mut io.output, &l, linenumber, &re, &args);
        }
    }
    Ok(())
}

fn from_files(args: Args, re: &Regex) -> Result<(), Box<dyn Error>> {
    let mut output = stdout();
    for filename in &args.filenames {
        if args.names {
            println!("{}", filename.purple());
        }
        if let Ok(lines) = read_lines(filename) {
            for line in lines.enumerate() {
                if let (linenumber, Ok(l)) = line {
                    handle_line(&mut output, &l, linenumber, &re, &args)
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

fn handle_line<O: Write>(output: &mut O, line: &str, linenumber: usize, re: &Regex, args: &Args) {
    let matches = re.find_iter(line);
    let mut offset = 0;
    let mut found = false;
    for (i, m) in matches.enumerate() {
        found = true; // marker that we have a match in this line

        // print all before match
        if i == 0 && args.linenumber {
            write!(output, "{}:", linenumber).expect("Error writing output stream");
        }
        write!(output, "{}", &line[offset..m.start()]).expect("Error writing output stream");

        // print match
        if args.color {
            write!(output, "{}", m.as_str().bold().red()).expect("Error writing output stream");
        } else {
            write!(output, "{}", m.as_str()).expect("Error writing output stream");
        }

        // advance position to after match
        offset = m.end();
    }

    // only print line if there was a match
    if found {
        // print all after last match
        write!(output, "{}\n", &line[offset..]).expect("Error writing output stream");
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
    }

    #[test]
    fn test_from_stream() {
        let args = Args {
            insensitive: true,
            query: "foo".to_string(),
            filenames: vec![],
            names: false,
            linenumber: false,
            color: false,
        };

        let input = b"foo bar
bar baz
Foo baz";
        let mut v = Vec::new();
        let io = Io {
            input: &input[..],
            output: &mut v,
        };
        let re = create_regex(&args);
        from_stdin(io, args, &re).unwrap();

        let actual = String::from_utf8(v).expect("Not UTF-8");
        let expected = "foo bar
Foo baz
";
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_stream_end() {
        let args = Args {
            insensitive: true,
            query: "foo$".to_string(),
            filenames: vec![],
            names: false,
            linenumber: false,
            color: false,
        };

        let input = b"foo bar
bar baz
bar baz FOO
foo baz";
        let mut v = Vec::new();
        let io = Io {
            input: &input[..],
            output: &mut v,
        };
        let re = create_regex(&args);
        from_stdin(io, args, &re).unwrap();

        let actual = String::from_utf8(v).expect("Not UTF-8");
        let expected = "bar baz FOO
";
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_from_stream_end_color() {
        let args = Args {
            insensitive: true,
            query: "foo$".to_string(),
            filenames: vec![],
            names: false,
            linenumber: false,
            color: true,
        };

        let input = b"foo bar
bar baz
bar baz FOO
foo baz";
        let mut v = Vec::new();
        let io = Io {
            input: &input[..],
            output: &mut v,
        };
        let re = create_regex(&args);
        from_stdin(io, args, &re).unwrap();

        let actual = String::from_utf8(v).expect("Not UTF-8");
        let expected = "bar baz \u{1b}[1;31mFOO\u{1b}[0m
";
        assert_eq!(expected, actual);
    }
}
