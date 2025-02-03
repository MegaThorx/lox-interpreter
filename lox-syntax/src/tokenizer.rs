use phf::{phf_map, Map};
use crate::token::{Token, TokenType};

static KEYWORDS: Map<&'static str, TokenType> = phf_map! {
    "and" => TokenType::And,
    "class" => TokenType::Class,
    "else" => TokenType::Else,
    "false" => TokenType::False,
    "for" => TokenType::For,
    "fun" => TokenType::Fun,
    "if" => TokenType::If,
    "nil" => TokenType::Nil,
    "or" => TokenType::Or,
    "print" => TokenType::Print,
    "return" => TokenType::Return,
    "super" => TokenType::Super,
    "this" => TokenType::This,
    "true" => TokenType::True,
    "var" => TokenType::Var,
    "while" => TokenType::While,
};

pub struct Scanner<'a> {
    source: &'a str,
    line: usize,
    current: usize,
    start: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Scanner {
            source,
            line: 0,
            current: 0,
            start: 0,
        }
    }

    pub fn scan_tokens(&mut self) -> (Vec<Token>, Vec<String>) {
        let mut tokens: Vec<Token> = Vec::new();
        let mut errors: Vec<String> = Vec::new();
        self.current = 0;
        self.line = 1;

        let mut peekable = self.source.chars().peekable();

        loop {
            if self.source.len() <= self.current {
                break;
            }

            let token = peekable.next().unwrap();

            self.start = self.current;
            self.current += token.len_utf8();

            let should_ignore = match token {
                ' ' => true,
                '\r' => true,
                '\t' => true,
                '\n' => {
                    self.line += 1;
                    true
                },
                _ => false,
            };

            if should_ignore {
                continue;
            }

            let token_type = match token {
                '(' => Some(TokenType::LeftParen),
                ')' => Some(TokenType::RightParen),
                '{' => Some(TokenType::LeftBrace),
                '}' => Some(TokenType::RightBrace),
                ',' => Some(TokenType::Comma),
                '.' => Some(TokenType::Dot),
                ';' => Some(TokenType::Semicolon),
                '-' => Some(TokenType::Minus),
                '+' => Some(TokenType::Plus),
                '*' => Some(TokenType::Star),
                _ => None,
            };

            if let Some(token_type) = token_type {
                tokens.push(Token::new(token_type, &self.source[self.start..self.current], self.line));
                continue;
            }

            let token_type = match (token, peekable.peek()) {
                ('=', Some('=')) => Some(TokenType::EqualEqual),
                ('!', Some('=')) => Some(TokenType::BangEqual),
                ('<', Some('=')) => Some(TokenType::LessEqual),
                ('>', Some('=')) => Some(TokenType::GreaterEqual),
                (_, _) => None,
            };

            if let Some(token_type) = token_type {
                self.current += 1;
                peekable.next();
                tokens.push(Token::new(token_type, &self.source[self.start..self.current], self.line));
                continue;
            }

            if token == '/' && peekable.peek() == Some(&'/') {
                peekable.next(); // Consume second slash
                self.current += token.len_utf8();

                for token in peekable.by_ref() {
                    self.current += token.len_utf8();
                    if token == '\n' {
                        self.line += 1;
                        break;
                    }
                }

                continue;
            }

            let token_type = match token {
                '/' => Some(TokenType::Slash),
                '=' => Some(TokenType::Equal),
                '!' => Some(TokenType::Bang),
                '<' => Some(TokenType::Less),
                '>' => Some(TokenType::Greater),
                _ => None,
            };

            if let Some(token_type) = token_type {
                tokens.push(Token::new(token_type, &self.source[self.start..self.current], self.line));
                continue;
            }

            if token == '"' {
                let line_start = self.line;
                loop {
                    if let Some(token) = peekable.next() {
                        self.current += token.len_utf8();
                        if token == '"' {
                            tokens.push(Token::new(TokenType::String(&self.source[self.start + 1..self.current - 1]), &self.source[self.start..self.current], line_start));
                            break;
                        } else if token == '\n' {
                            self.line += 1;
                        }
                    } else {
                        errors.push(format!("[line {}] Error: Unterminated string.", self.line));
                        break;
                    }
                }
                continue;
            }

            if token.is_ascii_digit() {
                let mut found_dot = false;
                while let Some(token) = peekable.peek() {
                    if token.is_ascii_digit() {
                        self.current += 1;
                        peekable.next();
                    } else if *token == '.' && !found_dot {
                        found_dot = true;
                        peekable.next();
                        self.current += 1;
                    } else {
                        break;
                    }
                }

                let value: f64 = self.source[self.start..self.current]
                    .parse()
                    .unwrap();

                tokens.push(Token::new(TokenType::Number(value), &self.source[self.start..self.current], self.line));

                continue;
            }

            if token.is_ascii_alphabetic() || token == '_' {
                while let Some(token) = peekable.peek() {
                    if token.is_ascii_alphanumeric() || *token == '_' {
                        peekable.next();
                        self.current += 1;
                    } else {
                        break;
                    }
                }

                if let Some(token_type) = KEYWORDS.get(&self.source[self.start..self.current]) {
                    tokens.push(Token::new(token_type.clone(), &self.source[self.start..self.current], self.line));
                } else {
                    tokens.push(Token::new(TokenType::Identifier(&self.source[self.start..self.current]), &self.source[self.start..self.current], self.line));
                }

                continue;
            }

            errors.push(format!("[line {}] Error: Unexpected character: {}", self.line, token));
        }

        tokens.push(Token::new(TokenType::Eof, "", self.line));

        (tokens, errors)
    }
}


