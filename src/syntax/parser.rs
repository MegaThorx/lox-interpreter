use crate::syntax::expression::{BinaryOperation, Expression, Literal, UnaryOperation};
use crate::syntax::statement::Statement;
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

    pub fn parse(&mut self) -> Result<Vec<Statement>, String> {
        let mut statements = Vec::<Statement>::new();

        while !self.check(TokenType::Eof) {
            statements.push(self.parse_statement()?);
        }

        Ok(statements)
    }

    fn parse_statement(&mut self) -> Result<Statement, String> {
        let statement = if matches!(self, TokenType::Print) {
            let expression = self.parse_expression()?;

            if !self.check(TokenType::Semicolon) {
                return Err(format!("[line {}] Expect ';' after expression.", self.current().line));
            }

            self.advance();

            Statement::Print(expression)
        } else if matches!(self, TokenType::Var) {
            let token = self.consume();

            if let TokenType::Identifier(name) = token.token {
                let mut expression: Option<Expression> = None;
                if matches!(self, TokenType::Equal) {
                    expression = Some(self.parse_expression()?);
                }

                if !self.check(TokenType::Semicolon) {
                    return Err(format!("[line {}] Expect ';' after value.", self.current().line));
                }

                self.advance();

                Statement::Variable(name.to_string(), expression)
            } else {
                return Err(format!("[line {}] Expect variable name.", token.line));
            }
        } else if matches!(self, TokenType::LeftBrace) {
            let mut statements: Vec<Statement> = Vec::new();

            while !self.check(TokenType::RightBrace) && !self.is_at_end() {
                statements.push(self.parse_statement()?);
            }

            if !self.check(TokenType::RightBrace) {
                return Err(format!("[line {}] Expect '{}' after block.", self.current().line, '}'));
            }

            self.advance();

            Statement::Block(statements)
        } else if matches!(self, TokenType::If) {
            if !self.check(TokenType::LeftParen) {
                return Err(format!("[line {}] Expect '(' after if.", self.current().line));
            }
            self.advance();

            let expression = self.parse_expression()?;

            if !self.check(TokenType::RightParen) {
                return Err(format!("[line {}] Expect ')' after if condition.", self.current().line));
            }
            self.advance();

            let if_body = self.parse_statement()?;
            let mut else_body: Option<Box<Statement>> = None;

            if matches!(self, TokenType::Else) {
                else_body = Some(Box::new(self.parse_statement()?));
            }

            Statement::If(expression, Box::new(if_body), else_body)
        } else {
            let expression = self.parse_expression()?;

            if !self.check(TokenType::Semicolon) {
                return Err(format!("[line {}] Expect ';' after value.", self.current().line));
            }

            self.advance();

            Statement::Expression(expression)
        };

        Ok(statement)
    }

    pub fn parse_expression(&mut self) -> Result<Expression, String> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Result<Expression, String> {
        let mut expression = self.parse_equality()?;

        while matches!(self, TokenType::Equal) {
            expression = match expression {
                Expression::Variable(name) => Expression::Assign(name, Box::new(self.parse_expression()?)),
                _ => {
                    return Err("Invalid assignment target.".to_string());
                }
            }
        }

        Ok(expression)
    }

    fn parse_equality(&mut self) -> Result<Expression, String> {
        let mut expression = self.parse_comparison()?;

        while matches!(self, TokenType::EqualEqual, TokenType::BangEqual) {
            expression = match self.previous().token {
                TokenType::EqualEqual => Expression::Binary(BinaryOperation::Equal, Box::new(expression), Box::new(self.parse_comparison()?)),
                _ => Expression::Binary(BinaryOperation::NotEqual, Box::new(expression), Box::new(self.parse_comparison()?)), // Last one can only be BangEqual
            }
        }

        Ok(expression)
    }

    fn parse_comparison(&mut self) -> Result<Expression, String> {
        let mut expression = self.parse_term()?;

        while matches!(self, TokenType::Greater, TokenType::GreaterEqual, TokenType::Less, TokenType::LessEqual) {
            expression = match self.previous().token {
                TokenType::Greater => Expression::Binary(BinaryOperation::Greater, Box::new(expression), Box::new(self.parse_term()?)),
                TokenType::GreaterEqual => Expression::Binary(BinaryOperation::GreaterEqual, Box::new(expression), Box::new(self.parse_term()?)),
                TokenType::Less => Expression::Binary(BinaryOperation::Less, Box::new(expression), Box::new(self.parse_term()?)),
                _ => Expression::Binary(BinaryOperation::LessEqual, Box::new(expression), Box::new(self.parse_term()?)), // Last one can only be LessEqual
            }
        }

        Ok(expression)
    }

    fn parse_term(&mut self) -> Result<Expression, String> {
        let mut expression = self.parse_factor()?;

        while matches!(self, TokenType::Plus, TokenType::Minus) {
            expression = match self.previous().token {
                TokenType::Plus => Expression::Binary(BinaryOperation::Plus, Box::new(expression), Box::new(self.parse_factor()?)),
                _ => Expression::Binary(BinaryOperation::Minus, Box::new(expression), Box::new(self.parse_factor()?)), // Last one can only be Minus
            }
        }

        Ok(expression)
    }

    fn parse_factor(&mut self) -> Result<Expression, String> {
        let mut expression = self.parse_unary()?;

        while matches!(self, TokenType::Star, TokenType::Slash) {
            expression = match self.previous().token {
                TokenType::Star => Expression::Binary(BinaryOperation::Multiply, Box::new(expression), Box::new(self.parse_unary()?)),
                _ => Expression::Binary(BinaryOperation::Divide, Box::new(expression), Box::new(self.parse_unary()?)), // Last one can only be Slash
            }
        }

        Ok(expression)
    }

    fn parse_unary(&mut self) -> Result<Expression, String> {
        if matches!(self, TokenType::Minus, TokenType::Bang) {
            return Ok(match self.previous().token {
                TokenType::Minus => Expression::Unary(UnaryOperation::Minus, Box::new(self.parse_unary()?)),
                _ => Expression::Unary(UnaryOperation::Not, Box::new(self.parse_unary()?)), // Last one can only be Bang
            });
        }

        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expression, String> {
        let token = self.consume();
        match token.token {
            TokenType::True => Ok(Expression::Literal(Literal::Bool(true))),
            TokenType::False => Ok(Expression::Literal(Literal::Bool(false))),
            TokenType::Number(number) => Ok(Expression::Literal(Literal::Number(number))),
            TokenType::String(string) => Ok(Expression::Literal(Literal::String(string.to_string()))),
            TokenType::Nil => Ok(Expression::Literal(Literal::None)),
            TokenType::Identifier(name) => Ok(Expression::Variable(name.to_string())),
            TokenType::LeftParen => {
                let expression = self.parse_expression()?;

                if !self.check(TokenType::RightParen) {
                    let token = self.current();
                    return Err(match token.token {
                        TokenType::Eof => format!("[line {}] Error at end: Expect expression.", token.line),
                        _ => format!("[line {}] Error at '{}': Expect expression.", token.line, token.lexeme)
                    });
                }

                self.advance();

                Ok(Expression::Grouping(Box::new(expression)))
            },
            _ => Err(match token.token {
                TokenType::Eof => format!("[line {}] Error at end: Expect expression.", token.line),
                _ => format!("[line {}] Error at '{}': Expect expression.", token.line, token.lexeme)
            }),
        }
    }

    fn consume(&mut self) -> &Token<'a> {
        self.advance();
        &self.tokens[self.current - 1]
    }

    fn previous(&self) -> &Token<'a> {
        &self.tokens[self.current - 1]
    }

    fn current(&self) -> &Token<'a> {
        &self.tokens[self.current]
    }

    fn check(&self, token_type: TokenType) -> bool {
        self.current().token == token_type
    }

    fn advance(&mut self) {
        self.current += 1;
    }

    fn is_at_end(&self) -> bool {
        self.check(TokenType::Eof)
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;
    use crate::syntax::expression::Expression;
    use crate::syntax::parser::Parser;
    use crate::syntax::tokenizer::Scanner;

    fn run_expression(source: &str) -> Result<Expression, String> {
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        parser.parse_expression()
    }

    fn run_statement(source: &str) -> Result<String, String> {
        let mut scanner = Scanner::new(source);
        let (tokens, _) = scanner.scan_tokens();
        let mut parser = Parser::new(tokens);
        let statements = parser.parse()?;
        Ok(statements.iter().map(|statement| statement.to_string()).collect::<Vec<String>>().join(" "))
    }

    #[rstest]
    #[case("true", "true")]
    #[case("false", "false")]
    #[case("nil", "nil")]
    fn test_parser_booleans_and_nil(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(expected, run_expression(input).unwrap().to_string());
    }

    #[rstest]
    #[case("123", "123.0")]
    #[case("123.123", "123.123")]
    #[case("32453454", "32453454.0")]
    #[case("32453454.32453454000", "32453454.32453454")]
    fn test_parser_numbers(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(expected, run_expression(input).unwrap().to_string());
    }

    #[rstest]
    #[case("\"test\"", "test")]
    #[case("(\"foo\")", "(group foo)")]
    #[case("((\"foo\"))", "(group (group foo))")]
    fn test_parser_string(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(expected, run_expression(input).unwrap().to_string());
    }

    #[rstest]
    #[case("!false", "(! false)")]
    #[case("!\"foo\")", "(! foo)")]
    #[case("!123", "(! 123.0)")]
    fn test_parser_unary_not(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(expected, run_expression(input).unwrap().to_string());
    }

    #[rstest]
    #[case("-4", "(- 4.0)")]
    fn test_parser_unary_minus(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(expected, run_expression(input).unwrap().to_string());
    }

    #[rstest]
    #[case("16 * 38 / 58", "(/ (* 16.0 38.0) 58.0)")]
    #[case("(15 * -78 / (15 * 40))", "(group (/ (* 15.0 (- 78.0)) (group (* 15.0 40.0))))")]
    #[case("(1 / 2) * (-3 / -2)", "(* (group (/ 1.0 2.0)) (group (/ (- 3.0) (- 2.0))))")]
    #[case("52 + 80 - 94", "(- (+ 52.0 80.0) 94.0)")]
    #[case("(1 + 2) * (-3 - -2)", "(* (group (+ 1.0 2.0)) (group (- (- 3.0) (- 2.0))))")]
    fn test_parser_arithmetic(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(expected, run_expression(input).unwrap().to_string());
    }

    #[rstest]
    #[case("83 < 99 < 115", "(< (< 83.0 99.0) 115.0)")]
    #[case("83 > 99 > 115", "(> (> 83.0 99.0) 115.0)")]
    #[case("83 >= 99 <= 115", "(<= (>= 83.0 99.0) 115.0)")]
    fn test_parser_comparison(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(expected, run_expression(input).unwrap().to_string());
    }

    #[rstest]
    #[case("\"baz\" == \"baz\"", "(== baz baz)")]
    #[case("\"baz\" != \"baz\"", "(!= baz baz)")]
    fn test_parser_equal(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(expected, run_expression(input).unwrap().to_string());
    }
    
    #[rstest]
    #[case("(72 +)", "[line 1] Error at ')': Expect expression.")]
    #[case("(72 +", "[line 1] Error at end: Expect expression.")]
    #[case("(72 + 42", "[line 1] Error at end: Expect expression.")]
    #[case("(72 }", "[line 1] Error at '}': Expect expression.")]
    fn test_parser_syntax_error(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(expected, run_expression(input).err().unwrap());
    }

    #[rstest]
    #[case("print \"hello world\";", "(print (; hello world))")]
    #[case("print 123.1;", "(print (; 123.1))")]
    #[case("print nil;", "(print (; nil))")]
    #[case("print true;", "(print (; true))")]
    #[case("print test;", "(print (; (variable test)))")]
    fn test_parser_statement_print(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(expected, run_statement(input).unwrap());
    }

    #[rstest]
    #[case("var test = \"hello world\";", "(var test = (; hello world))")]
    #[case("var test = 123.1;", "(var test = (; 123.1))")]
    #[case("var test = nil;", "(var test = (; nil))")]
    #[case("var test = true;", "(var test = (; true))")]
    #[case("var test = test;", "(var test = (; (variable test)))")]
    #[case("test = test;", "(; (assign test (variable test)))")]
    #[case("var test;", "(var test)")]
    fn test_parser_statement_variable(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(expected, run_statement(input).unwrap());
    }

    #[rstest]
    #[case("\"hello world\";", "(; hello world)")]
    #[case("123.1;", "(; 123.1)")]
    #[case("nil;", "(; nil)")]
    #[case("true;", "(; true)")]
    #[case("test;", "(; (variable test))")]
    fn test_parser_statement_expression(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(expected, run_statement(input).unwrap());
    }

    #[rstest]
    #[case("{\"hello world\";}", "(block ((; hello world)))")]
    #[case("{123.1;}", "(block ((; 123.1)))")]
    #[case("{nil;}", "(block ((; nil)))")]
    #[case("{true;}", "(block ((; true)))")]
    #[case("{test;}", "(block ((; (variable test))))")]
    fn test_parser_statement_block(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(expected, run_statement(input).unwrap());
    }

    #[rstest]
    #[case("print \"hello world\"", "[line 1] Expect ';' after expression.")]
    #[case("var test = 1", "[line 1] Expect ';' after value.")]
    #[case("var test = (", "[line 1] Error at end: Expect expression.")]
    #[case("var", "[line 1] Expect variable name.")]
    #[case("{", "[line 1] Expect '}' after block.")]
    #[case("1 + 1", "[line 1] Expect ';' after value.")]
    #[case("2 = 1", "Invalid assignment target.")]
    fn test_parser_statement_error(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(expected, run_statement(input).err().unwrap());
    }
}