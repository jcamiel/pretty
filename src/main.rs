use serde_json::Value;
use std::env::Args;
use std::io::Read;
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
    let mut parser = Parser::new(bytes);
    parser.parse()
}

struct Parser<'input> {
    buffer: &'input [u8],
    pos: usize,
}

const SPACE: u8 = b' ';
const TAB: u8 = b'\t';
const LF: u8 = b'\n';
const CR: u8 = b'\r';


impl<'input> Parser<'input> {
    fn new(buffer: &'input [u8]) -> Self {
        Parser { buffer, pos: 0 }
    }

    #[inline(always)]
    fn skip_whitespaces(&mut self) {
        let buffer = self.buffer;
        let len = buffer.len();
        let mut pos = self.pos;
        while pos < len {
            match buffer[pos] {
                SPACE | TAB | LF | CR => pos += 1,
                _ => break,
            }
        }
        self.pos = pos;
    }

    pub fn parse(&mut self) -> String {
        let buffer_len = self.buffer.len();
        let mut json = Vec::with_capacity(buffer_len);

        while self.pos < buffer_len {
            self.skip_whitespaces();
            if self.pos < buffer_len {
                json.push(self.buffer[self.pos]);
                self.pos += 1;
            }
        }

        unsafe { String::from_utf8_unchecked(json) }
    }
}

#[cfg(test)]
mod tests {
    use crate::Parser;

    #[test]
    fn parse_simple_json() {
        let input = r#"{
        "foo": "bar"
        }"#;
        let mut parser = Parser::new(input.as_bytes());
        let out = parser.parse();
        println!("{out}");
    }

}
