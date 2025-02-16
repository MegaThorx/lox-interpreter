mod utils;

use lox_syntax::tokenizer::Scanner;
use wasm_bindgen::prelude::*;
use js_sys::Function;
use lox_runtime::interpreter::Interpreter;
use lox_syntax::parser::Parser;

#[wasm_bindgen]
pub fn run(code: &str, print: Function) -> Result<(), String> {
    let mut scanner = Scanner::new(code);
    let (tokens, errors) = scanner.scan_tokens();

    if !errors.is_empty() {
        return Err(errors.iter().map(|error| error.to_string()).collect::<Vec<String>>().join("\n"));
    }

    let mut parser = Parser::new(tokens);
    let statements = parser.parse();

    if statements.is_ok() {
        let mut interpreter = Interpreter::new(|value| {    
            print.call1(&JsValue::NULL, &JsValue::from_str(&value)).unwrap();
        });
        
        let result = interpreter.run(&statements.unwrap());

        if result.is_ok() {
            Ok(())
        } else {
            Err(result.err().unwrap())
        }
    } else {
        Err(statements.err().unwrap())
    }
}

