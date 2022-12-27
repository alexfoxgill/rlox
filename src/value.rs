use core::fmt;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64)
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
             Value::Nil => write!(f, "nil"),
             Value::Bool(b) => write!(f, "{b}"),
             Value::Number(n) => write!(f, "{n}")
        }
    }
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
