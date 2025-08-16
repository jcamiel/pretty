use serde_json::Value;
use std::env::Args;
use std::io::Read;
use std::iter::Peekable;
use std::str::{Chars, Utf8Error};
use std::{env, io};

fn main() {
    let (with_serde, iter_count) = parse_args(env::args()).unwrap();

    let mut buffer = Vec::new();
    io::stdin().read_to_end(&mut buffer).unwrap();

    let run = if with_serde { pretty_serde } else { pretty };

    for _ in 1..=iter_count {
        let s = run(&buffer);
        println!("{s}");
    }
}

fn parse_args(args: Args) -> Result<(bool, usize), String> {
    let mut args = args.skip(1);
    let mut with_serde = false;
    let mut iter_count = 1;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--serde" => {
                with_serde = true;
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

    Ok((with_serde, iter_count))
}

fn pretty_serde(bytes: &[u8]) -> String {
    let json = serde_json::from_slice::<Value>(bytes).unwrap();
    serde_json::to_string_pretty(&json).unwrap()
}

#[allow(dead_code)]
fn pretty_raw(bytes: &[u8]) -> String {
    let mut json = Vec::with_capacity(bytes.len());
    let mut i = 0;

    // Process complete 8-byte chunks
    while i + 8 <= bytes.len() {
        let chunk = &bytes[i..i + 8];

        // Process all 8 bytes individually
        for &byte in chunk {
            json.push(byte);
        }

        i += 8;
    }

    // Handle remaining bytes (less than 8)
    if i < bytes.len() {
        for &byte in &bytes[i..] {
            json.push(byte);
        }
    }

    let s: String = unsafe { String::from_utf8_unchecked(json) };
    s
}

fn pretty(bytes: &[u8]) -> String {
    let mut parser = Parser::try_new(bytes).unwrap();
    parser.parse()
}

struct Parser<'input> {
    bytes_len: usize,
    buffer: Peekable<Chars<'input>>,
}

const SPACE: char = ' ';
const TAB: char = '\t';
const LF: char = '\n';
const CR: char = '\r';

impl<'input> Parser<'input> {
    fn try_new(buffer: &'input [u8]) -> Result<Self, String> {
        let buffer = str::from_utf8(buffer);
        let buffer = match buffer {
            Ok(b) => b,
            Err(_) => return Err("Invalid UTF-8 string".to_string()),
        };
        let bytes_len = buffer.len();
        let buffer = buffer.chars().peekable();
        let parser = Parser { buffer, bytes_len };
        Ok(parser)
    }

    pub fn parse(&mut self) -> String {
        let mut output = String::with_capacity(2 * self.bytes_len);

        loop {
            match self.buffer.next() {
                Some(SPACE) | Some(TAB) | Some(LF) | Some(CR) => continue,
                Some(c) => {
                    output.push(c);
                    continue;
                }
                None => break,
            }
        }
        output
    }
}

#[cfg(test)]
mod tests {
    use crate::Parser;

    // #[test]
    // fn parse_simple_json() {
    //     let input = r#"{
    //     "foo": "bar"
    //     }"#;
    //     let mut parser = Parser::new(input.as_bytes());
    //     let out = parser.parse();
    //     println!("{out}");
    // }
}
