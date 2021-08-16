use super::env::RcEnv;
use crate::file_system::File;
use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::string::ToString;
use alloc::{string::String, vec::Vec};
use core::cell::RefCell;
use core::fmt::{self, Display, Formatter};

#[derive(Clone)]
pub enum MalType {
    Number(i64),
    Symbol(String),
    List(Vec<MalType>),
    Bool(bool),
    String(String),
    Nil,
    Func {
        body: Box<MalType>,
        args: Box<MalType>,
        env: RcEnv,
        is_macro: bool,
    },
    Builtin {
        eval: fn(env: &RcEnv) -> MalType,
        args: Box<MalType>,
        env: RcEnv,
    },
    File(Rc<RefCell<File>>),
}

impl MalType {
    pub fn new_builtin(eval: fn(env: &RcEnv) -> MalType, args: &[&str], env: &RcEnv) -> MalType {
        MalType::Builtin {
            eval,
            args: Box::new(MalType::List(
                args.iter()
                    .map(|arg| MalType::Symbol(arg.to_string()))
                    .collect(),
            )),
            env: env.clone(),
        }
    }

    pub fn quote() -> MalType {
        MalType::Symbol("quote".to_string())
    }

    pub fn quasiquote() -> MalType {
        MalType::Symbol("quasiquote".to_string())
    }

    pub fn unquote() -> MalType {
        MalType::Symbol("unquote".to_string())
    }

    pub fn splice_unquote() -> MalType {
        MalType::Symbol("splice-unquote".to_string())
    }
}

impl Display for MalType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", super::pr_str(self, true))
    }
}

impl PartialEq for MalType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (MalType::Nil, MalType::Nil) => true,
            (MalType::Number(x), MalType::Number(y)) => x == y,
            (MalType::Bool(x), MalType::Bool(y)) => x == y,
            (MalType::String(x), MalType::String(y)) => x == y,
            (MalType::Symbol(x), MalType::Symbol(y)) => x == y,
            (MalType::List(x), MalType::List(y)) => x == y,
            (MalType::Func { .. }, MalType::Func { .. })
            | (MalType::Builtin { .. }, MalType::Builtin { .. }) => {
                panic!("Equality comparison not supported for Funcs and Builtins")
            }
            _ => false,
        }
    }
}
impl Eq for MalType {}

impl From<&str> for MalType {
    fn from(input: &str) -> Self {
        MalType::String(input.to_string())
    }
}

impl From<bool> for MalType {
    fn from(input: bool) -> Self {
        MalType::Bool(input)
    }
}
