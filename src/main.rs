mod format;

use crate::format::{Color, Parser};
use serde_json::Value;
use std::env::Args;
use std::env;
use std::path::PathBuf;

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
            eprintln!("Error reading file '{}': {}", config.file_path.display(), err);
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
    file_path: PathBuf,
}

fn print_usage() {
    println!("Usage: pretty [OPTIONS] <JSON_FILE>");
    println!();
    println!("A fast JSON pretty-printer");
    println!();
    println!("Arguments:");
    println!("  <JSON_FILE>  Path to the JSON file to format");
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
    let mut file_path: Option<PathBuf> = None;
    
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
                    file_path = Some(PathBuf::from(other));
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


