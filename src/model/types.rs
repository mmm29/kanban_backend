#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UniqueId(i64);

impl UniqueId {
    pub const fn from_raw(raw: i64) -> Self {
        Self(raw)
    }

    pub const fn raw(&self) -> i64 {
        self.0
    }
}
