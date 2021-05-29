use super::types::MalType;
use alloc::{collections::BTreeMap, string::String};

pub struct Env<'a> {
    data: BTreeMap<String, MalType>,
    outer: Option<&'a Env<'a>>,
}

impl<'a> Env<'a> {
    pub fn new(outer: Option<&'a Env>) -> Env<'a> {
        Env {
            data: BTreeMap::new(),
            outer,
        }
    }

    pub fn set(&mut self, key: String, val: MalType) {
        self.data.insert(key, val);
    }

    pub fn find(&self, key: &str) -> Option<&Env> {
        if self.data.contains_key(key) {
            Some(self)
        } else {
            self.outer.as_ref().and_then(|env| env.find(key))
        }
    }

    pub fn get(&self, key: &str) -> Option<MalType> {
        self.find(key)
            .and_then(|env| env.data.get(key))
            .map(|value| value.clone())
    }
}
