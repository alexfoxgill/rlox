use std::{collections::HashMap, mem};

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct StrId(usize);

pub struct StringInterner {
    map: HashMap<&'static str, StrId>,
    vec: Vec<&'static str>,
    buf: String,
    full: Vec<String>
}

impl StringInterner {
    pub fn with_capacity(cap: usize) -> StringInterner {
        let cap = cap.next_power_of_two();
        StringInterner {
            map: HashMap::new(),
            vec: Vec::new(),
            buf: String::with_capacity(cap),
            full: Vec::new()
        }
    }

    pub fn intern(&mut self, name: &str) -> (StrId, &'static str) {
        if let Some(&id) = self.map.get(name) {
            return (id, self.vec[id.0]);
        }

        let name = unsafe { self.alloc(name) };
        let id = StrId(self.map.len());
        self.map.insert(name, id);
        self.vec.push(name);

        (id, name)
    }

    pub fn lookup(&self, id: StrId) -> &str {
        self.vec[id.0]
    }

    unsafe fn alloc(&mut self, name: &str) -> &'static str {
        let cap = self.buf.capacity();
        if cap < self.buf.len() + name.len() {
            let new_cap = (cap.max(name.len()) + 1).next_power_of_two();
            let new_buf = String::with_capacity(new_cap);
            let old_buf = mem::replace(&mut self.buf, new_buf);
            self.full.push(old_buf);
        }

        let interned = {
            let start = self.buf.len();
            self.buf.push_str(name);
            &self.buf[start..]
        };

        &*(interned as *const str)
    }

}