mod format;

use crate::format::{Color, Parser};
use serde_json::Value;
use std::env::Args;
use std::{env, io};

fn main() {
    let config = match parse_args(env::args()) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Error parsing arguments: {}", err);
            std::process::exit(1);
        }
    };

    let mut buffer = Vec::new();
    if let Err(err) = read_stdin_to_buffer(&mut buffer) {
        eprintln!("Error reading from stdin: {}", err);
        std::process::exit(1);
    }

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

fn read_stdin_to_buffer(buffer: &mut Vec<u8>) -> io::Result<usize> {
    // On Windows, when reading large amounts of data from stdin through PowerShell piping,
    // we might encounter issues with console buffer limitations. Use chunked reading
    // to handle this more robustly.
    #[cfg(windows)]
    {
        read_stdin_chunked(buffer)
    }
    #[cfg(not(windows))]
    {
        use std::io::Read;
        io::stdin().read_to_end(buffer)
    }
}

#[cfg(windows)]
fn read_stdin_chunked(buffer: &mut Vec<u8>) -> io::Result<usize> {
    use std::io::Read;
    
    // Use a moderate chunk size to avoid potential buffer issues on Windows
    const CHUNK_SIZE: usize = 8192;
    let mut chunk = vec![0u8; CHUNK_SIZE];
    let mut total_read = 0;
    
    loop {
        match io::stdin().read(&mut chunk) {
            Ok(0) => break, // EOF
            Ok(n) => {
                buffer.extend_from_slice(&chunk[..n]);
                total_read += n;
            }
            Err(e) => return Err(e),
        }
    }
    
    Ok(total_read)
}

#[derive(Debug)]
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

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Read};

    #[test]
    fn test_chunked_reading_simulation() {
        // Test that chunked reading works correctly with large data
        let test_data = vec![42u8; 20000]; // 20KB of test data
        let mut cursor = Cursor::new(test_data.clone());
        let mut buffer = Vec::new();
        
        // Simulate our Windows chunked reading approach
        const CHUNK_SIZE: usize = 8192;
        let mut chunk = vec![0u8; CHUNK_SIZE];
        let mut total_read = 0;
        
        loop {
            match cursor.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => {
                    buffer.extend_from_slice(&chunk[..n]);
                    total_read += n;
                }
                Err(_) => panic!("Read error"),
            }
        }
        
        assert_eq!(buffer, test_data);
        assert_eq!(total_read, 20000);
    }
}
