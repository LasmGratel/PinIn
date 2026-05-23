use crate::searchers::{Logic, Searcher};
use crate::utils::{Accelerator, Compressor};
use crate::PinIn;

pub struct SimpleSearcher<T> {
    pub(crate) context: PinIn,
    pub(crate) objects: Vec<T>,
    pub(crate) names: Vec<String>,
    pub(crate) strs: Compressor,
    pub(crate) acc: Accelerator,
    pub(crate) logic: Logic,
    pub(crate) last_modification: u64,
}

impl<T> SimpleSearcher<T> {
    pub fn new(logic: Logic, context: PinIn) -> Self {
        let acc = Accelerator::new(context.clone());
        Self {
            context,
            objects: Vec::new(),
            names: Vec::new(),
            strs: Compressor::default(),
            acc,
            logic,
            last_modification: 0,
        }
    }

    pub fn reset(&mut self) {
        self.acc.reset();
        self.last_modification = self.context.modification();
    }

    pub(crate) fn renew(&mut self) {
        if self.last_modification != self.context.modification() {
            self.reset();
        }
    }
}

impl<T> Searcher<T> for SimpleSearcher<T> {
    fn put(&mut self, name: &str, identifier: T) {
        self.names.push(name.to_string());
        self.strs.put(name);
        self.objects.push(identifier);
    }

    fn search<'a>(&'a mut self, name: &str) -> Vec<&'a T> {
        self.renew();
        let context = self.context.clone();
        self.names.iter().zip(self.objects.iter()).filter_map(|(n, v)| self.logic.test(&context, n, name).then_some(v)).collect()
    }

    fn context(&self) -> PinIn {
        self.context.clone()
    }
}
