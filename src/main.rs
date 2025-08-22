mod format;

use crate::format::{Color, Parser};
use serde_json::Value;
use std::env::Args;
use std::env;

fn main() {
    let config = match parse_args(env::args()) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Error parsing arguments: {}", err);
            std::process::exit(1);
        }
    };

    let buffer = match std::fs::read(&config.file_path) {
        Ok(data) => data,
        Err(err) => {
            eprintln!("Error reading file '{}': {}", config.file_path, err);
            std::process::exit(1);
        }
    };

    let run = if config.with_serde {
        pretty_serde
    } else {
        pretty
    };

    for _ in 1..=config.iter_count {
        let s = run(&buffer, config.with_color);
        println!("{s}");
    }
}

fn pretty_serde(bytes: &[u8], _color: bool) -> String {
    let json = serde_json::from_slice::<Value>(bytes).unwrap();
    serde_json::to_string_pretty(&json).unwrap()
}

fn pretty(bytes: &[u8], color: bool) -> String {
    let color = if color {
        Color::AnsiCode
    } else {
        Color::NoColor
    };
    let mut parser = Parser::new(bytes, color);
    let mut output = String::new();
    parser.format(&mut output).unwrap();
    output
}


#[derive(Debug)]
struct Config {
    with_serde: bool,
    with_color: bool,
    iter_count: usize,
    file_path: String,
}

fn parse_args(args: Args) -> Result<Config, String> {
    parse_args_impl(args.skip(1))
}

fn parse_args_impl<I>(mut args: I) -> Result<Config, String>
where
    I: Iterator<Item = String>,
{
    let mut with_serde = false;
    let mut with_color = true;
    let mut iter_count = 1;
    let mut file_path = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--serde" => {
                with_serde = true;
            }
            "--no-color" => {
                with_color = false;
            }
            "--iter" => {
                if let Some(value) = args.next() {
                    match value.parse::<usize>() {
                        Ok(v) => iter_count = v,
                        Err(_) => {
                            let err = format!("Invalid value for --iter: {value}");
                            return Err(err);
                        }
                    }
                } else {
                    return Err("Missing value for --iter".to_string());
                }
            }
            other => {
                if file_path.is_none() {
                    file_path = Some(other.to_string());
                } else {
                    let err = format!("Unknown argument: {other}");
                    return Err(err);
                }
            }
        }
    }

    let file_path = file_path.ok_or("Missing required argument: JSON file path")?;

    Ok(Config {
        with_serde,
        with_color,
        iter_count,
        file_path,
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_file_argument_required() {
        use super::parse_args_impl;
        
        // Test that parse_args requires a file argument
        let args = vec![];
        let result = parse_args_impl(args.into_iter());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing required argument: JSON file path"));
    }

    #[test]
    fn test_parse_args_with_file() {
        use super::parse_args_impl;
        
        let args = vec!["test.json".to_string()];
        let result = parse_args_impl(args.into_iter());
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.file_path, "test.json");
        assert!(!config.with_serde);
        assert!(config.with_color);
        assert_eq!(config.iter_count, 1);
    }

    #[test]
    fn test_parse_args_with_options() {
        use super::parse_args_impl;
        
        let args = vec![
            "--serde".to_string(),
            "--no-color".to_string(),
            "--iter".to_string(),
            "5".to_string(),
            "test.json".to_string(),
        ];
        let result = parse_args_impl(args.into_iter());
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.file_path, "test.json");
        assert!(config.with_serde);
        assert!(!config.with_color);
        assert_eq!(config.iter_count, 5);
    }
}
