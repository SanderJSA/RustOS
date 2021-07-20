use super::eval;
use super::types::MalType;
use alloc::rc::Rc;
use alloc::{collections::BTreeMap, string::*};
use core::cell::RefCell;

pub type RcEnv = Rc<RefCell<Env>>;

pub struct Env {
    data: BTreeMap<String, MalType>,
    outer: Option<RcEnv>,
}

impl Env {
    pub fn new(outer: Option<RcEnv>) -> Env {
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
                if value_iter.next().is_some() {
                    panic!("Too many arguments given");
                }
            }
            _ => panic!("Incorrect binding structure"),
        }
    }
}

pub fn bind_list(env: &RcEnv, bindings: &[MalType]) {
    if let [MalType::Symbol(key), value, tail @ ..] = bindings {
        env.borrow_mut().set(key, eval(value, env));
        bind_list(env, tail);
    }
}
