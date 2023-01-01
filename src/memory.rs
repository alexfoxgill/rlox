use crate::{
    string_intern::{StringInterner, StrId},
    value::{Function, NativeFunction, Closure},
};

pub struct Memory {
    strings: StringInterner,
    pub functions: Vec<Function>,
    pub natives: Vec<NativeFunction>,
    pub closures: Vec<Closure>
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            strings: StringInterner::with_capacity(16),
            functions: Vec::new(),
            natives: Vec::new(),
            closures: Vec::new()
        }
    }

    pub fn string_id(&mut self, string: &str) -> StrId {
        self.strings.intern(string).0
    }

    pub fn string_intern(&mut self, string: &str) -> &'static str {
        self.strings.intern(string).1
    }

    pub fn get_string(&self, id: StrId) -> &str {
        self.strings.lookup(id)
    }
}
