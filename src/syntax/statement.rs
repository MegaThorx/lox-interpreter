use crate::syntax::expression::Expression;

pub enum Statement {
    Print(Expression),
    Expression(Expression),
}