use std::fmt::{self, Debug, Display, Formatter};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Identifier {
    namespace: Namespace,
    path: String,
}

#[derive(Clone, PartialEq, Eq, Hash)]
enum Namespace {
    Minecraft,
    Other(String),
}

impl Identifier {
    pub fn new(namespace: impl Into<String> + AsRef<str>, path: impl Into<String>) -> Self {
        Identifier {
            namespace: if namespace.as_ref() == "minecraft" {
                Namespace::Minecraft
            } else {
                Namespace::Other(namespace.into())
            },
            path: path.into(),
        }
    }

    pub fn namespace(&self) -> &str {
        match &self.namespace {
            Namespace::Minecraft => "minecraft",
            Namespace::Other(v) => v,
        }
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.namespace(), self.path)
    }
}

impl Debug for Identifier {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.namespace(), self.path)
    }
}
