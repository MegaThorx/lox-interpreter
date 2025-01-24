use crate::syntax::expression::{Expression, Literal};
use crate::syntax::token::{Token, TokenType};

pub struct Parser<'a> {
    tokens: Vec<Token<'a>>
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> Self {
        Parser {
            tokens
        }
    }

    pub fn parse_expression(&mut self) -> Expression {
        let token = self.tokens.first().unwrap();

        match token.token {
            TokenType::True => Expression::Literal(Literal::Bool(true)),
            TokenType::False => Expression::Literal(Literal::Bool(false)),
            TokenType::Number(number)  => Expression::Literal(Literal::Number(number)),
            TokenType::String(string) => Expression::Literal(Literal::String(string)),
            TokenType::Nil => Expression::Literal(Literal::None),
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
}