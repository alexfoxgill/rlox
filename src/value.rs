use core::fmt;
use std::rc::Rc;

use crate::string_intern::StrId;

#[derive(PartialEq, Clone, Debug)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
    Object(Rc<Object>),
}

impl Value {
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Value::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&'static str> {
        if let Value::Object(o) = self {
            if let Object::String(s) = o.as_ref() {
                Some(s)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn as_string_id(&self) -> Option<StrId> {
        if let Value::Object(o) = self {
            if let Object::StringId(id) = o.as_ref() {
                Some(*id)
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Number(n) => write!(f, "{n}"),
            Value::Object(obj) => match obj.as_ref() {
                Object::String(s) => write!(f, "{s}"),
                Object::StringId(id) => write!(f, "{id:?}"),
            },
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum Object {
    String(&'static str),
    StringId(StrId),
}

pub struct ValueArray {
    pub values: Vec<Value>,
}

impl ValueArray {
    pub fn new() -> ValueArray {
        ValueArray {
            values: Vec::with_capacity(8),
        }
    }

    pub fn write(&mut self, value: Value) {
        self.values.push(value);
    }

    pub fn free(&mut self) {
        self.values.clear();
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }
}
