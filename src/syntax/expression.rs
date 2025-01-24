use std::fmt::Display;

pub enum Literal<'a> {
    Bool(bool),
    Number(f64),
    String(&'a str),
    None,
}

impl Display for Literal<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Bool(bool) => write!(f, "{}", bool),
            Literal::Number(number) => {
                match number.fract() == 0.0 {
                    true => write!(f, "{:.1}", number),
                    _ => write!(f, "{}", number),
                }
            },
            Literal::String(string) => write!(f, "{}", string),
            Literal::None => write!(f, "nil"),
        }
    }
}

pub enum Expression<'a> {
    Literal(Literal<'a>),
}

impl Display for Expression<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Literal(literal) => write!(f, "{}", literal),
        }
    }
}