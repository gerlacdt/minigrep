## minigrep

A learning project which has similar functionality as `grep`.

It support various features:

* read input from STDIN
* read input from multiple files
* read input recursively from a directory
* show line numbers (optional)
* color matches (optional)
* show filenames with matches (optional)
* Perl-style regular expressions via [regex](https://docs.rs/regex/latest/regex/)


### Example usage

``` bash
# after compilation with cargo build, you can run the binary

# run without options to see help
target/debug/minigrep

# more detailed help
target/debug/minigrep --help

# search recursively in current directory for "foo"
target/debug/minigrep . -r -q "foo"

# search recursively in current directory for "foo" with color output, linenumbers, filenames and case-insensitive
target/debug/minigrep . -r -q "foo" --color -n --with-filenames --insensitive

# same as above with short options
target/debug/minigrep . -r -q "foo" -c -n -H -i

# search explicitly in files
target/debug/minigrep src/lib.rs src/main.rs -q "foo" -c -n -H

# search in STDIN
find src -iname "*.rs" | xargs target/debug/minigrep -q "foo" -H -c -i
```


### Development

``` bash
# build project
cargo build

# run tests
cargo test

# run single tes
cargo test test_tmp_dir_recursive
```

#### More features

* consider ignored files and directories from .gitignore and other standard tools
* better performance, distribute parallel work over multiple cores
* more pragmatic Rust code, better tests, add integration tests
