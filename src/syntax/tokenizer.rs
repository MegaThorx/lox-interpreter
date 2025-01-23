use crate::syntax::token::{Token, TokenType};

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

            let token_type = match token {
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

            errors.push(format!("[line {}] Error: Unexpected character: {}", self.line, token));
        }

        tokens.push(Token::new(TokenType::Eof, "", self.line));

        (tokens, errors)
    }
}


#[cfg(test)]
mod tests {
    use crate::syntax::token::{Token, TokenType};
    use crate::syntax::tokenizer::Scanner;

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
        let source = "({=}){==}(!){!=}<(>(>=(<=";
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
}