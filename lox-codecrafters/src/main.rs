use std::env;
use std::fs;
use std::process::exit;
use lox_runtime::interpreter::Interpreter;
use lox_syntax::parser::Parser;
use lox_syntax::tokenizer::Scanner;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} tokenize <filename>", args[0]);
        eprintln!("Usage: {} parse <filename>", args[0]);
        eprintln!("Usage: {} evaluate <filename>", args[0]);
        eprintln!("Usage: {} run <filename>", args[0]);
        return;
    }

    let command = &args[1];
    let filename = &args[2];

    let file_contents = fs::read_to_string(filename).unwrap_or_else(|_| {
        eprintln!("Failed to read file {}", filename);
        String::new()
    });

    match command.as_str() {
        "tokenize" => {
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
                let mut interpreter = Interpreter::new(|_|{});
                let result = interpreter.evaluate_expression(&expression.unwrap());

                if result.is_ok() {
                    println!("{}", result.unwrap());
                } else {
                    eprintln!("{}", result.err().unwrap());
                    exit(70);
                }
            } else {
                eprintln!("{}", expression.err().unwrap());
                exit(65);
            }
        },
        "run" => {
            let mut scanner = Scanner::new(&file_contents);
            let (tokens, errors) = scanner.scan_tokens();

            for error in errors.iter() {
                eprintln!("{}", error);
            }

            if !errors.is_empty() {
                exit(65);
            }

            let mut parser = Parser::new(tokens);
            let statements = parser.parse();

            if statements.is_ok() {
                let mut interpreter = Interpreter::new(|value| println!("{}", value));
                let result = interpreter.run(&statements.unwrap());

                if result.is_ok() {
                } else {
                    eprintln!("{}", result.err().unwrap());
                    exit(70);
                }
            } else {
                eprintln!("{}", statements.err().unwrap());
                exit(65);
            }
        },
        _ => {
            eprintln!("Unknown command: {}", command);
        }
    }
}
