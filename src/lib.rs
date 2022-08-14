use std::{
    env,
    error::Error,
    io::{stdin, Read},
    str,
};

use colored::Colorize;
use regex::{Regex, RegexBuilder};

pub fn grep(config: Config) -> Result<(), Box<dyn Error>> {
    let lines = stdin().lines();

    let re = match RegexBuilder::new(&config.query)
        .case_insensitive(config.ignore_case)
        .build()
    {
        // let re = match Regex::new(&config.query) {
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

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let mut contents = String::new();
    stdin().read_to_string(&mut contents)?;

    let results = if config.ignore_case {
        search_case_insensitive(&config.query, &contents)
    } else {
        search(&config.query, &contents)
    };

    for line in results {
        println!("{}", line);
    }
    Ok(())
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Config {
    pub query: String,
    pub ignore_case: bool,
}

impl Config {
    pub fn new(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        args.next(); // fist argument is the program name, so skip it
        let query = match args.next() {
            Some(arg) => arg,
            None => return Err("Did not get a query string"),
        };

        let ignore_case = env::var("IGNORE_CASE").is_ok();

        Ok(Config { query, ignore_case })
    }
}

pub fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    contents
        .lines()
        .filter(|line| line.contains(query))
        .collect()
}

pub fn search_case_insensitive<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    let query = query.to_lowercase();
    contents
        .lines()
        .filter(|line| line.to_lowercase().contains(&query))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_sensitive() {
        let query = "duct";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.
Duct tape.";

        assert_eq!(vec!["safe, fast, productive."], search(query, contents));
    }

    #[test]
    fn case_insensitive() {
        let query = "rUsT";
        let contents = "\
        Rust:
safe, fast, productive.
Pick three.
Trust me.";

        assert_eq!(
            vec!["Rust:", "Trust me."],
            search_case_insensitive(query, contents)
        );
    }
}
