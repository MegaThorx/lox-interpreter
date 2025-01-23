use std::fmt::Display;

#[derive(Debug, PartialEq)]
pub enum TokenType<'a> {
    // Single character tokens
    LeftParen, RightParen, LeftBrace, RightBrace,
    Comma, Dot, Semicolon, Minus, Plus, Star,

    // One or two character tokens
    Slash,
    Equal, EqualEqual,
    Bang, BangEqual,
    Less, LessEqual,
    Greater, GreaterEqual,

    // Literals
    String(&'a str),

    Eof,
}

impl Display for TokenType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let token_name = match self {
            &TokenType::String(_) => "STRING".to_string(),
            _ => {
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
                
                token_name
            }
        };

        write!(f, "{}", token_name)
    }
}

#[derive(Debug, PartialEq)]
pub struct Token<'a> {
    pub token: TokenType<'a>,
    pub lexeme: &'a str,
    pub line: usize,
}

impl Token<'_> {
    pub fn new<'a>(token: TokenType<'a>, lexeme: &'a str, line: usize) -> Token<'a> {
        Token {
            token,
            lexeme,
            line,
        }
    } 
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self.token {
            TokenType::String(value) => value.to_string(),
            _ => "null".to_string(),
        };

        write!(f, "{} {} {}", self.token, self.lexeme, value)
    }
}