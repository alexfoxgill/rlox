use std::rc::Rc;

use crate::{string_intern::StrId, chunk::Chunk};

#[derive(PartialEq, Clone)]
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
                return Some(s)
            }
        }
        None
    }

    pub fn as_string_id(&self) -> Option<StrId> {
        if let Value::Object(o) = self {
            if let Object::StringId(id) = o.as_ref() {
                return Some(*id)
            }
        }
        None
    }
}

#[derive(PartialEq, Clone)]
pub enum Object {
    String(&'static str),
    StringId(StrId),
    Function(usize)
}

pub struct Function {
    pub arity: usize,
    pub chunk: Chunk,
    pub name: StrId
}

#[derive(Eq, PartialEq, Clone, Copy, Debug, Hash)]
pub enum FunctionType {
    Script,
    Function
}