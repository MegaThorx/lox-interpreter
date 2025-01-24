use crate::syntax::expression::{Expression, Literal, UnaryOperation};
use crate::syntax::token::{Token, TokenType};

pub struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    current: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> Self {
        Parser {
            tokens,
            current: 0,
        }
    }

    pub fn parse_expression(&mut self) -> Expression {
        let token = &self.tokens[self.current];
        self.current += 1;

        match token.token {
            TokenType::True => Expression::Literal(Literal::Bool(true)),
            TokenType::False => Expression::Literal(Literal::Bool(false)),
            TokenType::Number(number)  => Expression::Literal(Literal::Number(number)),
            TokenType::String(string) => Expression::Literal(Literal::String(string)),
            TokenType::Nil => Expression::Literal(Literal::None),
            TokenType::LeftParen => Expression::Grouping(Box::new(self.parse_expression())),
            TokenType::Minus => Expression::Unary(UnaryOperation::Minus, Box::new(self.parse_expression())),
            TokenType::Bang => Expression::Unary(UnaryOperation::Not, Box::new(self.parse_expression())),
            _ => panic!("Not implemented"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::syntax::parser::Parser;
    use crate::syntax::tokenizer::Scanner;

    #[test]
    fn test_parser_booleans_true() {
        let source = "true";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        assert_eq!(parser.parse_expression().to_string(), "true");
    }

    #[test]
    fn test_parser_booleans_false() {
        let source = "false";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        assert_eq!(parser.parse_expression().to_string(), "false");
    }

    #[test]
    fn test_parser_nil() {
        let source = "nil";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        assert_eq!(parser.parse_expression().to_string(), "nil");
    }

    #[test]
    fn test_parser_number() {
        let source = "123";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        assert_eq!(parser.parse_expression().to_string(), "123.0");
    }

    #[test]
    fn test_parser_number_with_decimals() {
        let source = "123.123";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        assert_eq!(parser.parse_expression().to_string(), "123.123");
    }

    #[test]
    fn test_parser_string() {
        let source = "\"test\"test";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        assert_eq!(parser.parse_expression().to_string(), "test");
    }

    #[test]
    fn test_parser_group_simple() {
        let source = "(\"foo\")";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        assert_eq!(parser.parse_expression().to_string(), "(group foo)");
    }

    #[test]
    fn test_parser_group_multiple() {
        let source = "((\"foo\"))";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        assert_eq!(parser.parse_expression().to_string(), "(group (group foo))");
    }

    #[test]
    fn test_unary_operator_not() {
        let source = "!false";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        assert_eq!(parser.parse_expression().to_string(), "(! false)");
    }

    #[test]
    fn test_unary_operator_minus() {
        let source = "-4";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        assert_eq!(parser.parse_expression().to_string(), "(- 4.0)");
    }
}