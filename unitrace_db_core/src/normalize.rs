use alloc::string::String;

use unitrace_db_handler::traits::Normalizer;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DefaultNormalizer;

impl DefaultNormalizer {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Normalizer for DefaultNormalizer {
    fn normalize(&self, input: &str) -> String {
        let mut out = String::with_capacity(input.len());
        for c in input.chars() {
            if c.is_ascii_alphanumeric() {
                out.push(c.to_ascii_uppercase());
            }
        }
        out
    }
}
