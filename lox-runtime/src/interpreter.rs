use std::fmt::Display;
use std::time::{SystemTime, UNIX_EPOCH};
use lox_syntax::expression::{BinaryOperation, Expression, Literal, UnaryOperation};
use lox_syntax::statement::Statement;
use crate::environment::Environment;

#[derive(PartialEq, Debug, Clone)]
pub enum Value {
    Bool(bool),
    Number(f64),
    String(String),
    Callable(Callable),
    None,
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(bool) => *bool,
            Value::None => false,
            _ => true,
        }
    }

    pub fn is_equal(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Bool(left), Value::Bool(right)) => left == right,
            (Value::Number(left), Value::Number(right)) => left == right,
            (Value::String(left), Value::String(right)) => left == right,
            (Value::None, Value::None) => true,
            _ => false,
        }
    }

    pub fn from_literal(literal: Literal) -> Value {
        match literal {
            Literal::Bool(value) => Value::Bool(value),
            Literal::Number(value) => Value::Number(value),
            Literal::String(value) => Value::String(value),
            Literal::None => Value::None,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Bool(bool) => write!(f, "{}", bool),
            Value::Number(number) => {
                match number.fract() == 0.0 {
                    true => write!(f, "{:.0}", number),
                    _ => write!(f, "{}", number),
                }
            },
            Value::String(string) => write!(f, "{}", string),
            Value::Callable(callable) => write!(f, "{}", callable),
            Value::None => write!(f, "nil"),
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum Callable {
    Native(usize, Box<fn(&Vec<Value>) -> Value>)
}


impl Display for Callable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Callable::Native(_, _) => write!(f, "<native fn>"),
        }
    }
}

pub struct Interpreter<F: FnMut(String)> {
    environment: Environment,
    print: F
}

impl<F: FnMut(String)> Interpreter<F> {
    pub fn new(print: F) -> Self {
        let mut environment = Environment::default();

        environment.declare("clock".to_string(), Value::Callable(
            Callable::Native(0, Box::new(|_args| {
                Value::Number(match SystemTime::now().duration_since(UNIX_EPOCH) {
                    Ok(duration) => duration.as_secs_f64().floor(),
                    Err(_) => 0.0,
                })
            }))
        ));

        Self {
            environment,
            print
        }
    }

    pub fn run(&mut self, statements: &Vec<Statement>) -> Result<(), String> {
        self.run_statements(statements)?;

        Ok(())
    }

    fn run_statements(&mut self, statements: &Vec<Statement>) -> Result<(), String> {
        for statement in statements {
            self.run_statement(statement)?;
        }

        Ok(())
    }

    fn run_statement(&mut self, statement: &Statement) -> Result<(), String> {
        match statement {
            Statement::Print(expression) => {
                let value = format!("{}", self.evaluate(expression)?);
                (self.print)(value);
            },
            Statement::Expression(expression) => {
                self.evaluate(expression)?;
            },
            Statement::Variable(name, expression) => {
                if expression.is_some() {
                    let value = self.evaluate(expression.as_ref().unwrap())?;
                    self.environment.declare(name.to_string(), value);
                } else {
                    self.environment.declare(name.to_string(), Value::None);
                }
            },
            Statement::Block(statements) => {
                self.environment.push_scope();
                let result =self.run_statements(statements);
                self.environment.pop_scope();
                if result.is_err() {
                    return Err(result.err().unwrap())
                }
            },
            Statement::If(condition, if_body, else_body) => {
                if self.evaluate(condition)?.is_truthy() {
                    self.environment.push_scope();
                    let result = self.run_statement(if_body);
                    self.environment.pop_scope();
                    if result.is_err() {
                        return Err(result.err().unwrap())
                    }
                } else if let Some(else_body) = else_body {
                    self.environment.push_scope();
                    let result = self.run_statement(else_body);
                    self.environment.pop_scope();
                    if result.is_err() {
                        return Err(result.err().unwrap())
                    }
                }
            },
            Statement::While(condition, body) => {
                while self.evaluate(condition)?.is_truthy() { // TODO: If the evaluate errors it will not pop the scope
                    self.environment.push_scope();
                    let result = self.run_statement(body);
                    self.environment.pop_scope();
                    if result.is_err() {
                        return Err(result.err().unwrap())
                    }
                }
            },
            Statement::For(initial, condition, incrementer, body) => {
                self.environment.push_scope();
                
                if let Some(initial) = initial {
                    let result = self.run_statement(initial);
                    if result.is_err() {
                        self.environment.pop_scope();
                        return Err(result.err().unwrap())
                    }
                }

                while {
                    if let Some(condition) = condition {
                        self.evaluate(condition)?.is_truthy()
                    } else {
                        true
                    }
                } { // TODO: If the evaluate errors it will not pop the scope
                    let result = self.run_statement(body);

                    if result.is_err() {
                        self.environment.pop_scope();
                        return Err(result.err().unwrap())
                    }

                    if let Some(incrementer) = incrementer {
                        self.evaluate(incrementer)?; // TODO: If the evaluate errors it will not pop the scope
                    }
                }
                self.environment.pop_scope();
            }
        }

        Ok(())
    }

