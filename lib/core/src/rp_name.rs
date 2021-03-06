//! Describes a fully qualified name as a model

use super::RpVersionedPackage;
use std::fmt;

#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RpName {
    /// Alias used if the name was imported from another package.
    pub prefix: Option<String>,
    /// Package that name belongs to.
    pub package: RpVersionedPackage,
    /// Absolute parts of the name, from the root of the package.
    pub parts: Vec<String>,
}

impl RpName {
    pub fn new(prefix: Option<String>, package: RpVersionedPackage, parts: Vec<String>) -> RpName {
        RpName {
            prefix: prefix,
            package: package,
            parts: parts,
        }
    }

    pub fn extend<I>(&self, it: I) -> RpName
    where
        I: IntoIterator<Item = String>,
    {
        let mut parts = self.parts.clone();
        parts.extend(it);

        RpName {
            prefix: self.prefix.clone(),
            package: self.package.clone(),
            parts: parts,
        }
    }

    pub fn push(&self, part: String) -> RpName {
        let mut parts = self.parts.clone();
        parts.push(part);

        RpName {
            prefix: self.prefix.clone(),
            package: self.package.clone(),
            parts: parts,
        }
    }

    pub fn join<S: AsRef<str>>(&self, joiner: S) -> String {
        self.parts.join(joiner.as_ref())
    }

    /// Convert to a name without a prefix component.
    pub fn without_prefix(self) -> RpName {
        RpName {
            prefix: None,
            package: self.package,
            parts: self.parts,
        }
    }

    /// Localize name.
    ///
    /// Strips version of any type which is _not_ imported.
    pub fn localize(self) -> RpName {
        if self.prefix.is_some() {
            return self;
        }

        self.without_version()
    }

    /// Convert to a name without a version component.
    pub fn without_version(self) -> RpName {
        RpName {
            prefix: self.prefix,
            package: self.package.without_version(),
            parts: self.parts,
        }
    }

    pub fn with_package(self, package: RpVersionedPackage) -> RpName {
        RpName {
            prefix: self.prefix,
            package: package,
            parts: self.parts,
        }
    }

    /// Build a new name out if the given paths.
    pub fn with_parts(self, parts: Vec<String>) -> RpName {
        RpName {
            prefix: self.prefix,
            package: self.package,
            parts: parts,
        }
    }

    pub fn is_same(&self, other: &RpName) -> bool {
        self.package == other.package && self.parts == other.parts
    }
}

impl fmt::Display for RpName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref prefix) = self.prefix {
            write!(f, "{}::{}", prefix, self.parts.join("::"))
        } else {
            write!(f, "{}", self.parts.join("::"))
        }
    }
}
