mod cached;
mod simple;
mod tree;

use crate::PinIn;

pub use cached::CachedSearcher;
pub use simple::SimpleSearcher;
pub use tree::TreeSearcher;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Logic {
    Begin,
    Contain,
    Equal,
}

impl Logic {
    pub fn test(self, p: &PinIn, s1: &str, s2: &str) -> bool {
        match self {
            Logic::Begin => p.begins(s1, s2),
            Logic::Contain => p.contains(s1, s2),
            Logic::Equal => p.matches(s1, s2),
        }
    }

    pub fn raw(self, s1: &str, s2: &str) -> bool {
        match self {
            Logic::Begin => s1.starts_with(s2),
            Logic::Contain => s1.contains(s2),
            Logic::Equal => s1 == s2,
        }
    }

    pub(crate) fn filter_logic(self) -> Logic {
        match self {
            Logic::Equal => Logic::Begin,
            other => other,
        }
    }
}

pub trait Searcher<T> {
    fn put(&mut self, name: &str, identifier: T);
    fn search<'a>(&'a mut self, name: &str) -> Vec<&'a T>;
    fn context(&self) -> PinIn;
}
