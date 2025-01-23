use std::fmt::Display;

#[derive(Debug, PartialEq)]
pub enum TokenType {
    // Single character tokens
    LeftParen, RightParen,

    Eof,
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = format!("{:?}", self);
        let mut chars = name.chars();
        let mut token_name = String::new();

        if let Some(first_char) = chars.next() {
            token_name.push(first_char.to_ascii_uppercase());
            for char in chars {
                if char == char.to_ascii_uppercase() {
                    token_name.push('_');
                    token_name.push(char);
                } else {
                    token_name.push(char.to_ascii_uppercase());
                }
            }
        }

        write!(f, "{}", token_name)
    }
}

#[derive(Debug, PartialEq)]
pub struct Token<'a> {
    pub token: TokenType,
    pub lexeme: &'a str,
    pub line: usize,
}

impl Token<'_> {
    pub fn new(token: TokenType, lexeme: &str, line: usize) -> Token {
        Token {
            token,
            lexeme,
            line,
        }
    }
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.token, self.lexeme, "null")
    }
}