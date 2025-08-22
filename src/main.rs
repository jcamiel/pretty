mod format;

use crate::format::{Color, Parser};
use serde_json::Value;
use std::env;
use std::env::Args;
use std::io::Read;
use std::path::PathBuf;

fn main() {
    let config = match parse_args(env::args()) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Error parsing arguments: {}", err);
            std::process::exit(1);
        }
    };

    let buffer = match &config.file_path {
        None => {
            // Read from stdin
            let mut buffer = Vec::new();
            match std::io::stdin().read_to_end(&mut buffer) {
                Ok(_) => buffer,
                Err(err) => {
                    eprintln!("Error reading from stdin: {}", err);
                    std::process::exit(1);
                }
            }
        }
        Some(path) => {
            // Read from file
            match std::fs::read(path) {
                Ok(data) => data,
                Err(err) => {
                    eprintln!("Error reading file '{}': {}", path.display(), err);
                    std::process::exit(1);
                }
            }
        }
    };

    let run = if config.with_serde {
        pretty_serde
    } else {
        pretty
    };

    for _ in 1..=config.iter_count {
        match run(&buffer, config.with_color) {
            Ok(s) => println!("{s}"),
            Err(err) => {
                eprintln!("Error: {err}");
                std::process::exit(2);
            }
        }
    }
}

fn pretty_serde(bytes: &[u8], _color: bool) -> Result<String, String> {
    let json = serde_json::from_slice::<Value>(bytes).unwrap();
    serde_json::to_string_pretty(&json).map_err(|err| err.to_string())
}

fn pretty(bytes: &[u8], color: bool) -> Result<String, String> {
    let color = if color {
        Color::AnsiCode
    } else {
        Color::NoColor
    };
    let mut parser = Parser::new(bytes, color);
    let mut output = String::new();
    parser.format(&mut output).map_err(|err| err.to_string())?;
    Ok(output)
}

#[derive(Debug)]
struct Config {
    with_serde: bool,
    with_color: bool,
    iter_count: usize,
    file_path: Option<PathBuf>,
}

fn print_usage() {
    println!("Usage: pretty [OPTIONS] <JSON_FILE>");
    println!();
    println!("A fast JSON pretty-printer");
    println!();
    println!("Arguments:");
    println!("  <JSON_FILE>  Path to the JSON file to format (use '-' for stdin)");
    println!();
    println!("Options:");
    println!("  --serde       Use serde for JSON parsing");
    println!("  --no-color    Disable colored output");
    println!("  --iter <N>    Number of iterations to run [default: 1]");
    println!("  -h, --help    Print this help message");
}

fn parse_args(args: Args) -> Result<Config, String> {
    let args: Vec<String> = args.skip(1).collect();

    // Handle help flags first
    if args.is_empty() {
        print_usage();
        std::process::exit(0);
    }

    for arg in &args {
        if arg == "--help" || arg == "-h" {
            print_usage();
            std::process::exit(0);
        }
    }

    let mut with_serde = false;
    let mut with_color = true;
    let mut iter_count = 1;
    let mut file_path: Option<Option<PathBuf>> = None;
    let mut args_iter = args.into_iter();

    while let Some(arg) = args_iter.next() {
        match arg.as_str() {
            "--serde" => {
                with_serde = true;
            }
            "--no-color" => {
                with_color = false;
            }
            "--iter" => {
                if let Some(value) = args_iter.next() {
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
                    if other == "-" {
                        file_path = Some(None);
                    } else {
                        file_path = Some(Some(PathBuf::from(other)));
                    }
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