    pub fn evaluate(&mut self, expression: &Expression) -> Result<Value, String> {
        match expression {
            Expression::Assign(name, expression) => {
                let result = self.evaluate(expression)?;
                self.environment.assign(name.clone(), result.clone())?;
                Ok(result)
            },
            Expression::Literal(literal) => Ok(Value::from_literal(literal.clone())),
            Expression::Grouping(expression) => self.evaluate(expression),
            Expression::Unary(operation, expression) => {
                match operation {
                    UnaryOperation::Minus => match self.evaluate(expression)? {
                        Value::Number(number) => Ok(Value::Number(-number)),
                        _ => Err("Operand must be a number.".to_string()),
                    },
                    UnaryOperation::Not => Ok(Value::Bool(!self.evaluate(expression)?.is_truthy())),
                }
            },
            Expression::Binary(operation, left, right) => {
                let left = self.evaluate(left)?;
                let right = self.evaluate(right)?;

                Ok(match operation {
                    BinaryOperation::Equal => Value::Bool(left.is_equal(&right)),
                    BinaryOperation::NotEqual => Value::Bool(!left.is_equal(&right)),
                    operation => match (left, right) {
                        (Value::Number(left), Value::Number(right)) => match operation {
                            BinaryOperation::Multiply => Value::Number(left * right),
                            BinaryOperation::Divide => Value::Number(left / right),
                            BinaryOperation::Plus => Value::Number(left + right),
                            BinaryOperation::Minus => Value::Number(left - right),
                            BinaryOperation::Greater => Value::Bool(left > right),
                            BinaryOperation::GreaterEqual => Value::Bool(left >= right),
                            BinaryOperation::Less => Value::Bool(left < right),
                            _ => Value::Bool(left <= right), // Last one can only be LessEqual
                        },
                        (Value::String(left), Value::String(right)) => match operation {
                            BinaryOperation::Plus => Value::String(format!("{}{}", left, right)),
                            _ => return Err("Operands must be a numbers.".to_string()),
                        }
                        (_, _) => return Err("Operands must be a numbers.".to_string())
                    }
                })
            },
            Expression::Variable(name) => {
                if let Some(value) = self.environment.get(name) {
                    match value {
                        Value::Bool(boolean) => Ok(Value::Bool(*boolean)),
                        Value::Number(number) => Ok(Value::Number(*number)),
                        Value::String(string) => Ok(Value::String(string.clone())),
                        Value::Callable(callable) => Ok(Value::Callable(callable.clone())),
                        Value::None => Ok(Value::None),
                    }
                } else {
                    Err(format!("Undefined variable '{}'.", name))
                }
            },
            Expression::And(left, right) => {
                let left = self.evaluate(left)?;

                if !left.is_truthy() {
                    return Ok(left);
                }

                self.evaluate(right)
            },
            Expression::Or(left, right) => {
                let left = self.evaluate(left)?;

                if left.is_truthy() {
                    return Ok(left);
                }

                self.evaluate(right)
            },
            Expression::Call(callee, _arguments) => {
                let callee = self.evaluate(callee)?;

                match callee {
                    Value::Callable(callable) => {
                        match callable {
                            Callable::Native(_ary, function) => {
                                Ok(function(&Vec::new()))
                            }
                        }
                    }
                    _ => Err("Can only call functions and classes.".to_string())
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;
    use std::time::Duration;
    use lox_syntax::parser::Parser;
    use lox_syntax::tokenizer::Scanner;
    use crate::interpreter::{Interpreter, Value};

    fn run_evaluate(source: &str) -> Result<Value, String> {
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let mut interpreter = Interpreter::new(|_|{});
        interpreter.evaluate(&parser.parse_expression()?)
    }

    fn run_statement(source: &str) -> Result<Vec<String>, String> {
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let mut prints: Vec<String> = Vec::new();
        let mut interpreter = Interpreter::new(|value|{
            prints.push(value);
        });
        interpreter.run(&parser.parse()?)?;
        Ok(prints)
    }

    #[rstest]
    #[case("true", "true")]
    #[case("false", "false")]
    #[case("nil", "nil")]
    fn test_evaluate_booleans_and_nil(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(expected, run_evaluate(input).unwrap().to_string());
    }

    #[rstest]
    #[case("\"hello world!\"", "hello world!")]
    #[case("\"foo!\"", "foo!")]
    #[case("\"hello\non\nthe\nother\nside\"", "hello\non\nthe\nother\nside")]
    fn test_evaluate_string(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(expected, run_evaluate(input).unwrap().to_string());
    }

    #[rstest]
    #[case("10.40", "10.4")]
    #[case("10.41", "10.41")]
    #[case("54.12300", "54.123")]
    fn test_evaluate_float(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(expected, run_evaluate(input).unwrap().to_string());
    }

    #[rstest]
    #[case("10", "10")]
    #[case("123", "123")]
    #[case("54", "54")]
    fn test_evaluate_integer(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(expected, run_evaluate(input).unwrap().to_string());
    }

    #[rstest]
    #[case("(\"hello world!\")", "hello world!")]
    #[case("((\"hello world!\"))", "hello world!")]
    #[case("(true)", "true")]
    #[case("(10.40)", "10.4")]
    #[case("((false))", "false")]
    fn test_evaluate_group(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(expected, run_evaluate(input).unwrap().to_string());
    }

    #[rstest]
    #[case("-73", "-73")]
    #[case("--73", "73")]
    #[case("!true", "false")]
    #[case("!false", "true")]
    #[case("!nil", "true")]
    #[case("!10.40", "false")]
    #[case("!\"hello\"", "false")]
    #[case("!!false", "false")]
    #[case("!(!false)", "false")]
    fn test_evaluate_unary(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(expected, run_evaluate(input).unwrap().to_string());
    }

    #[rstest]
    #[case("42 / 5", "8.4")]
    #[case("18 * 3 / (3 * 6)", "3")]
    #[case("(10.40 * 2) / 2", "10.4")]
    #[case("70 - 65", "5")]
    #[case("69 - 93", "-24")]
    #[case("10.40 - 2", "8.4")]
    #[case("23 + 28 - (-(61 - 99))", "13")]
    fn test_evaluate_arithmetic(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(expected, run_evaluate(input).unwrap().to_string());
    }

    #[rstest]
    #[case("\"hello\" + \" world!\"", "hello world!")]
    #[case("\"foo\" + \"bar\"", "foobar")]
    #[case("\"42\" + \"24\"", "4224")]
    fn test_evaluate_string_concatenation(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(expected, run_evaluate(input).unwrap().to_string());
    }

    #[rstest]
    #[case("57 > -65", "true")]
    #[case("57 > 65", "false")]
    #[case("11 >= 11", "true")]
    #[case("12 >= 11", "true")]
    #[case("10 >= 11", "false")]
    #[case("57 > -65", "true")]
    #[case("(54 - 67) >= -(114 / 57 + 11)", "true")]
    #[case("57 < 65", "true")]
    #[case("57 < -65", "false")]
    #[case("11 <= 11", "true")]
    #[case("12 <= 11", "false")]
    #[case("10 <= 11", "true")]
    fn test_evaluate_relational(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(expected, run_evaluate(input).unwrap().to_string());
    }

    #[rstest]
    #[case("\"hello\" == \"world\"", "false")]
    #[case("\"foo\" == \"foo\"", "true")]
    #[case("true == true", "true")]
    #[case("false == false", "true")]
    #[case("true == false", "false")]
    #[case("5 == 5", "true")]
    #[case("5 == 6", "false")]
    #[case("5.5 == 5.5", "true")]
    #[case("5.5 == 6.5", "false")]
    #[case("nil == nil", "true")]
    #[case("true == nil", "false")]
    #[case("1 == nil", "false")]
    #[case("1 == false", "false")]
    #[case("1 == \"foo\"", "false")]
    fn test_evaluate_equality_equals(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(expected, run_evaluate(input).unwrap().to_string());
    }

    #[rstest]
    #[case("\"hello\" != \"world\"", "true")]
    #[case("\"foo\" != \"foo\"", "false")]
    #[case("true != true", "false")]
    #[case("false != false", "false")]
    #[case("true != false", "true")]
    #[case("5 != 5", "false")]
    #[case("5 != 6", "true")]
    #[case("5.5 != 5.5", "false")]
    #[case("5.5 != 6.5", "true")]
    #[case("nil != nil", "false")]
    fn test_evaluate_equality_not_equals(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(expected, run_evaluate(input).unwrap().to_string());
    }

    #[rstest]
    #[case("-\"foo\"", "Operand must be a number.")]
    #[case("-false", "Operand must be a number.")]
    #[case("-nil", "Operand must be a number.")]
    #[case("\"foo\" * 42", "Operands must be a numbers.")]
    #[case("(\"foo\" * \"bar\")", "Operands must be a numbers.")]
    #[case("true / 2", "Operands must be a numbers.")]
    #[case("true / false", "Operands must be a numbers.")]
    #[case("\"foo\" + true", "Operands must be a numbers.")]
    #[case("42 - true", "Operands must be a numbers.")]
    #[case("true + false", "Operands must be a numbers.")]
    #[case("\"foo\" - \"bar\"", "Operands must be a numbers.")]
    #[case("\"foo\" < false", "Operands must be a numbers.")]
    #[case("true < 2", "Operands must be a numbers.")]
    #[case("(\"foo\" + \"bar\") < 42", "Operands must be a numbers.")]
    #[case("false > true", "Operands must be a numbers.")]
    #[case("\"foo\" <= false", "Operands must be a numbers.")]
    #[case("\"foo\" >= false", "Operands must be a numbers.")]
    fn test_evaluate_runtime_error(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(expected, run_evaluate(input).err().unwrap());
    }

    #[rstest]
    #[case("print \"hello\";", vec!["hello"])]
    #[case("var a = 1;print a;{var a = 2; print a;}print a;", vec!["1", "2", "1"])]
    #[case("var a = 1;print a;{a = 2; print a;}print a;", vec!["1", "2", "2"])]
    #[case("var a;print a;{a = 2; print a;}print a;", vec!["nil", "2", "2"])]
    #[case("var a = \"a\";print a;{var a = true; print a;}a = nil; print a;", vec!["a", "true", "nil"])]
    #[case("if (true) print \"a\";", vec!["a"])]
    #[case("if (true) { print \"a\"; }", vec!["a"])]
    #[case("if (true) { print \"a\"; } else { print \"b\"; }", vec!["a"])]
    #[case("if (false) { print \"a\"; } else { print \"b\"; }", vec!["b"])]
    #[case("if (true) { print \"a\"; } else if (true) { print \"b\"; }", vec!["a"])]
    #[case("if (false) { print \"a\"; } else if (true) { print \"b\"; }", vec!["b"])]
    fn test_statements(#[case] input: &str, #[case] expected: Vec<&str>) {
        assert_eq!(expected, run_statement(input).unwrap());
    }


    #[rstest]
    #[case("print \"hi\" or 2;", vec!["hi"])]
    #[case("print nil or \"yes\";", vec!["yes"])]
    #[case("print false or \"ok\";", vec!["ok"])]
    #[case("print nil or \"ok\";", vec!["ok"])]
    #[case("print nil or false;", vec!["false"])]
    #[case("print true or \"bar\";", vec!["true"])]
    #[case("print 22 or \"quz\";", vec!["22"])]
    #[case("print 22 and \"quz\";", vec!["quz"])]
    #[case("print true and false;", vec!["false"])]
    #[case("print false and true;", vec!["false"])]
    #[case("print \"quz\" or \"quz\";", vec!["quz"])]
    #[case("if (\"hi\" or 2) { print \"yes\"; }", vec!["yes"])]
    #[case("if (false) {  } else { print \"yes\"; }", vec!["yes"])]
    #[case("if (false) {  }", vec![])]
    fn test_statements_logical(#[case] input: &str, #[case] expected: Vec<&str>) {
        assert_eq!(expected, run_statement(input).unwrap());
    }

    #[rstest]
    #[case("if (a) { print \"yes\"; }", "Undefined variable 'a'.")]
    #[case("if (1) { print a; }", "Undefined variable 'a'.")]
    #[case("if (false) { } else { print a; }", "Undefined variable 'a'.")]
    fn test_statements_logical_error(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(expected, run_statement(input).err().unwrap());
    }


    #[rstest]
    #[case("var i = 0; while(i < 5) {i = i + 1; print \"hi\"; }", vec!["hi", "hi", "hi", "hi", "hi"])]
    fn test_statements_while(#[case] input: &str, #[case] expected: Vec<&str>) {
        assert_eq!(expected, run_statement(input).unwrap());
    }

    #[rstest]
    #[case("while(i < 5) {i = i + 1; print \"hi\"; }", "Undefined variable 'i'.")]
    #[case("var i = 0; while(i < 5) {i = a + 1; print \"hi\"; }", "Undefined variable 'a'.")]
    fn test_statements_while_error(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(expected, run_statement(input).err().unwrap());
    }


    #[rstest]
    #[case("for (var baz = 0; baz < 3;) print baz = baz + 1;", vec!["1", "2", "3"])]
    #[case("for (var world = 0; world < 3; world = world + 1) { print world; }", vec!["0", "1", "2"])]
    fn test_statements_for(#[case] input: &str, #[case] expected: Vec<&str>) {
        assert_eq!(expected, run_statement(input).unwrap());
    }

    #[rstest]
    #[case("for (;i < 5;) {i = i + 1; print \"hi\"; }", "Undefined variable 'i'.")]
    #[case("for (;;) { print a; }", "Undefined variable 'a'.")]
    #[case("for (i = 0;;) { print a; }", "Undefined variable")]
    fn test_statements_for_error(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(expected, run_statement(input).err().unwrap());
    }

    #[rstest]
    #[case("clock();", vec![])]
    fn test_statements_call(#[case] input: &str, #[case] expected: Vec<&str>) {
        assert_eq!(expected, run_statement(input).unwrap());
    }

    #[rstest]
    #[case("print a;", "Undefined variable 'a'.")]
    #[timeout(Duration::from_millis(50))]
    #[case("for(;;) var a;", "[line 1] Error at 'var': Expect expression.")]
    fn test_statements_error(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(expected, run_statement(input).err().unwrap());
    }
}