use std::collections::HashMap;
use std::fmt::Display;
use crate::syntax::expression::{BinaryOperation, Expression, Literal, UnaryOperation};
use crate::syntax::statement::Statement;

pub enum Value {
    Bool(bool),
    Number(f64),
    String(String),
    None,
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
            Value::None => write!(f, "nil"),
        }
    }
}

impl Value {
    pub fn from_literal(literal: Literal) -> Value {
        match literal {
            Literal::Bool(value) => Value::Bool(value),
            Literal::Number(value) => Value::Number(value),
            Literal::String(value) => Value::String(value),
            Literal::None => Value::None,
        }
    }
}

pub fn run(statements: Vec<Statement>) -> Result<(), String>{
    let mut variables: HashMap<String, Value> = HashMap::new();
    
    for statement in statements {
        match statement {
            Statement::Print(expression) => println!("{}", evaluate(expression, Some(&mut variables))?),
            Statement::Expression(expression) => {
                evaluate(expression, Some(&mut variables))?;
            },
            Statement::Variable(name, expression) => {
                if expression.is_some() {
                    let value = evaluate(expression.unwrap(), Some(&mut variables))?;
                    variables.insert(name, value);
                } else {
                    variables.insert(name, Value::None);
                }
            },
        }
    }

    Ok(())
}

pub fn evaluate(expression: Expression, variables: Option<&mut HashMap<String, Value>>) -> Result<Value, String> {
    let result = evaluate_expression(expression, variables);
    if let Ok(literal) = result {
        Ok(Value::from_literal(literal))
    } else {
        Err(result.err().unwrap())
    }
}

fn evaluate_expression(expression: Expression, variables: Option<&mut HashMap<String, Value>>) -> Result<Literal, String> {
    match expression {
        Expression::Assign(name, expression) => {
            if let Some(variables) = variables {
                let result = evaluate_expression(*expression, Some(variables))?;
                variables.insert(name, Value::from_literal(result.clone()));

                Ok(result)
            } else {
                Ok(Literal::None)
            }
        },
        expression => {
            if let Some(variables) = variables {
                evaluate_expression_read_only(expression, Some(variables))
            } else {
                evaluate_expression_read_only(expression, None)
            }
        }
    }
}


fn evaluate_expression_read_only(expression: Expression, variables: Option<&HashMap<String, Value>>) -> Result<Literal, String> {
    match expression {
        Expression::Literal(literal) => Ok(literal),
        Expression::Grouping(expression) => evaluate_expression_read_only(*expression, variables),
        Expression::Unary(operation, expression) => {
            match operation {
                UnaryOperation::Minus => match evaluate_expression_read_only(*expression, variables)? {
                    Literal::Number(number) => Ok(Literal::Number(-number)),
                    _ => Err("Operand must be a number.".to_string()),
                },
                UnaryOperation::Not => Ok(Literal::Bool(!evaluate_expression_read_only(*expression, variables)?.is_truthy())),
            }
        },
        Expression::Binary(operation, left, right) => {
            let left = evaluate_expression_read_only(*left, variables)?;
            let right = evaluate_expression_read_only(*right, variables)?;

            Ok(match operation {
                BinaryOperation::Equal => Literal::Bool(left.is_equal(&right)),
                BinaryOperation::NotEqual => Literal::Bool(!left.is_equal(&right)),
                operation => match (left, right) {
                    (Literal::Number(left), Literal::Number(right)) => match operation {
                        BinaryOperation::Multiply => Literal::Number(left * right),
                        BinaryOperation::Divide => Literal::Number(left / right),
                        BinaryOperation::Plus => Literal::Number(left + right),
                        BinaryOperation::Minus => Literal::Number(left - right),
                        BinaryOperation::Greater => Literal::Bool(left > right),
                        BinaryOperation::GreaterEqual => Literal::Bool(left >= right),
                        BinaryOperation::Less => Literal::Bool(left < right),
                        _ => Literal::Bool(left <= right), // Last one can only be LessEqual
                    },
                    (Literal::String(left), Literal::String(right)) => match operation {
                        BinaryOperation::Plus => Literal::String(format!("{}{}", left, right)),
                        _ => return Err("Operands must be a numbers.".to_string()),
                    }
                    (_, _) => return Err("Operands must be a numbers.".to_string())
                }
            })
        },
        Expression::Variable(name) => {
            if let Some(variables) = variables {
                if let Some(variable) = variables.get(&name) {
                    match variable {
                        Value::Bool(boolean) => Ok(Literal::Bool(*boolean)),
                        Value::Number(number) => Ok(Literal::Number(*number)),
                        Value::String(string) => Ok(Literal::String(string.clone())),
                        Value::None => Ok(Literal::None),
                    }
                } else {
                    Err(format!("Undefined variable '{}'.", name))
                }

            } else {
                Err(format!("Undefined variable '{}'.", name))
            }
        },
        _ => unreachable!()
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;
    use crate::interpreter::{evaluate, Value};
    use crate::syntax::parser::Parser;
    use crate::syntax::tokenizer::Scanner;

    fn run(source: &str) -> Result<Value, String> {
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        evaluate(parser.parse_expression()?, None)
    }

    #[rstest]
    #[case("true", "true")]
    #[case("false", "false")]
    #[case("nil", "nil")]
    fn test_evaluate_booleans_and_nil(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(expected, run(input).unwrap().to_string());
    }

    #[rstest]
    #[case("\"hello world!\"", "hello world!")]
    #[case("\"foo!\"", "foo!")]
    #[case("\"hello\non\nthe\nother\nside\"", "hello\non\nthe\nother\nside")]
    fn test_evaluate_string(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(expected, run(input).unwrap().to_string());
    }

    #[rstest]
    #[case("10.40", "10.4")]
    #[case("10.41", "10.41")]
    #[case("54.12300", "54.123")]
    fn test_evaluate_float(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(expected, run(input).unwrap().to_string());
    }

    #[rstest]
    #[case("10", "10")]
    #[case("123", "123")]
    #[case("54", "54")]
    fn test_evaluate_integer(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(expected, run(input).unwrap().to_string());
    }

    #[rstest]
    #[case("(\"hello world!\")", "hello world!")]
    #[case("((\"hello world!\"))", "hello world!")]
    #[case("(true)", "true")]
    #[case("(10.40)", "10.4")]
    #[case("((false))", "false")]
    fn test_evaluate_group(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(expected, run(input).unwrap().to_string());
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
        assert_eq!(expected, run(input).unwrap().to_string());
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
        assert_eq!(expected, run(input).unwrap().to_string());
    }

    #[rstest]
    #[case("\"hello\" + \" world!\"", "hello world!")]
    #[case("\"foo\" + \"bar\"", "foobar")]
    #[case("\"42\" + \"24\"", "4224")]
    fn test_evaluate_string_concatenation(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(expected, run(input).unwrap().to_string());
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
        assert_eq!(expected, run(input).unwrap().to_string());
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
    fn test_evaluate_equality_equals(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(expected, run(input).unwrap().to_string());
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
        assert_eq!(expected, run(input).unwrap().to_string());
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
        assert_eq!(expected, run(input).err().unwrap());
    }
}