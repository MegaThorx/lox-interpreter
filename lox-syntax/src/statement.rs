use std::fmt::{Display, Formatter};
use crate::expression::Expression;

pub enum Statement {
    Print(Expression),
    Variable(String, Option<Expression>),
    Expression(Expression),
    Block(Vec<Statement>),
    If(Expression, Box<Statement>, Option<Box<Statement>>),
    While(Expression, Box<Statement>),
    For(Option<Box<Statement>>, Option<Expression>, Option<Expression>, Box<Statement>),
}

impl Display for Statement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::Print(expression) => write!(f, "(print (; {}))", expression),
            Statement::Variable(name, expression) => match expression {
                Some(expression) => write!(f, "(var {} = (; {}))", name, expression),
                None => write!(f, "(var {})", name),
            },
            Statement::Expression(expression) => write!(f, "(; {})", expression),
            Statement::Block(statements) => write!(f, "(block ({}))", statements.iter().map(|statement| statement.to_string()).collect::<Vec<String>>().join(" ")),
            Statement::If(expression, if_body, else_body) => match else_body {
                Some(else_body) => write!(f, "(if {}, {} {})", expression, if_body, else_body),
                None => write!(f, "(if {}, {})", expression, if_body),  
            },
            Statement::While(expression, body) => write!(f, "(while ({}) {})", expression, body),
            Statement::For(initial, condition, incrementer, body) => match initial {
                Some(initial) => match condition {
                    Some(condition) => match incrementer {
                        Some(incrementer) => write!(f, "(for ({};{};{}) {})", initial, condition, incrementer, body),
                        None => write!(f, "(for ({};{};) {})", initial, condition, body),
                    }
                    None => match incrementer {
                        Some(incrementer) => write!(f, "(for ({};;{}) {})", initial, incrementer, body),
                        None => write!(f, "(for ({};;) {})", initial, body),
                    }
                },
                None => match condition {
                    Some(condition) => match incrementer {
                        Some(incrementer) => write!(f, "(for (;{};{}) {})", condition, incrementer, body),
                        None => write!(f, "(for (;{};) {})", condition, body),
                    }
                    None => match incrementer {
                        Some(incrementer) => write!(f, "(for (;;{}) {})", incrementer, body),
                        None => write!(f, "(for (;;) {})", body),
                    }
                }
            },
        }
    }
}