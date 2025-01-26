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
    for statement in statements {
        match statement {
            Statement::Print(expression) => println!("{}", evaluate(expression)?),
            Statement::Expression(expression) => {
                evaluate(expression)?;
            }
        }
    }

    Ok(())
}

pub fn evaluate(expression: Expression) -> Result<Value, String> {
    let result = evaluate_expression(expression);
    if let Ok(literal) = result {
        Ok(Value::from_literal(literal))
    } else {
        Err(result.err().unwrap())
    }

}

fn evaluate_expression(expression: Expression) -> Result<Literal, String> {
    match expression {
        Expression::Literal(literal) => Ok(literal),
        Expression::Grouping(expression) => evaluate_expression(*expression),
        Expression::Unary(operation, expression) => {
            match operation {
                UnaryOperation::Minus => match evaluate_expression(*expression)? {
                    Literal::Number(number) => Ok(Literal::Number(-number)),
                    _ => Err("Operand must be a number.".to_string()),
                },
                UnaryOperation::Not => Ok(Literal::Bool(!evaluate_expression(*expression)?.is_truthy())),
            }
        },
        Expression::Binary(operation, left, right) => {
            let (left, right) = (evaluate_expression(*left)?, evaluate_expression(*right)?);

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
    }
}

#[cfg(test)]
mod tests {
    use std::io::stdout;
    use crate::interpreter::evaluate;
    use crate::syntax::parser::Parser;
    use crate::syntax::tokenizer::Scanner;

    #[test]
    fn test_evaluate_bool_true() {
        let source = "true";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let result = evaluate(parser.parse_expression().unwrap());
        assert_eq!(result.unwrap().to_string(), "true");
    }

    #[test]
    fn test_evaluate_bool_false() {
        let source = "false";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let result = evaluate(parser.parse_expression().unwrap());
        assert_eq!(result.unwrap().to_string(), "false");
    }

    #[test]
    fn test_evaluate_nil() {
        let source = "nil";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let result = evaluate(parser.parse_expression().unwrap());
        assert_eq!(result.unwrap().to_string(), "nil");
    }

    #[test]
    fn test_evaluate_string() {
        let source = "\"hello world!\"";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let result = evaluate(parser.parse_expression().unwrap());
        assert_eq!(result.unwrap().to_string(), "hello world!");
    }

    #[test]
    fn test_evaluate_float() {
        let source = "10.40";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let result = evaluate(parser.parse_expression().unwrap());
        assert_eq!(result.unwrap().to_string(), "10.4");
    }

    #[test]
    fn test_evaluate_integer() {
        let source = "10";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let result = evaluate(parser.parse_expression().unwrap());
        assert_eq!(result.unwrap().to_string(), "10");
    }

    #[test]
    fn test_evaluate_group_string() {
        let source = "(\"hello world!\")";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let result = evaluate(parser.parse_expression().unwrap());
        assert_eq!(result.unwrap().to_string(), "hello world!");
    }

    #[test]
    fn test_evaluate_group_bool_true() {
        let source = "(true)";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let result = evaluate(parser.parse_expression().unwrap());
        assert_eq!(result.unwrap().to_string(), "true");
    }

    #[test]
    fn test_evaluate_group_number() {
        let source = "(10.40)";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let result = evaluate(parser.parse_expression().unwrap());
        assert_eq!(result.unwrap().to_string(), "10.4");
    }

    #[test]
    fn test_evaluate_group_group_bool_false() {
        let source = "((false))";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let result = evaluate(parser.parse_expression().unwrap());
        assert_eq!(result.unwrap().to_string(), "false");
    }

    #[test]
    fn test_evaluate_unary_minus_number() {
        let source = "-73";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let result = evaluate(parser.parse_expression().unwrap());
        assert_eq!(result.unwrap().to_string(), "-73");
    }

    #[test]
    fn test_evaluate_unary_not_bool() {
        let source = "!true";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let result = evaluate(parser.parse_expression().unwrap());
        assert_eq!(result.unwrap().to_string(), "false");
    }
    #[test]
    fn test_evaluate_unary_not_nil() {
        let source = "!nil";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let result = evaluate(parser.parse_expression().unwrap());
        assert_eq!(result.unwrap().to_string(), "true");
    }

    #[test]
    fn test_evaluate_unary_not_number() {
        let source = "!10.40";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let result = evaluate(parser.parse_expression().unwrap());
        assert_eq!(result.unwrap().to_string(), "false");
    }

    #[test]
    fn test_evaluate_unary_group_not_bool() {
        let source = "!((false))";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let result = evaluate(parser.parse_expression().unwrap());
        assert_eq!(result.unwrap().to_string(), "true");
    }

    #[test]
    fn test_evaluate_arithmetic_division() {
        let source = "42 / 5";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let result = evaluate(parser.parse_expression().unwrap());
        assert_eq!(result.unwrap().to_string(), "8.4");
    }

    #[test]
    fn test_evaluate_arithmetic_multiplication_and_division() {
        let source = "18 * 3 / (3 * 6)";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let result = evaluate(parser.parse_expression().unwrap());
        assert_eq!(result.unwrap().to_string(), "3");
    }

    #[test]
    fn test_evaluate_arithmetic_multiplication_and_division_2() {
        let source = "(10.40 * 2) / 2";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let result = evaluate(parser.parse_expression().unwrap());
        assert_eq!(result.unwrap().to_string(), "10.4");
    }

    #[test]
    fn test_evaluate_arithmetic_minus() {
        let source = "70 - 65";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let result = evaluate(parser.parse_expression().unwrap());
        assert_eq!(result.unwrap().to_string(), "5");
    }

    #[test]
    fn test_evaluate_arithmetic_minus_negative() {
        let source = "69 - 93";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let result = evaluate(parser.parse_expression().unwrap());
        assert_eq!(result.unwrap().to_string(), "-24");
    }

    #[test]
    fn test_evaluate_arithmetic_minus_float() {
        let source = "10.40 - 2";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let result = evaluate(parser.parse_expression().unwrap());
        assert_eq!(result.unwrap().to_string(), "8.4");
    }

    #[test]
    fn test_evaluate_arithmetic_plus_minus_group() {
        let source = "23 + 28 - (-(61 - 99))";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let result = evaluate(parser.parse_expression().unwrap());
        assert_eq!(result.unwrap().to_string(), "13");
    }

    #[test]
    fn test_evaluate_string_concatenation() {
        let source = "\"hello\" + \" world!\"";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let result = evaluate(parser.parse_expression().unwrap());
        assert_eq!(result.unwrap().to_string(), "hello world!");
    }

    #[test]
    fn test_evaluate_string_concatenation_2() {
        let source = "\"42\" + \"24\"";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let result = evaluate(parser.parse_expression().unwrap());
        assert_eq!(result.unwrap().to_string(), "4224");
    }

    #[test]
    fn test_evaluate_string_concatenation_3() {
        let source = "\"foo\" + \"bar\"";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let result = evaluate(parser.parse_expression().unwrap());
        assert_eq!(result.unwrap().to_string(), "foobar");
    }

    #[test]
    fn test_evaluate_relational_greater() {
        let source = "57 > -65";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let result = evaluate(parser.parse_expression().unwrap());
        assert_eq!(result.unwrap().to_string(), "true");
    }

    #[test]
    fn test_evaluate_relational_greater_equal() {
        let source = "11 >= 11";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let result = evaluate(parser.parse_expression().unwrap());
        assert_eq!(result.unwrap().to_string(), "true");
    }

    #[test]
    fn test_evaluate_relational_greater_equal_group() {
        let source = "(54 - 67) >= -(114 / 57 + 11)";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let result = evaluate(parser.parse_expression().unwrap());
        assert_eq!(result.unwrap().to_string(), "true");
    }

    #[test]
    fn test_evaluate_relational_less() {
        let source = "57 < -65";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let result = evaluate(parser.parse_expression().unwrap());
        assert_eq!(result.unwrap().to_string(), "false");
    }

    #[test]
    fn test_evaluate_relational_less_equal() {
        let source = "11 <= 11";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let result = evaluate(parser.parse_expression().unwrap());
        assert_eq!(result.unwrap().to_string(), "true");
    }

    #[test]
    fn test_evaluate_equality_equals() {
        let source = "\"hello\" == \"world\"";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let result = evaluate(parser.parse_expression().unwrap());
        assert_eq!(result.unwrap().to_string(), "false");
    }

    #[test]
    fn test_evaluate_equality_equals_2() {
        let source = "\"foo\" == \"foo\"";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let result = evaluate(parser.parse_expression().unwrap());
        assert_eq!(result.unwrap().to_string(), "true");
    }

    #[test]
    fn test_evaluate_equality_equals_3() {
        let source = "true == true";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let result = evaluate(parser.parse_expression().unwrap());
        assert_eq!(result.unwrap().to_string(), "true");
    }

    #[test]
    fn test_evaluate_equality_equals_4() {
        let source = "5 == 5";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let result = evaluate(parser.parse_expression().unwrap());
        assert_eq!(result.unwrap().to_string(), "true");
    }

    #[test]
    fn test_evaluate_equality_equals_5() {
        let source = "nil == nil";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let result = evaluate(parser.parse_expression().unwrap());
        assert_eq!(result.unwrap().to_string(), "true");
    }

    #[test]
    fn test_evaluate_equality_not_equals() {
        let source = "\"foo\" != \"bar\"";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let result = evaluate(parser.parse_expression().unwrap());
        assert_eq!(result.unwrap().to_string(), "true");
    }

    #[test]
    fn test_evaluate_equality_equals_string_and_number() {
        let source = "61 == \"61\"";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let result = evaluate(parser.parse_expression().unwrap());
        assert_eq!(result.unwrap().to_string(), "false");
    }

    #[test]
    fn test_evaluate_runtime_error_unary_minus() {
        let source = "-\"foo\"";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let result = evaluate(parser.parse_expression().unwrap());
        assert_eq!(result.err().unwrap(), "Operand must be a number.");
    }

    #[test]
    fn test_evaluate_runtime_error_unary_minus_2() {
        let source = "-false";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let result = evaluate(parser.parse_expression().unwrap());
        assert_eq!(result.err().unwrap(), "Operand must be a number.");
    }

    #[test]
    fn test_evaluate_runtime_error_unary_minus_3() {
        let source = "-(\"foo\" + \"bar\")";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let result = evaluate(parser.parse_expression().unwrap());
        assert_eq!(result.err().unwrap(), "Operand must be a number.");
    }

    #[test]
    fn test_evaluate_runtime_error_unary_minus_4() {
        let source = "-false";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let result = evaluate(parser.parse_expression().unwrap());
        assert_eq!(result.err().unwrap(), "Operand must be a number.");
    }

    #[test]
    fn test_evaluate_runtime_error_binary_multiply() {
        let source = "\"foo\" * 42";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let result = evaluate(parser.parse_expression().unwrap());
        assert_eq!(result.err().unwrap(), "Operands must be a numbers.");
    }

    #[test]
    fn test_evaluate_runtime_error_binary_multiply_2() {
        let source = "(\"foo\" * \"bar\")";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let result = evaluate(parser.parse_expression().unwrap());
        assert_eq!(result.err().unwrap(), "Operands must be a numbers.");
    }

    #[test]
    fn test_evaluate_runtime_error_binary_divide() {
        let source = "true / 2";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let result = evaluate(parser.parse_expression().unwrap());
        assert_eq!(result.err().unwrap(), "Operands must be a numbers.");
    }

    #[test]
    fn test_evaluate_runtime_error_binary_divide_2() {
        let source = "false / true";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let result = evaluate(parser.parse_expression().unwrap());
        assert_eq!(result.err().unwrap(), "Operands must be a numbers.");
    }

    #[test]
    fn test_evaluate_runtime_error_binary_plus() {
        let source = "\"foo\" + true";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let result = evaluate(parser.parse_expression().unwrap());
        assert_eq!(result.err().unwrap(), "Operands must be a numbers.");
    }

    #[test]
    fn test_evaluate_runtime_error_binary_plus_2() {
        let source = "42 - true";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let result = evaluate(parser.parse_expression().unwrap());
        assert_eq!(result.err().unwrap(), "Operands must be a numbers.");
    }

    #[test]
    fn test_evaluate_runtime_error_binary_plus_3() {
        let source = "true + false";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let result = evaluate(parser.parse_expression().unwrap());
        assert_eq!(result.err().unwrap(), "Operands must be a numbers.");
    }

    #[test]
    fn test_evaluate_runtime_error_binary_plus_4() {
        let source = "\"foo\" - \"bar\"";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let result = evaluate(parser.parse_expression().unwrap());
        assert_eq!(result.err().unwrap(), "Operands must be a numbers.");
    }

    #[test]
    fn test_evaluate_runtime_error_binary_greater() {
        let source = "\"foo\" < false";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let result = evaluate(parser.parse_expression().unwrap());
        assert_eq!(result.err().unwrap(), "Operands must be a numbers.");
    }

    #[test]
    fn test_evaluate_runtime_error_binary_greater_2() {
        let source = "true < 2";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let result = evaluate(parser.parse_expression().unwrap());
        assert_eq!(result.err().unwrap(), "Operands must be a numbers.");
    }

    #[test]
    fn test_evaluate_runtime_error_binary_greater_3() {
        let source = "(\"foo\" + \"bar\") < 42";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let result = evaluate(parser.parse_expression().unwrap());
        assert_eq!(result.err().unwrap(), "Operands must be a numbers.");
    }

    #[test]
    fn test_evaluate_runtime_error_binary_less() {
        let source = "false > true";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let result = evaluate(parser.parse_expression().unwrap());
        assert_eq!(result.err().unwrap(), "Operands must be a numbers.");
    }

    #[test]
    fn test_evaluate_runtime_error_binary_greater_equal() {
        let source = "\"foo\" <= false";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let result = evaluate(parser.parse_expression().unwrap());
        assert_eq!(result.err().unwrap(), "Operands must be a numbers.");
    }

    #[test]
    fn test_evaluate_runtime_error_binary_less_equal() {
        let source = "\"foo\" >= false";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let result = evaluate(parser.parse_expression().unwrap());
        assert_eq!(result.err().unwrap(), "Operands must be a numbers.");
    }
}