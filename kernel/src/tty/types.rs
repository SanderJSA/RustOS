use super::env::Env;
use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::{string::String, vec::Vec};
use core::cell::RefCell;
use core::fmt::{self, Display, Formatter};

#[derive(Clone)]
pub enum MalType {
    Number(i64),
    Symbol(String),
    List(Vec<MalType>),
    Bool(bool),
    Nil,
    Func {
        args: Box<MalType>,
        body: Box<MalType>,
        env: Rc<RefCell<Env>>,
    },
    Builtin {
        eval: fn(ast: &MalType, env: &Rc<RefCell<Env>>) -> MalType,
        args: Box<MalType>,
        body: Box<MalType>,
        env: Rc<RefCell<Env>>,
    },
}

impl MalType {
    pub fn eval_func(self, values: &[MalType]) -> MalType {
        if let MalType::Builtin {
            eval,
            args,
            body,
            env,
        } = self
        {
            let mut env = Env::new(Some(env));
            env.bind(&args, values);
            eval(&body, &Rc::new(RefCell::new(env)))
        } else {
            panic!("Not a builtin");
        }
    }
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
            MalType::Nil => write!(f, "nil"),
            MalType::Bool(true) => write!(f, "true"),
            MalType::Bool(false) => write!(f, "false"),
        }
    }
}
