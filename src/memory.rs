use crate::{
    string_intern::StringInterner,
    value::{Function, NativeFunction},
};

pub struct Memory {
    pub strings: StringInterner,
    pub functions: Vec<Function>,
    pub natives: Vec<NativeFunction>,
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            strings: StringInterner::with_capacity(16),
            functions: Vec::new(),
            natives: Vec::new(),
        }
    }
}
