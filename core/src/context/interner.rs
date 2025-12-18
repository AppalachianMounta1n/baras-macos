use lasso::{Spur, ThreadedRodeo};
use std::sync::OnceLock;

/// Interned string key - 4 bytes instead of 24 for String.
pub type IStr = Spur;

/// Global string interner for combat log data.
static INTERNER: OnceLock<ThreadedRodeo> = OnceLock::new();

/// Get the global interner (initializes on first call).
pub fn interner() -> &'static ThreadedRodeo {
    INTERNER.get_or_init(ThreadedRodeo::default)
}

/// Intern a string, returning a key.
pub fn intern(s: &str) -> IStr {
    interner().get_or_intern(s)
}

/// Resolve an interned key back to a string.
pub fn resolve(key: IStr) -> &'static str {
    interner().resolve(&key)
}
