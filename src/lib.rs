use colored::Colorize;
use regex::{Regex, RegexBuilder};
use std::fs::File;
use std::io::{self, BufRead, Write};
use std::{error::Error, io::stdin, io::stdout};
use walkdir::WalkDir;

pub fn grep<O: Write>(args: Args, writer: &mut O) -> Result<(), Box<dyn Error>> {
    let re = create_regex(&args);
    if args.filenames.is_empty() {
        let io = Io {
            input: stdin().lock(),
            output: &mut stdout().lock(),
        };
        from_stdin(io, args, &re)?;
        return Ok(());
    }
    from_files(args, &re, writer)
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
    io: Io<I, O>,
    args: Args,
    re: &Regex,
) -> Result<(), Box<dyn Error>> {
    let lines = io.input.lines();
    for line in lines.enumerate() {
        if let (mut linenumber, Ok(l)) = line {
            linenumber += 1;
            if let Some(output) = handle_line(&l, linenumber, &re, &args) {
                write!(io.output, "{}", output).unwrap();
            }
        }
    }
    Ok(())
}

fn from_files<O: Write>(args: Args, re: &Regex, writer: &mut O) -> Result<(), Box<dyn Error>> {
    if args.recursive {
        // do recursive search only for a single directory
        if args.filenames.len() != 1 {
            panic!("Recursive Search only works for a single directory");
        }
        let dir = &args.filenames[0];
        let walker = WalkDir::new(dir).into_iter();
        for entry in walker.into_iter().filter_map(|e| e.ok()) {
            if entry.path().is_file() {
                let filename = entry.path().to_str().expect("Invalid Path or Filename");
                handle_file(filename, &args, &re, writer);
            }
        }
        writeln!(writer, "").expect("ERROR: could not write to STDOUT "); // newline delimiter for every file
    } else {
        // do search for the given list of files
        for filename in &args.filenames {
            handle_file(filename, &args, &re, writer);
            writeln!(writer, "").expect("ERROR: could not write to STDOUT "); // newline delimiter for every file
        }
    }

    Ok(())
}

fn handle_file<O: Write>(filename: &str, args: &Args, re: &Regex, writer: &mut O) {
    let mut found = false;
    let file = File::open(filename).expect("ERROR, file cannot be opened");
    let lines = io::BufReader::new(file).lines();
    for line in lines.enumerate() {
        if let (mut linenumber, Ok(l)) = line {
            linenumber += 1;
            if let Some(output) = handle_line(&l, linenumber, &re, &args) {
                if found == false && args.names {
                    writeln!(writer, "{}", filename.purple())
                        .expect("ERROR: could not write to STDOUT");
                    found = true;
                }
                write!(writer, "{}", output).unwrap();
            }
        }
    }
}

