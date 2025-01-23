mod syntax;

use std::env;
use std::fs;
use std::io::{self, Write};
use std::process::exit;
use crate::syntax::tokenizer::Scanner;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        writeln!(io::stderr(), "Usage: {} tokenize <filename>", args[0]).unwrap();
        return;
    }

    let command = &args[1];
    let filename = &args[2];

    match command.as_str() {
        "tokenize" => {
            let file_contents = fs::read_to_string(filename).unwrap_or_else(|_| {
                eprintln!("Failed to read file {}", filename);
                String::new()
            });

            let mut scanner = Scanner::new(&file_contents);
            let (tokens, errors) = scanner.scan_tokens();

            for error in errors.iter() {
                eprintln!("{}", error);
            }

            for token in tokens {
                println!("{}", token);
            }

            if !errors.is_empty() {
                exit(65);
            }
        }
        _ => {
            writeln!(io::stderr(), "Unknown command: {}", command).unwrap();
            return;
        }
    }
}
