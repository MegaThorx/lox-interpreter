use std::fmt::{Display, Formatter};
use crate::syntax::expression::Expression;

pub enum Statement {
    Print(Expression),
    Variable(String, Option<Expression>),
    Expression(Expression),
    Block(Vec<Statement>),
}

impl Display for Statement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::Print(expression) => write!(f, "(print (; {} ))", expression),
            Statement::Variable(name, expression) => match expression {
                Some(expression) => write!(f, "(var {} = (; {} ))", name, expression),
                None => write!(f, "(var {})", name),
            },
            Statement::Expression(expression) => write!(f, "(; {})", expression),
            Statement::Block(statements) => write!(f, "(block (; {} ))", statements.iter().map(|statement| statement.to_string()).collect::<Vec<String>>().join(" ")),
        }
    }
}