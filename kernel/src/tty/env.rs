use super::types::MalType;
use alloc::rc::Rc;
use alloc::{collections::BTreeMap, string::*};
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

    pub fn set(&mut self, key: &str, val: MalType) {
        self.data.insert(key.to_string(), val);
    }

    pub fn get(&self, key: &str) -> Option<MalType> {
        self.data.get(key).map(|value| value.clone()).or_else(|| {
            self.outer
                .as_ref()
                .and_then(|outer_env| outer_env.borrow().get(key))
        })
    }

    pub fn bind(&mut self, args: MalType, values: MalType) {
        match (args, values) {
            (MalType::List(args), MalType::List(values)) => {
                let mut value_iter = values.into_iter();
                for arg in args.iter() {
                    if let MalType::Symbol(sym) = arg {
                        if sym == "&" {
                            self.set(sym, MalType::List(value_iter.collect()));
                            return;
                        }
                        self.set(sym, value_iter.next().unwrap());
                    }
                }
            }
            _ => panic!("Incorrect binding structure"),
        }
    }
}
