use crate::searchers::{Logic, Searcher, SimpleSearcher};
use crate::PinIn;

pub struct TreeSearcher<T> {
    inner: SimpleSearcher<T>,
}

impl<T> TreeSearcher<T> {
    pub fn new(logic: Logic, context: PinIn) -> Self {
        Self { inner: SimpleSearcher::new(logic, context) }
    }

    pub fn refresh(&mut self) {
        self.inner.reset();
    }
}

impl<T> Searcher<T> for TreeSearcher<T> {
    fn put(&mut self, name: &str, identifier: T) {
        self.inner.put(name, identifier);
    }

    fn search<'a>(&'a mut self, name: &str) -> Vec<&'a T> {
        self.inner.search(name)
    }

    fn context(&self) -> PinIn {
        self.inner.context()
    }
}