fn handle_line(line: &str, linenumber: usize, re: &Regex, args: &Args) -> Option<String> {
    let matches = re.find_iter(line);
    let mut offset = 0;
    let mut found = false;
    let mut result = String::new();
    for (i, m) in matches.enumerate() {
        found = true; // marker that we have a match in this line

        // print all before match
        if i == 0 && args.linenumber {
            result.push_str(&format!("{}:", linenumber));
        }
        result.push_str(&format!("{}", &line[offset..m.start()]));

        // print match
        if args.color {
            result.push_str(&format!("{}", m.as_str().bold().red()));
        } else {
            result.push_str(&format!("{}", m.as_str()));
        }

        // advance position to after match
        offset = m.end();
    }

    // only print line if there was a match
    if found {
        // print all after last match
        result.push_str(&format!("{}\n", &line[offset..]));
        Some(result)
    } else {
        None
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

    /// enable recursive search in directories
    #[clap(short = 'r', long, value_parser)]
    recursive: bool,

    /// list of filenames to search in
    #[clap(value_parser)]
    filenames: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::fixture::{FileWriteStr, PathChild};

    #[test]
    fn test_tmp_dir_recursive() -> Result<(), Box<dyn std::error::Error>> {
        let dir = assert_fs::TempDir::new().unwrap();
        if let Some(dirname) = dir.path().to_str() {
            let file1 = dir.child("foo.txt");
            let file2 = dir.child("bar.txt");
            let file3 = dir.child("baz.txt");
            file1
                .write_str(
                    "bar baz
foo bar
baz Foo",
                )
                .unwrap();

            file2
                .write_str(
                    "bar baz
baz bar
foo foo FOO",
                )
                .unwrap();

            file3
                .write_str(
                    "bar baz
",
                )
                .unwrap();

            let args = Args {
                insensitive: true,
                query: "foo".to_string(),
                filenames: vec![dirname.to_string()],
                names: false,
                linenumber: false,
                color: false,
                recursive: true,
            };

            let mut v = Vec::new();
            let _ = grep(args, &mut v);

            // only use for checking manually
            // let _ = grep(args, &mut stdout().lock());

            let actual = String::from_utf8(v).expect("Not UTF-8");
            let expected = "foo bar\nbaz Foo\nfoo foo FOO\n\n";
            assert_eq!(expected, actual);
        }
        Ok(())
    }

    #[test]
    fn test_tmp_files_with_names_no_color() -> Result<(), Box<dyn std::error::Error>> {
        let poem = assert_fs::NamedTempFile::new("poem.txt")?;
        poem.write_str(
            "I'm nobody! Who are you?
Are you nobody, too?
Then there's a pair of us - don't tell!
They'd banish us, you know.
How dreary to be somebody!
How public, like a frog
To tell your name the livelong day
To an admiring bog!
",
        )?;

        if let Some(filename) = poem.to_str() {
            let args = Args {
                insensitive: true,
                query: "nobody".to_string(),
                filenames: vec![filename.to_string()],
                names: true,
                linenumber: true,
                color: false,
                recursive: false,
            };

            let mut v = Vec::new();
            let _ = grep(args, &mut v);

            let actual = String::from_utf8(v).expect("Not UTF-8");
            let expected = format!(
                "\u{1b}[35m{}\u{1b}[0m\n1:I'm nobody! Who are you?\n2:Are you nobody, too?\n\n",
                filename
            );
            assert_eq!(expected, actual);

            return Ok(());
        }

        panic!("temp file could not be used");
    }

    struct Case<'a> {
        testname: String,
        input: &'a [u8],
        query: String,
        names: bool,
        insensitive: bool,
        linenumber: bool,
        color: bool,
        expected: String,
    }

    macro_rules! grep_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let c = $value;
                    let mut v = Vec::new();
                    let io = Io {
                        input: c.input,
                        output: &mut v,
                    };
                    let args = Args {
                        insensitive: c.insensitive,
                        query: c.query.to_string(),
                        filenames: vec![],
                        names: c.names,
                        linenumber: c.linenumber,
                        color: c.color,
                        recursive: false,
                    };
                    let re = create_regex(&args);
                    from_stdin(io, args, &re).unwrap();

                    let actual = String::from_utf8(v).expect("Not UTF-8");
                    assert_eq!(c.expected, actual, "FAILED, testcase: {}", c.testname);
                }
            )*
        }
    }

    grep_tests! {
        grep_match_ending_case_insensitive: Case {
            testname: "match_ending_case_insensitive".to_string(),
            input: b"foo bar
bar baz
bar baz FOO
foo baz",
            query: "foo$".to_string(),
            names: false,
            insensitive: true,
            linenumber: false,
            color: false,
            expected: "bar baz FOO\n"
                .to_string(),
        },
        grep_match_ending_case_insensitive_with_color: Case {
            testname: "match_ending_case_insensitive_with_color".to_string(),
            input: b"foo bar
bar baz
bar baz FOO
foo baz",
            query: "foo$".to_string(),
            names: false,
            insensitive: true,
            linenumber: false,
            color: true,
            expected: "bar baz \u{1b}[1;31mFOO\u{1b}[0m\n"
                .to_string(),
        },
        grep_match_multiple_lines: Case {
            testname: "match_multiple_lines".to_string(),
            input: b"foo bar
bar baz
bar baz FOO
foo baz",
            query: "foo".to_string(),
            names: false,
            insensitive: false,
            linenumber: false,
            color: false,
            expected: "foo bar
foo baz
"
                .to_string(),
        },
        grep_match_case_sensitive: Case {
            testname: "match_case_sensitive".to_string(),
            input: b"foo bar
bar baz
bar baz FOO
foo baz",
            query: "FOO".to_string(),
            names: false,
            insensitive: false,
            linenumber: false,
            color: false,
            expected: "bar baz FOO\n"
                .to_string(),
        },

        grep_match_multiple_lines_with_linenumbers: Case {
            testname: "match_multiple_lines_with_linenumbers".to_string(),
            input: b"foo bar
bar baz
bar baz FOO
foo baz",
            query: "foo".to_string(),
            names: false,
            insensitive: false,
            linenumber: true,
            color: false,
            expected: "1:foo bar
4:foo baz
"
                .to_string(),
        },

    }
}