#[cfg(test)]
mod tests {
    use crate::token::{Token, TokenType};
    use crate::tokenizer::Scanner;

    #[test]
    fn test_lexer_single_character_tokens() {
        let source = "{(,.;-+*)}";
        let mut scanner = Scanner::new(source);
        let (tokens, errors) = scanner.scan_tokens();

        assert!(errors.is_empty());
        assert_eq!(tokens, vec![
            Token { token: TokenType::LeftBrace, lexeme: "{", line: 1 },
            Token { token: TokenType::LeftParen, lexeme: "(", line: 1 },
            Token { token: TokenType::Comma, lexeme: ",", line: 1 },
            Token { token: TokenType::Dot, lexeme: ".", line: 1 },
            Token { token: TokenType::Semicolon, lexeme: ";", line: 1 },
            Token { token: TokenType::Minus, lexeme: "-", line: 1 },
            Token { token: TokenType::Plus, lexeme: "+", line: 1 },
            Token { token: TokenType::Star, lexeme: "*", line: 1 },
            Token { token: TokenType::RightParen, lexeme: ")", line: 1 },
            Token { token: TokenType::RightBrace, lexeme: "}", line: 1 },
            Token { token: TokenType::Eof, lexeme: "", line: 1 }
        ]);
    }

    #[test]
    fn test_lexer_one_or_two_character_tokens() {
        let source = "({=}){==}(!){!=}<(>(>=(<=/(//()";
        let mut scanner = Scanner::new(source);
        let (tokens, errors) = scanner.scan_tokens();

        assert!(errors.is_empty());
        assert_eq!(tokens, vec![
            Token { token: TokenType::LeftParen, lexeme: "(", line: 1 },
            Token { token: TokenType::LeftBrace, lexeme: "{", line: 1 },
            Token { token: TokenType::Equal, lexeme: "=", line: 1 },
            Token { token: TokenType::RightBrace, lexeme: "}", line: 1 },
            Token { token: TokenType::RightParen, lexeme: ")", line: 1 },
            Token { token: TokenType::LeftBrace, lexeme: "{", line: 1 },
            Token { token: TokenType::EqualEqual, lexeme: "==", line: 1 },
            Token { token: TokenType::RightBrace, lexeme: "}", line: 1 },
            Token { token: TokenType::LeftParen, lexeme: "(", line: 1 },
            Token { token: TokenType::Bang, lexeme: "!", line: 1 },
            Token { token: TokenType::RightParen, lexeme: ")", line: 1 },
            Token { token: TokenType::LeftBrace, lexeme: "{", line: 1 },
            Token { token: TokenType::BangEqual, lexeme: "!=", line: 1 },
            Token { token: TokenType::RightBrace, lexeme: "}", line: 1 },
            Token { token: TokenType::Less, lexeme: "<", line: 1 },
            Token { token: TokenType::LeftParen, lexeme: "(", line: 1 },
            Token { token: TokenType::Greater, lexeme: ">", line: 1 },
            Token { token: TokenType::LeftParen, lexeme: "(", line: 1 },
            Token { token: TokenType::GreaterEqual, lexeme: ">=", line: 1 },
            Token { token: TokenType::LeftParen, lexeme: "(", line: 1 },
            Token { token: TokenType::LessEqual, lexeme: "<=", line: 1 },
            Token { token: TokenType::Slash, lexeme: "/", line: 1 },
            Token { token: TokenType::LeftParen, lexeme: "(", line: 1 },
            Token { token: TokenType::Eof, lexeme: "", line: 1 }
        ]);
    }

