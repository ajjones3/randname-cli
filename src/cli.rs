// Argument parsing kept separate from std::env so it can be tested
// with plain string slices instead of real process arguments.

use crate::namegen::Style;

#[derive(Debug, PartialEq)]
pub struct Config {
    pub count: usize,
    pub syllables: usize,
    pub seed: Option<u64>,
    pub style: Style,
}

impl Default for Config {
    fn default() -> Self {
        Config { count: 5, syllables: 2, seed: None, style: Style::Common }
    }
}

/// Parses CLI flags into a Config. Accepts `--count`/`-c`,
/// `--syllables`/`-s`, `--seed`, and `--style`, each requiring a
/// following value. Unknown flags or malformed values come back as
/// an error instead of panicking, so main can decide how to report
/// them.
pub fn parse_args(args: &[String]) -> Result<Config, String> {
    let mut config = Config::default();
    let mut i = 0;

    while i < args.len() {
        let flag = args[i].as_str();
        match flag {
            "--count" | "-c" => config.count = parse_value(args, &mut i, flag)?,
            "--syllables" | "-s" => config.syllables = parse_value(args, &mut i, flag)?,
            "--seed" => config.seed = Some(parse_value(args, &mut i, flag)?),
            "--style" => {
                i += 1;
                let raw = args.get(i).ok_or_else(|| format!("{flag} requires a value"))?;
                config.style = raw.parse::<Style>()?;
            }
            other => return Err(format!("unknown flag: {other}")),
        }
        i += 1;
    }

    Ok(config)
}

fn parse_value<T: std::str::FromStr>(args: &[String], i: &mut usize, flag: &str) -> Result<T, String> {
    *i += 1;
    let raw = args.get(*i).ok_or_else(|| format!("{flag} requires a value"))?;
    raw.parse::<T>().map_err(|_| format!("{flag} expects a number, got '{raw}'"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn defaults_with_no_args() {
        assert_eq!(parse_args(&args(&[])).unwrap(), Config::default());
    }

    #[test]
    fn parses_count_and_syllables() {
        let config = parse_args(&args(&["--count", "10", "--syllables", "3"])).unwrap();
        assert_eq!(config.count, 10);
        assert_eq!(config.syllables, 3);
    }

    #[test]
    fn parses_short_flags() {
        let config = parse_args(&args(&["-c", "2", "-s", "1"])).unwrap();
        assert_eq!(config.count, 2);
        assert_eq!(config.syllables, 1);
    }

    #[test]
    fn parses_seed() {
        let config = parse_args(&args(&["--seed", "99"])).unwrap();
        assert_eq!(config.seed, Some(99));
    }

    #[test]
    fn parses_style() {
        let config = parse_args(&args(&["--style", "elvish"])).unwrap();
        assert_eq!(config.style, Style::Elvish);
    }

    #[test]
    fn rejects_unknown_style() {
        assert!(parse_args(&args(&["--style", "orcish"])).is_err());
    }

    #[test]
    fn rejects_unknown_flag() {
        assert!(parse_args(&args(&["--bogus"])).is_err());
    }

    #[test]
    fn rejects_missing_value() {
        assert!(parse_args(&args(&["--count"])).is_err());
    }

    #[test]
    fn rejects_non_numeric_value() {
        assert!(parse_args(&args(&["--count", "five"])).is_err());
    }
}
