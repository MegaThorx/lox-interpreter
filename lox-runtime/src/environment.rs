use std::collections::hash_map::Entry::Occupied;
use std::collections::HashMap;
use crate::value::{Error, Value};

type Scope = HashMap<String, Value>;

pub struct Environment {
    scopes: Vec<Scope>,
}

impl Default for Environment {
    fn default() -> Self {
        Self { scopes: vec![Default::default()] }
    }
}

impl Environment {
    pub fn push_scope(&mut self) {
        self.scopes.push(Default::default());
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn current_mut(&mut self) -> &mut Scope {
        self.scopes.last_mut().expect("scopes is empty")
    }

    pub fn declare(&mut self, name: String, value: Value) {
        self.current_mut().insert(name, value);
    }

    pub fn assign(&mut self, name: String, value: Value) -> Result<(), Error> {
        for scope in self.scopes.iter_mut().rev() {
            if let Occupied(mut entry) = scope.entry(name.clone()) {
                entry.insert(value);
                return Ok(());
            }
        }
        
        Err(Error::Runtime("Undefined variable".into()))
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        for scope in self.scopes.iter().rev() {
            if scope.contains_key(name) {
                return scope.get(name);
            }
        }
        
        None
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;
    use crate::environment::Environment;
    use crate::value::Value;

    #[rstest]
    #[case(Value::String("string".into()))]
    #[case(Value::Number(432.1))]
    #[case(Value::Bool(true))]
    #[case(Value::None)]
    fn test_environment_declare(#[case] value: Value) {
        let mut env = Environment::default();
        env.declare("name".to_string(), value.clone());
        assert_eq!(&value, env.get("name").unwrap());
    }

    #[rstest]
    #[case(Value::String("hello".into()), Value::String("world".into()))]
    #[case(Value::Number(432.1), Value::Number(42.0))]
    #[case(Value::Bool(true), Value::Bool(false))]
    #[case(Value::None, Value::Number(432.1))]
    fn test_environment_declare_shadow(#[case] value1: Value, #[case] value2: Value) {
        let mut env = Environment::default();
        env.declare("name".to_string(), value1.clone());
        env.push_scope();
        env.declare("name".to_string(), value2.clone());
        assert_eq!(&value2, env.get("name").unwrap());
        env.pop_scope();
        assert_eq!(&value1, env.get("name").unwrap());
    }

    #[rstest]
    #[case(Value::String("hello".into()), Value::String("world".into()))]
    #[case(Value::Number(432.1), Value::Number(42.0))]
    #[case(Value::Bool(true), Value::Bool(false))]
    #[case(Value::None, Value::Number(432.1))]
    fn test_environment_declare_and_assign(#[case] value1: Value, #[case] value2: Value) {
        let mut env = Environment::default();
        env.declare("name".to_string(), value1.clone());
        assert_eq!(&value1, env.get("name").unwrap());
        assert!(env.assign("name".to_string(), value2.clone()).is_ok());
        assert_eq!(&value2, env.get("name").unwrap());
    }

    #[rstest]
    #[case(Value::String("string".into()))]
    #[case(Value::Number(432.1))]
    #[case(Value::Bool(true))]
    #[case(Value::None)]
    fn test_environment_assign_without_declare(#[case] value: Value) {
        let mut env = Environment::default();
        assert!(env.assign("name".to_string(), value.clone()).is_err());
    }

    #[test]
    fn test_environment_get_without_declare() {
        let env = Environment::default();
        assert!(env.get("name").is_none());
    }
}