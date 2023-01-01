use crate::{chunk::Chunk, string_intern::StrId};

#[derive(PartialEq, Clone)]
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
            _ => None
        }
    }

    pub fn as_string_id(&self) -> Option<StrId> {
        match self {
            Value::StringId(id) => Some(*id),
            _ => None
        }
    }

    pub fn as_function(&self) -> Option<FunctionId> {
        match self {
            Value::Function(id) => Some(*id),
            _ => None
        }
    }

    pub fn as_native_function(&self) -> Option<NativeFunctionId> {
        match self {
            Value::NativeFunction(id) => Some(*id),
            _ => None
        }
    }

    pub fn as_closure(&self) -> Option<ClosureId> {
        match self {
            Value::Closure(id) => Some(*id),
            _ => None
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct FunctionId(pub usize);

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct ClosureId(pub usize);

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct NativeFunctionId(pub usize);

pub struct Function {
    pub arity: usize,
    pub chunk: Chunk,
    pub name: StrId,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug, Hash)]
pub enum FunctionType {
    Script,
    Function,
}

pub struct Closure {
    pub function: FunctionId
}

pub struct NativeFunction {
    pub name: StrId,
    pub callable: Box<dyn Fn(&[Value]) -> Value>,
}

impl NativeFunction {
    pub fn new(name: StrId, callable: Box<dyn Fn(&[Value]) -> Value>) -> Self {
        Self { name, callable }
    }
}
