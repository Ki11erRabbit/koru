
#[derive(Clone, PartialOrd, PartialEq, Eq, Ord, Hash)]
pub struct AttrSet {
    pub key: String,
    pub value: String,
}

impl AttrSet {
    pub fn new(key: String, value: String) -> Self {
        AttrSet {
            key,
            value,
        }
    }
}

impl std::fmt::Display for AttrSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "'{}'='{}'", self.key, self.value)
    }
}

impl std::fmt::Debug for AttrSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}