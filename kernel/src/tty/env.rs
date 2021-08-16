use super::eval;
use super::types::MalType;
use alloc::rc::Rc;
use alloc::{collections::BTreeMap, string::*};
use core::cell::RefCell;
use core::fmt::{self, Display, Formatter};

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
        self.data.get(key).cloned().or_else(|| {
            self.outer
                .as_ref()
                .and_then(|outer_env| outer_env.borrow().get(key))
        })
    }

    pub fn bind(&mut self, args: &MalType, values: &[MalType]) {
        match args {
            MalType::List(args) => {
                let mut value_iter = values.iter().cloned();
                for arg in args.iter() {
                    if let MalType::Symbol(sym) = arg {
                        if sym == "&" {
                            if let Some(MalType::Symbol(bind_sym)) = args.last() {
                                self.set(bind_sym, MalType::List(value_iter.collect()));
                                return;
                            } else {
                                panic!("Expected symbol after \"&\"");
                            }
                        }
                        self.set(sym, value_iter.next().expect("Not enough arguments given"));
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

impl Display for Env {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.data.keys())?;
        if let Some(outer) = self.outer.as_ref() {
            write!(f, "{}", outer.borrow())
        } else {
            Ok(())
        }
    }
}

pub fn bind_list(env: RcEnv, bindings: &[MalType]) {
    if let [MalType::Symbol(key), value, tail @ ..] = bindings {
        let value = eval(value.clone(), env.clone());
        env.borrow_mut().set(key, value);
        bind_list(env, tail);
    }
}
