use alloc::{string::String, vec::Vec};
use core::fmt::{self, Display, Formatter};

#[derive(Clone, PartialEq, Eq)]
pub enum MalType {
    Number(i64),
    Symbol(String),
    List(Vec<MalType>),
    Func(fn(MalType, MalType) -> MalType),
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
            MalType::Func(_) => write!(f, "#<function>"),
        }
    }
}
