use std::cell::RefCell;
use std::collections::hash_map::Entry::Occupied;
use std::collections::HashMap;
use std::rc::Rc;
use crate::value::{Error, Value};

#[derive(Default, PartialEq, Debug, Clone)]
pub struct Environment {
    values: HashMap<String, Value>,
    enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new_with_enclosing(enclosing: Rc<RefCell<Environment>>) -> Self {
        Self {
            values: HashMap::new(),
            enclosing: Some(enclosing),
        }
    }

    pub fn declare(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    pub fn assign(&mut self, name: String, value: Value) -> Result<(), Error> {
        if let Occupied(mut entry) = self.values.entry(name.clone()) {
            entry.insert(value);
            Ok(())
        } else if self.enclosing.is_some() {
            self.enclosing.as_ref().unwrap().borrow_mut().assign(name, value)?;
            Ok(())
        } else {
            Err(Error::Runtime(format!("Undefined variable '{}'.", name)))
        }
    }

    pub fn get(&self, name: &str) -> Result<Value, Error> {
        if self.values.contains_key(name) {
            Ok(self.values.get(name).unwrap().clone())
        } else if self.enclosing.is_some() {
            Ok(self.enclosing.as_ref().unwrap().borrow().get(name)?)
        } else {
            Err(Error::Runtime(format!("Undefined variable '{}'.", name)))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;
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
        assert_eq!(value, env.get("name").unwrap());
    }

    #[rstest]
    #[case(Value::String("hello".into()), Value::String("world".into()))]
    #[case(Value::Number(432.1), Value::Number(42.0))]
    #[case(Value::Bool(true), Value::Bool(false))]
    #[case(Value::None, Value::Number(432.1))]
    fn test_environment_declare_shadow(#[case] value1: Value, #[case] value2: Value) {
        let mut env = Environment::default();
        env.declare("name".to_string(), value1.clone());
        let env = Rc::new(RefCell::new(env));
        let mut env2 = Environment::new_with_enclosing(Rc::clone(&env));
        env2.declare("name".to_string(), value2.clone());
        assert_eq!(value2, env2.get("name").unwrap());
        assert_eq!(value1, env.borrow().get("name").unwrap());
    }

    #[rstest]
    #[case(Value::String("hello".into()), Value::String("world".into()))]
    #[case(Value::Number(432.1), Value::Number(42.0))]
    #[case(Value::Bool(true), Value::Bool(false))]
    #[case(Value::None, Value::Number(432.1))]
    fn test_environment_declare_and_assign(#[case] value1: Value, #[case] value2: Value) {
        let mut env = Environment::default();
        env.declare("name".to_string(), value1.clone());
        assert_eq!(value1, env.get("name").unwrap());
        assert!(env.assign("name".to_string(), value2.clone()).is_ok());
        assert_eq!(value2, env.get("name").unwrap());
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
        assert!(env.get("name").is_err());
    }
}