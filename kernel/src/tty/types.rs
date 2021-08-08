use super::env::RcEnv;
use alloc::boxed::Box;
use alloc::{string::String, vec::Vec};
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
        args: Box<MalType>,
        body: Box<MalType>,
        env: RcEnv,
    },
    Builtin {
        eval: fn(env: &RcEnv) -> MalType,
        args: Box<MalType>,
        env: RcEnv,
    },
}

impl Display for MalType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", super::pr_str(self, true))
    }
}
