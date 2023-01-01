use crate::{
    memory::{ClosureId, FunctionId, NativeFunctionId},
    string_intern::StrId,
};

#[derive(PartialEq, Clone, Copy)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
    String(&'static str),
    StringId(StrId),
    Function(FunctionId),
    Closure(ClosureId),
    NativeFunction(NativeFunctionId),
}

impl Value {
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Value::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&'static str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_string_id(&self) -> Option<StrId> {
        match self {
            Value::StringId(id) => Some(*id),
            _ => None,
        }
    }

    pub fn as_function(&self) -> Option<FunctionId> {
        match self {
            Value::Function(id) => Some(*id),
            _ => None,
        }
    }

    pub fn as_native_function(&self) -> Option<NativeFunctionId> {
        match self {
            Value::NativeFunction(id) => Some(*id),
            _ => None,
        }
    }

    pub fn as_closure(&self) -> Option<ClosureId> {
        match self {
            Value::Closure(id) => Some(*id),
            _ => None,
        }
    }
}
