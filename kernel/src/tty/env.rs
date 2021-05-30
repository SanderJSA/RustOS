use super::types::MalType;
use alloc::rc::Rc;
use alloc::{collections::BTreeMap, string::String};
use core::cell::RefCell;

pub struct Env {
    data: BTreeMap<String, MalType>,
    outer: Option<Rc<RefCell<Env>>>,
}

impl Env {
    pub fn new(outer: Option<Rc<RefCell<Env>>>) -> Env {
        Env {
            data: BTreeMap::new(),
            outer,
        }
    }

    pub fn set(&mut self, key: String, val: MalType) {
        self.data.insert(key, val);
    }

    pub fn get(&self, key: &str) -> Option<MalType> {
        self.data.get(key).map(|value| value.clone()).or_else(|| {
            self.outer
                .as_ref()
                .and_then(|outer_env| outer_env.borrow().get(key))
        })
    }

    pub fn bind(&mut self, args: MalType, values: MalType) {
        if let MalType::List(args) = args {
            if let MalType::List(values) = values {
                for (arg, value) in args.into_iter().zip(values.into_iter()) {
                    if let MalType::Symbol(sym) = arg {
                        self.set(sym, value);
                    }
                }
            }
        }
    }
}
