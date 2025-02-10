use std::fmt::Display;
use lox_syntax::expression::Literal;
use lox_syntax::statement::Statement;

#[derive(PartialEq, Debug, Clone)]
pub enum Value {
    Bool(bool),
    Number(f64),
    String(String),
    Callable(Callable),
    None,
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(bool) => *bool,
            Value::None => false,
            _ => true,
        }
    }

    pub fn is_equal(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Bool(left), Value::Bool(right)) => left == right,
            (Value::Number(left), Value::Number(right)) => left == right,
            (Value::String(left), Value::String(right)) => left == right,
            (Value::None, Value::None) => true,
            _ => false,
        }
    }

    pub fn from_literal(literal: Literal) -> Value {
        match literal {
            Literal::Bool(value) => Value::Bool(value),
            Literal::Number(value) => Value::Number(value),
            Literal::String(value) => Value::String(value),
            Literal::None => Value::None,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Bool(bool) => write!(f, "{}", bool),
            Value::Number(number) => {
                match number.fract() == 0.0 {
                    true => write!(f, "{:.0}", number),
                    _ => write!(f, "{}", number),
                }
            },
            Value::String(string) => write!(f, "{}", string),
            Value::Callable(callable) => write!(f, "{}", callable),
            Value::None => write!(f, "nil"),
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum Callable {
    Native(usize, Box<fn(&Vec<Value>) -> Value>),
    Function(String, Vec<String>, Box<Statement>),
}


impl Display for Callable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Callable::Native(_, _) => write!(f, "<native fn>"),
            Callable::Function(name, _, _) => write!(f, "<fn {}>", name),
        }
    }
}

pub enum Error {
    Runtime(String),
    Return(Value),
}
