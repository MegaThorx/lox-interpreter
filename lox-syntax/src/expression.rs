use std::fmt::Display;

#[derive(PartialEq, Debug, Clone)]
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

#[derive(PartialEq, Debug, Clone)]
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

#[derive(PartialEq, Debug, Clone)]
pub enum BinaryOperation {
    Multiply,
    Divide,
    Plus,
    Minus,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Equal,
    NotEqual,
}

impl Display for BinaryOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryOperation::Multiply => write!(f, "*"),
            BinaryOperation::Divide => write!(f, "/"),
            BinaryOperation::Plus => write!(f, "+"),
            BinaryOperation::Minus => write!(f, "-"),
            BinaryOperation::Greater => write!(f, ">"),
            BinaryOperation::GreaterEqual => write!(f, ">="),
            BinaryOperation::Less => write!(f, "<"),
            BinaryOperation::LessEqual => write!(f, "<="),
            BinaryOperation::Equal => write!(f, "=="),
            BinaryOperation::NotEqual => write!(f, "!="),
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum Expression {
    Literal(Literal),
    Grouping(Box<Expression>),
    Unary(UnaryOperation, Box<Expression>),
    Binary(BinaryOperation, Box<Expression>, Box<Expression>),
    Variable(String),
    Assign(String, Box<Expression>),
    And(Box<Expression>, Box<Expression>),
    Or(Box<Expression>, Box<Expression>),
    Call(Box<Expression>, Vec<Expression>),
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Literal(literal) => write!(f, "{}", literal),
            Expression::Grouping(expression) => write!(f, "(group {})", expression),
            Expression::Unary(operator, expression) => write!(f, "({} {})", operator, expression),
            Expression::Binary(operator, left, right) => write!(f, "({} {} {})", operator, left, right),
            Expression::Variable(name) => write!(f, "(variable {})", name),
            Expression::Assign(name, expression) => write!(f, "(assign {} {})", name, expression),
            Expression::And(left, right) => write!(f, "({} and {})", left, right),
            Expression::Or(left, right) => write!(f, "({} or {})", left, right),
            Expression::Call(callee, arguments) => match arguments.is_empty() {
                true => write!(f, "(call {})", callee),
                false => write!(f, "(call {} {})", callee, arguments.iter().map(|statement| statement.to_string()).collect::<Vec<String>>().join(" ")),
            },
        }
    }
}