    #[test]
    fn test_lexer_lexical_errors() {
        let source = ",.$(#";
        let mut scanner = Scanner::new(source);
        let (tokens, errors) = scanner.scan_tokens();

        assert_eq!(errors, vec![
            "[line 1] Error: Unexpected character: $",
            "[line 1] Error: Unexpected character: #",
        ]);
        assert_eq!(tokens, vec![
            Token { token: TokenType::Comma, lexeme: ",", line: 1 },
            Token { token: TokenType::Dot, lexeme: ".", line: 1 },
            Token { token: TokenType::LeftParen, lexeme: "(", line: 1 },
            Token { token: TokenType::Eof, lexeme: "", line: 1 }
        ]);
    }

    #[test]
    fn test_lexer_whitespaces() {
        let source = " \t\r\n";
        let mut scanner = Scanner::new(source);
        let (tokens, errors) = scanner.scan_tokens();

        assert!(errors.is_empty());
        assert_eq!(tokens, vec![
            Token { token: TokenType::Eof, lexeme: "", line: 2 }
        ]);
    }

    #[test]
    fn test_lexer_literal_string() {
        let source = " \"Hello World\"\n\"\"";
        let mut scanner = Scanner::new(source);
        let (tokens, errors) = scanner.scan_tokens();

        assert!(errors.is_empty());
        assert_eq!(tokens, vec![
            Token { token: TokenType::String("Hello World"), lexeme: "\"Hello World\"", line: 1 },
            Token { token: TokenType::String(""), lexeme: "\"\"", line: 2 },
            Token { token: TokenType::Eof, lexeme: "", line: 2 }
        ]);
    }


    #[test]
    fn test_lexer_literal_string_with_newline() {
        let source = " \"Hello\nWorld\"";
        let mut scanner = Scanner::new(source);
        let (tokens, errors) = scanner.scan_tokens();

        assert!(errors.is_empty());
        assert_eq!(tokens, vec![
            Token { token: TokenType::String("Hello\nWorld"), lexeme: "\"Hello\nWorld\"", line: 1 },
            Token { token: TokenType::Eof, lexeme: "", line: 2 }
        ]);
    }

    #[test]
    fn test_lexer_literal_string_unterminated() {
        let source = " \"Hello World";
        let mut scanner = Scanner::new(source);
        let (tokens, errors) = scanner.scan_tokens();

        assert_eq!(errors, vec![
            "[line 1] Error: Unterminated string.".to_string(),
        ]);
        assert_eq!(tokens, vec![
            Token { token: TokenType::Eof, lexeme: "", line: 1 }
        ]);
    }

    #[test]
    fn test_lexer_literal_number() {
        let source = "123 123.123 .1 1";
        let mut scanner = Scanner::new(source);
        let (tokens, errors) = scanner.scan_tokens();

        assert!(errors.is_empty());
        assert_eq!(tokens, vec![
            Token { token: TokenType::Number(123.0), lexeme: "123", line: 1 },
            Token { token: TokenType::Number(123.123), lexeme: "123.123", line: 1 },
            Token { token: TokenType::Dot, lexeme: ".", line: 1 },
            Token { token: TokenType::Number(1.0), lexeme: "1", line: 1 },
            Token { token: TokenType::Number(1.0), lexeme: "1", line: 1 },
            Token { token: TokenType::Eof, lexeme: "", line: 1 }
        ]);
    }

