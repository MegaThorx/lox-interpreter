use crate::expression::{BinaryOperation, Expression, Literal, UnaryOperation};
use crate::statement::Statement;
use crate::token::{Token, TokenType};

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
            statements.push(self.parse_declaration()?);
        }

        Ok(statements)
    }

    fn parse_declaration(&mut self) -> Result<Statement, String> {
        if matches!(self, TokenType::Fun) {
            self.parse_function_declaration("function")
        } else {
            self.parse_variable_declaration()
        }
    }

    fn parse_function_declaration(&mut self, kind: &str) -> Result<Statement, String> {
        let token = self.consume();

        let identifier = match token.token {
            TokenType::Identifier(identifier) => identifier.to_string(),
            _ => return Err(format!("[line {}] Expect {} name.", self.current().line, kind)),
        };

        if !self.check(TokenType::LeftParen) {
            return Err(format!("[line {}] Expect '(' after {} name.", self.current().line, kind));
        }
        self.advance();

        let mut parameters: Vec<String> = Vec::new();

        if !self.check(TokenType::RightParen) {
            loop {
                if parameters.len() >= 255 {
                    return Err(format!("[line {}] Can't have more than 255 parameters.", self.current().line));
                }

                let token = self.consume();

                let identifier = match token.token {
                    TokenType::Identifier(identifier) => identifier.to_string(),
                    _ => return Err(format!("[line {}] Expect parameter name.", self.current().line)),
                };

                parameters.push(identifier);

                if !matches!(self, TokenType::Comma) {
                    break;
                }
            }
        }

        if !self.check(TokenType::RightParen) {
            return Err(format!("[line {}] Expect ')' after parameters.", self.current().line));
        }
        self.advance();

        let body = self.parse_statement()?;
        Ok(Statement::Function(identifier, parameters, Box::new(body)))
    }

    fn parse_variable_declaration(&mut self) -> Result<Statement, String> {
        if matches!(self, TokenType::Var) {
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

                Ok(Statement::Variable(name.to_string(), expression))
            } else {
                Err(format!("[line {}] Expect variable name.", token.line))
            }
        } else {
            self.parse_statement()
        }
    }

    fn parse_statement(&mut self) -> Result<Statement, String> {
        let statement = if matches!(self, TokenType::Print) {
            let expression = self.parse_expression()?;

            if !self.check(TokenType::Semicolon) {
                return Err(format!("[line {}] Expect ';' after expression.", self.current().line));
            }

            self.advance();

            Statement::Print(expression)
        } else if matches!(self, TokenType::Return) {
            let mut expression: Option<Expression> = None;

            if !self.check(TokenType::Semicolon) {
                expression = Some(self.parse_expression()?);
            }

            if !self.check(TokenType::Semicolon) {
                return Err(format!("[line {}] Expect ';' after return value.", self.current().line));
            }
            self.advance();

            Statement::Return(expression)
        } else if matches!(self, TokenType::LeftBrace) {
            let mut statements: Vec<Statement> = Vec::new();

            while !self.check(TokenType::RightBrace) && !self.is_at_end() {
                statements.push(self.parse_declaration()?);
            }

            if !self.check(TokenType::RightBrace) {
                return Err(format!("[line {}] Expect '{}' after block.", self.current().line, '}'));
            }

            self.advance();

            Statement::Block(statements)
        } else if matches!(self, TokenType::If) {
            if !self.check(TokenType::LeftParen) {
                return Err(format!("[line {}] Expect '(' after 'if'.", self.current().line));
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
        } else if matches!(self, TokenType::While) {
            if !self.check(TokenType::LeftParen) {
                return Err(format!("[line {}] Expect '(' after 'while'.", self.current().line));
            }
            self.advance();

            let expression = self.parse_expression()?;

            if !self.check(TokenType::RightParen) {
                return Err(format!("[line {}] Expect ')' after condition.", self.current().line));
            }
            self.advance();

            let body = self.parse_statement()?;

            Statement::While(expression, Box::new(body))
        } else if matches!(self, TokenType::For) {
            if !self.check(TokenType::LeftParen) {
                return Err(format!("[line {}] Expect '(' after 'for'.", self.current().line));
            }
            self.advance();

            let mut initial: Option<Box<Statement>> = None;

            if !self.check(TokenType::Semicolon) {
                initial = Some(Box::new(self.parse_variable_declaration()?));
            } else {
                self.advance();
            }

            let mut condition: Option<Expression> = None;

            if !self.check(TokenType::Semicolon) {
                condition = Some(self.parse_expression()?);
            }

            if !self.check(TokenType::Semicolon) {
                return Err(format!("[line {}] Expect ';' after for condition.", self.current().line));
            }
            self.advance();

            let mut incrementer: Option<Expression> = None;

            if !self.check(TokenType::RightParen) {
                incrementer = Some(self.parse_expression()?);
            }

            if !self.check(TokenType::RightParen) {
                return Err(format!("[line {}] Expect ')' after for clauses.", self.current().line));
            }
            self.advance();

            let body = self.parse_statement()?;

            Statement::For(initial, condition, incrementer, Box::new(body))
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
        let mut expression = self.parse_or()?;

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

    fn parse_or(&mut self) -> Result<Expression, String> {
        let mut expression = self.parse_and()?;

        while matches!(self, TokenType::Or) {
            let right = self.parse_and()?;
            expression = Expression::Or(Box::new(expression), Box::new(right));
        }

        Ok(expression)
    }

    fn parse_and(&mut self) -> Result<Expression, String> {
        let mut expression = self.parse_equality()?;

        while matches!(self, TokenType::And) {
            let right = self.parse_equality()?;
            expression = Expression::And(Box::new(expression), Box::new(right));
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

        self.parse_call()
    }

    fn parse_call(&mut self) -> Result<Expression, String> {
        let mut expression = self.parse_primary()?;

        loop {
            if matches!(self, TokenType::LeftParen) {
                expression = Expression::Call(Box::new(expression), self.finish_call()?);
            } else {
                break;
            }
        }

        Ok(expression)
    }

    fn finish_call(&mut self) -> Result<Vec<Expression>, String> {
        let mut arguments: Vec<Expression> = Vec::new();

        if !self.check(TokenType::RightParen) {
            loop {
                if arguments.len() >= 255 {
                    return Err(format!("[line {}] Can't have more than 255 arguments.", self.current().line));
                }

                arguments.push(self.parse_expression()?);

                if !matches!(self, TokenType::Comma) {
                    break;
                }
            }
        }

        if !matches!(self, TokenType::RightParen) {
            Err(format!("[line {}] Expect ')' after arguments.", self.current().line))
        } else {
            Ok(arguments)
        }
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
    use crate::expression::Expression;
    use crate::parser::Parser;
    use crate::tokenizer::Scanner;

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
    #[case("1 or 1", "(1.0 or 1.0)")]
    #[case("1 and 1", "(1.0 and 1.0)")]
    #[case("(1 and 1) or 1", "((group (1.0 and 1.0)) or 1.0)")]
    fn test_parser_and_or(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(expected, run_expression(input).unwrap().to_string());
    }

    #[rstest]
    #[case("test()", "(call (variable test))")]
    #[case("test(1)", "(call (variable test) 1.0)")]
    #[case(&format!("test(1{})", ", 1".repeat(254)), &format!("(call (variable test) 1.0{})", " 1.0".repeat(254)))]
    #[case("test(\"test\", a, 2)", "(call (variable test) test (variable a) 2.0)")]
    fn test_parser_call(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(expected, run_expression(input).unwrap().to_string());
    }

    #[rstest]
    #[case("test({", "[line 1] Error at '{': Expect expression.")]
    #[case("test(1", "[line 1] Expect ')' after arguments.")]
    #[case(&format!("test(1{})", ", 1".repeat(255)), "[line 1] Can't have more than 255 arguments.")]
    fn test_parser_call_error(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(expected, run_expression(input).err().unwrap().to_string());
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
    #[case("if (1==1) print 1;", "(if (== 1.0 1.0), (print (; 1.0)))")]
    #[case("if (1==1) print 1; else print 2;", "(if (== 1.0 1.0), (print (; 1.0)) (print (; 2.0)))")]
    #[case("while (1==1) print 1;", "(while ((== 1.0 1.0)) (print (; 1.0)))")]
    #[case("for (;;) print 1;", "(for (;;) (print (; 1.0)))")]
    #[case("for (var a = 1;;) print 1;", "(for ((var a = (; 1.0));;) (print (; 1.0)))")]
    #[case("for (var a = 1; a < 10;) print 1;", "(for ((var a = (; 1.0));(< (variable a) 10.0);) (print (; 1.0)))")]
    #[case("for (var a = 1;; a = 1) print 1;", "(for ((var a = (; 1.0));;(assign a 1.0)) (print (; 1.0)))")]
    #[case("for (var a = 1; a < 10; a = 1) print 1;", "(for ((var a = (; 1.0));(< (variable a) 10.0);(assign a 1.0)) (print (; 1.0)))")]
    #[case("for (; a < 10;) print 1;", "(for (;(< (variable a) 10.0);) (print (; 1.0)))")]
    #[case("for (; a < 10; a = 1) print 1;", "(for (;(< (variable a) 10.0);(assign a 1.0)) (print (; 1.0)))")]
    #[case("for (;; a = 1) print 1;", "(for (;;(assign a 1.0)) (print (; 1.0)))")]
    fn test_parser_statement_control_flow(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(expected, run_statement(input).unwrap());
    }

    #[rstest]
    #[case("fun bar() { print 10; }", "(function bar() (block ((print (; 10.0)))))")]
    #[case("fun bar(a, b, c) { print a + b + c; }", "(function bar(a, b, c) (block ((print (; (+ (+ (variable a) (variable b)) (variable c)))))))")]
    fn test_parser_statement_function(#[case] input: &str, #[case] expected: &str) {
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
    #[case("if", "[line 1] Expect '(' after 'if'.")]
    #[case("if(1==1", "[line 1] Expect ')' after if condition.")]
    #[case("while", "[line 1] Expect '(' after 'while'.")]
    #[case("while(1==1", "[line 1] Expect ')' after condition.")]
    #[case("for", "[line 1] Expect '(' after 'for'.")]
    #[case("for(var a = 1;a < 10", "[line 1] Expect ';' after for condition.")]
    #[case("for(var a = 1;a < 10; a = a + 1", "[line 1] Expect ')' after for clauses.")]
    fn test_parser_statement_error(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(expected, run_statement(input).err().unwrap());
    }
}