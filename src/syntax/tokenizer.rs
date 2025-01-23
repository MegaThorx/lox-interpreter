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

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        let mut tokens: Vec<Token> = Vec::new();
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
                _ => None,
            };

            if let Some(token_type) = token_type {
                tokens.push(Token::new(token_type, &self.source[self.start..self.current], self.line));
                continue;
            }
        }

        tokens.push(Token::new(TokenType::Eof, "", self.line));

        tokens
    }
}


#[cfg(test)]
mod tests {
    use crate::syntax::token::{Token, TokenType};
    use crate::syntax::tokenizer::Scanner;

    #[test]
    fn test_lexer_single_character_tokens() {
        let source = "(()";
        let mut scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens();

        assert_eq!(tokens, vec![
            Token { token: TokenType::LeftParen, lexeme: "(", line: 1 },
            Token { token: TokenType::LeftParen, lexeme: "(", line: 1 },
            Token { token: TokenType::RightParen, lexeme: ")", line: 1 },
            Token { token: TokenType::Eof, lexeme: "", line: 1 }
        ]);
    }
}