    #[test]
    fn test_lexer_literal_identifier() {
        let source = "tomato apple nuts1 deez_nuts _test";
        let mut scanner = Scanner::new(source);
        let (tokens, errors) = scanner.scan_tokens();

        assert!(errors.is_empty());
        assert_eq!(tokens, vec![
            Token { token: TokenType::Identifier("tomato"), lexeme: "tomato", line: 1 },
            Token { token: TokenType::Identifier("apple"), lexeme: "apple", line: 1 },
            Token { token: TokenType::Identifier("nuts1"), lexeme: "nuts1", line: 1 },
            Token { token: TokenType::Identifier("deez_nuts"), lexeme: "deez_nuts", line: 1 },
            Token { token: TokenType::Identifier("_test"), lexeme: "_test", line: 1 },
            Token { token: TokenType::Eof, lexeme: "", line: 1 }
        ]);
    }

    #[test]
    fn test_lexer_literal_keywords() {
        let source = "and class else false for fun if nil or print return super this true var while";
        let mut scanner = Scanner::new(source);
        let (tokens, errors) = scanner.scan_tokens();

        assert!(errors.is_empty());
        assert_eq!(tokens, vec![
            Token { token: TokenType::And, lexeme: "and", line: 1 },
            Token { token: TokenType::Class, lexeme: "class", line: 1 },
            Token { token: TokenType::Else, lexeme: "else", line: 1 },
            Token { token: TokenType::False, lexeme: "false", line: 1 },
            Token { token: TokenType::For, lexeme: "for", line: 1 },
            Token { token: TokenType::Fun, lexeme: "fun", line: 1 },
            Token { token: TokenType::If, lexeme: "if", line: 1 },
            Token { token: TokenType::Nil, lexeme: "nil", line: 1 },
            Token { token: TokenType::Or, lexeme: "or", line: 1 },
            Token { token: TokenType::Print, lexeme: "print", line: 1 },
            Token { token: TokenType::Return, lexeme: "return", line: 1 },
            Token { token: TokenType::Super, lexeme: "super", line: 1 },
            Token { token: TokenType::This, lexeme: "this", line: 1 },
            Token { token: TokenType::True, lexeme: "true", line: 1 },
            Token { token: TokenType::Var, lexeme: "var", line: 1 },
            Token { token: TokenType::While, lexeme: "while", line: 1 },
            Token { token: TokenType::Eof, lexeme: "", line: 1 }
        ]);
    }

    #[test]
    fn test_lexer_comment() {
        let source = "123// Hello World\n123.123";
        let mut scanner = Scanner::new(source);
        let (tokens, errors) = scanner.scan_tokens();

        assert!(errors.is_empty());
        assert_eq!(tokens, vec![
            Token { token: TokenType::Number(123.0), lexeme: "123", line: 1 },
            Token { token: TokenType::Number(123.123), lexeme: "123.123", line: 2 },
            Token { token: TokenType::Eof, lexeme: "", line: 2 }
        ]);
    }

    #[test]
    fn test_lexer_and_token_to_string() {
        let source = "\"test\" 123 123.123 asdf ==";
        let mut scanner = Scanner::new(source);
        let (tokens, errors) = scanner.scan_tokens();

        assert!(errors.is_empty());
        assert_eq!(tokens, vec![
            Token { token: TokenType::String("test"), lexeme: "\"test\"", line: 1 },
            Token { token: TokenType::Number(123.0), lexeme: "123", line: 1 },
            Token { token: TokenType::Number(123.123), lexeme: "123.123", line: 1 },
            Token { token: TokenType::Identifier("asdf"), lexeme: "asdf", line: 1 },
            Token { token: TokenType::EqualEqual, lexeme: "==", line: 1 },
            Token { token: TokenType::Eof, lexeme: "", line: 1 }
        ]);  
        assert_eq!(tokens.iter().map(|token| format!("{}", token)).collect::<Vec<String>>(), vec![
            "STRING \"test\" test",
            "NUMBER 123 123.0",
            "NUMBER 123.123 123.123",
            "IDENTIFIER asdf null",
            "EQUAL_EQUAL == null",
            "EOF  null"
        ]);
    }
}