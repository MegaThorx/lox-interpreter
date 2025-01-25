mod syntax;
mod interpreter;

use std::env;
use std::fs;
use std::process::exit;
use crate::interpreter::evaluate;
use crate::syntax::parser::Parser;
use crate::syntax::tokenizer::Scanner;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} tokenize <filename>", args[0]);
        eprintln!("Usage: {} parse <filename>", args[0]);
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
        },
        "parse" => {
            let file_contents = fs::read_to_string(filename).unwrap_or_else(|_| {
                eprintln!("Failed to read file {}", filename);
                String::new()
            });

            let mut scanner = Scanner::new(&file_contents);
            let (tokens, errors) = scanner.scan_tokens();

            for error in errors.iter() {
                eprintln!("{}", error);
            }

            if !errors.is_empty() {
                exit(65);
            }

            let mut parser = Parser::new(tokens);
            let expression = parser.parse_expression();

            if expression.is_ok() {
                println!("{}", expression.unwrap());
            } else {
                eprintln!("{}", expression.err().unwrap());
                exit(65);
            }
        },
        "evaluate" => {
            let file_contents = fs::read_to_string(filename).unwrap_or_else(|_| {
                eprintln!("Failed to read file {}", filename);
                String::new()
            });

            let mut scanner = Scanner::new(&file_contents);
            let (tokens, errors) = scanner.scan_tokens();

            for error in errors.iter() {
                eprintln!("{}", error);
            }

            if !errors.is_empty() {
                exit(65);
            }

            let mut parser = Parser::new(tokens);
            let expression = parser.parse_expression();

            if expression.is_ok() {
                let result = evaluate(expression.unwrap());

                if result.is_ok() {
                    println!("{}", result.unwrap());
                } else {
                    eprintln!("{}", result.err().unwrap());
                    exit(65);
                }
            } else {
                eprintln!("{}", expression.err().unwrap());
                exit(65);
            }
        },
        _ => {
            eprintln!("Unknown command: {}", command);
        }
    }
}
