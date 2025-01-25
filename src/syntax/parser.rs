use crate::syntax::expression::{BinaryOperation, Expression, Literal, UnaryOperation};
use crate::syntax::token::{Token, TokenType};

pub struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    current: usize,
}

macro_rules! matches {
    ($self : ident, $( $x : expr),*) => {
        {
            if $( $self.check($x) )||* {
                $self.advance();
                true
            } else {
                false
            }
        }
    };
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> Self {
        Parser {
            tokens,
            current: 0,
        }
    }

    pub fn parse_expression(&mut self) -> Expression {
        self.parse_factor()
    }

    fn parse_factor(&mut self) -> Expression {
        let mut expression = self.parse_unary();

        while matches!(self, TokenType::Star, TokenType::Slash) {
            match self.previous().unwrap().token {
                TokenType::Star => {
                    expression = Expression::Binary(BinaryOperation::Multiply, Box::new(expression), Box::new(self.parse_unary()));
                },
                TokenType::Slash => {
                    expression = Expression::Binary(BinaryOperation::Divide, Box::new(expression), Box::new(self.parse_unary()));
                },
                _ => break,
            }
        }

        expression
    }

    fn parse_unary(&mut self) -> Expression {
        if matches!(self, TokenType::Minus, TokenType::Bang) {
            match self.previous().unwrap().token {
                TokenType::Minus => {
                    return Expression::Unary(UnaryOperation::Minus, Box::new(self.parse_unary()))
                },
                TokenType::Bang => {
                    return Expression::Unary(UnaryOperation::Not, Box::new(self.parse_unary()))
                },
                _ => unreachable!(),
            }
        }

        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Expression {
        let token = self.consume();
        match token.token {
            TokenType::True => Expression::Literal(Literal::Bool(true)),
            TokenType::False => Expression::Literal(Literal::Bool(false)),
            TokenType::Number(number)  => Expression::Literal(Literal::Number(number)),
            TokenType::String(string) => Expression::Literal(Literal::String(string.to_string())),
            TokenType::Nil => Expression::Literal(Literal::None),
            TokenType::LeftParen => {
                let expression = self.parse_expression();

                if !self.check(TokenType::RightParen) {
                    panic!("Expected right parenthesis");
                }
                
                self.advance();

                Expression::Grouping(Box::new(expression))
            },
            _ => panic!("Not implemented {}", token.token),
        }
    }
    
    fn consume(&mut self) -> &Token<'a> {
        self.advance();
        &self.tokens[self.current - 1]
    }

    fn previous(&self) -> Option<&Token<'a>> {
        match self.current > 0 {
            true => Some(&self.tokens[self.current - 1]),
            _ => None
        }
    }

    fn check(&self, token_type: TokenType) -> bool {
        self.tokens[self.current].token == token_type
    }

    fn advance(&mut self) {
        self.current += 1;
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
    fn test_parser_unary_operator_not() {
        let source = "!false";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        assert_eq!(parser.parse_expression().to_string(), "(! false)");
    }

    #[test]
    fn test_parser_unary_operator_minus() {
        let source = "-4";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        assert_eq!(parser.parse_expression().to_string(), "(- 4.0)");
    }

    #[test]
    fn test_parser_arithmetic_operator_multiply_and_division() {
        let source = "16 * 38 / 58";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        assert_eq!(parser.parse_expression().to_string(), "(/ (* 16.0 38.0) 58.0)");
    }

    #[test]
    fn test_parser_arithmetic_operator_multiply_and_division_complex() {
        let source = "(15 * -78 / (15 * 40))";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        assert_eq!(parser.parse_expression().to_string(), "(group (/ (* 15.0 (- 78.0)) (group (* 15.0 40.0))))");
    }

    #[test]
    fn test_parser_arithmetic_operator_multiply_and_division_complex_2() {
        let source = "(1 / 2) * (-3 / -2)";
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        assert_eq!(parser.parse_expression().to_string(), "(* (group (/ 1.0 2.0)) (group (/ (- 3.0) (- 2.0))))");
    }
}