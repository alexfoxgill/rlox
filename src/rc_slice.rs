use std::{rc::Rc, ops::{Range, Deref}, fmt::Display};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RcSlice {
    string: Rc<str>,
    range: Range<usize>
}

impl RcSlice {
    pub fn new(string: Rc<str>, range: Range<usize>) -> Self { Self { string, range } }

    pub fn as_str(&self) -> &str {
        &* self
    }

    pub fn from_string(str: &str) -> RcSlice {
        RcSlice {
            string: Rc::from(str),
            range: 0..str.len()
        }
    }
}

impl Deref for RcSlice {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.string[self.range.clone()]
    }
}

impl Into<String> for RcSlice {
    fn into(self) -> String {
        self.as_str().into()
    }
}

impl<'a> Into<String> for &'a RcSlice {
    fn into(self) -> String {
        self.as_str().into()
    }
}

impl Display for RcSlice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}