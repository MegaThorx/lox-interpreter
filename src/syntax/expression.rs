use std::fmt::Display;

pub enum Literal {
    Bool(bool),
    Number(f64),
    String(String),
    None,
}

impl Display for Literal {
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

pub enum UnaryOperation {
    Minus,
    Not,
}

impl Display for UnaryOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnaryOperation::Minus => write!(f, "-"),
            UnaryOperation::Not => write!(f, "!"),
        }
    }
}

pub enum BinaryOperation {
    Multiply,
    Divide,
    Plus,
    Minus,
}

impl Display for BinaryOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryOperation::Multiply => write!(f, "*"),
            BinaryOperation::Divide => write!(f, "/"),
            BinaryOperation::Plus => write!(f, "+"),
            BinaryOperation::Minus => write!(f, "-"),
        }
    }
}

pub enum Expression {
    Literal(Literal),
    Grouping(Box<Expression>),
    Unary(UnaryOperation, Box<Expression>),
    Binary(BinaryOperation, Box<Expression>, Box<Expression>),
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Literal(literal) => write!(f, "{}", literal),
            Expression::Grouping(expression) => write!(f, "(group {})", expression),
            Expression::Unary(operator, expression) => write!(f, "({} {})", operator, expression),
            Expression::Binary(operator, left, right) => write!(f, "({} {} {})", operator, left, right),
        }
    }
}