mod format;

use crate::format::{Color, Parser};
use serde_json::Value;
use std::env::Args;
use std::io::Read;
use std::{env, io};

fn main() {
    let config = parse_args(env::args()).unwrap();

    let mut buffer = Vec::new();
    io::stdin().read_to_end(&mut buffer).unwrap();

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

struct Config {
    with_serde: bool,
    with_color: bool,
    iter_count: usize,
}

fn parse_args(args: Args) -> Result<Config, String> {
    let mut args = args.skip(1);
    let mut with_serde = false;
    let mut with_color = true;
    let mut iter_count = 1;

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
                let err = format!("Unknown argument: {other}");
                return Err(err);
            }
        }
    }

    Ok(Config {
        with_serde,
        with_color,
        iter_count,
    })
}
