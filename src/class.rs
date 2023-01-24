#[derive(Debug, Clone, PartialEq, Hash)]
pub(crate) struct Class {
    name: String,
}

impl Class {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
