use crate::syntax::expression::Expression;

pub enum Statement {
    Print(Expression),
    Variable(String, Option<Expression>),
    Expression(Expression),
}