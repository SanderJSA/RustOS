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
        match self {
            MalType::Number(num) => write!(f, "{}", num),
            MalType::Symbol(sym) => write!(f, "{}", sym),
            MalType::List(list) => {
                write!(f, "(")?;
                let mut list_iter = list.iter();
                list_iter.next().map(|first_val| write!(f, "{}", first_val));
                for val in list_iter {
                    write!(f, " {}", val)?;
                }
                write!(f, ")")
            }
            MalType::Func { .. } => write!(f, "#<function>"),
            MalType::Builtin { .. } => write!(f, "#<builtin>"),
            MalType::Nil => write!(f, "nil"),
            MalType::Bool(true) => write!(f, "true"),
            MalType::Bool(false) => write!(f, "false"),
            MalType::String(string) => write!(f, "\"{}\"", string),
        }
    }
}
