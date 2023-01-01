use crate::{
    chunk::Chunk,
    string_intern::{StrId, StringInterner},
    value::{Closure, Function, NativeFunction, Value},
};

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct FunctionId(pub usize);

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct ClosureId(pub usize);

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct NativeFunctionId(pub usize);

pub struct Memory {
    strings: StringInterner,
    functions: Vec<Function>,
    natives: Vec<NativeFunction>,
    closures: Vec<Closure>,
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            strings: StringInterner::with_capacity(16),
            functions: Vec::new(),
            natives: Vec::new(),
            closures: Vec::new(),
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

    pub fn function(&self, id: FunctionId) -> &Function {
        &self.functions[id.0]
    }

    pub fn function_mut(&mut self, id: FunctionId) -> &mut Function {
        &mut self.functions[id.0]
    }

    pub fn new_function(&mut self, name: &str) -> FunctionId {
        let id = self.functions.len();
        let name = self.string_id(name);
        self.functions.push(Function {
            arity: 0,
            chunk: Chunk::new(),
            name,
        });
        FunctionId(id)
    }

    pub fn closure(&self, id: ClosureId) -> &Closure {
        &self.closures[id.0]
    }

    pub fn closure_mut(&mut self, id: ClosureId) -> &mut Closure {
        &mut self.closures[id.0]
    }

    pub fn new_closure(&mut self, function: FunctionId) -> ClosureId {
        let id = self.closures.len();
        self.closures.push(Closure { function });
        ClosureId(id)
    }

    pub fn native(&self, id: NativeFunctionId) -> &NativeFunction {
        &self.natives[id.0]
    }

    pub fn new_native(
        &mut self,
        name: &str,
        function: impl Fn(&[Value]) -> Value + 'static,
    ) -> NativeFunctionId {
        let id = self.natives.len();
        let name = self.string_id(name);
        self.natives
            .push(NativeFunction::new(name, Box::new(function)));
        NativeFunctionId(id)
    }
}
