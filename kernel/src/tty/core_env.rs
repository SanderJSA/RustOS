use super::env::Env;
use super::types::MalType;
use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::{string::*, vec};
use core::cell::RefCell;

pub fn init_core_env(env: &Rc<RefCell<Env>>) {
    env.borrow_mut().set("+", init_num_op(core_add, env));
    env.borrow_mut().set("-", init_num_op(core_sub, env));
    env.borrow_mut().set("*", init_num_op(core_mul, env));
    env.borrow_mut().set("/", init_num_op(core_div, env));
    env.borrow_mut().set(
        "prn",
        MalType::Func {
            eval: core_prn,
            args: Box::new(MalType::List(vec![MalType::Symbol("a".to_string())])),
            body: Box::new(MalType::Nil),
            env: env.clone(),
        },
    );
    env.borrow_mut().set(
        "list",
        MalType::Func {
            eval: core_list,
            args: Box::new(MalType::List(vec![MalType::Symbol("a".to_string())])),
            body: Box::new(MalType::Nil),
            env: env.clone(),
        },
    );
}

fn init_num_op(
    eval_func: fn(ast: &MalType, env: &Rc<RefCell<Env>>) -> MalType,
    env: &Rc<RefCell<Env>>,
) -> MalType {
    MalType::Func {
        eval: eval_func,
        args: Box::new(MalType::List(vec![
            MalType::Symbol("a".to_string()),
            MalType::Symbol("b".to_string()),
        ])),
        body: Box::new(MalType::Nil),
        env: env.clone(),
    }
}

fn core_add(_: &MalType, env: &Rc<RefCell<Env>>) -> MalType {
    let mal_a = env.borrow().get("a").expect("symbol not found in env");
    let mal_b = env.borrow().get("b").expect("symbol not found in env");
    match (mal_a, mal_b) {
        (MalType::Number(a), MalType::Number(b)) => MalType::Number(a + b),
        _ => panic!("Operation can only be performed on numbers"),
    }
}

fn core_sub(_: &MalType, env: &Rc<RefCell<Env>>) -> MalType {
    let mal_a = env.borrow().get("a").expect("symbol not found in env");
    let mal_b = env.borrow().get("b").expect("symbol not found in env");
    match (mal_a, mal_b) {
        (MalType::Number(a), MalType::Number(b)) => MalType::Number(a - b),
        _ => panic!("Operation can only be performed on numbers"),
    }
}

fn core_mul(_: &MalType, env: &Rc<RefCell<Env>>) -> MalType {
    let mal_a = env.borrow().get("a").expect("symbol not found in env");
    let mal_b = env.borrow().get("b").expect("symbol not found in env");
    match (mal_a, mal_b) {
        (MalType::Number(a), MalType::Number(b)) => MalType::Number(a * b),
        _ => panic!("Operation can only be performed on numbers"),
    }
}

fn core_div(_: &MalType, env: &Rc<RefCell<Env>>) -> MalType {
    let mal_a = env.borrow().get("a").expect("symbol not found in env");
    let mal_b = env.borrow().get("b").expect("symbol not found in env");
    match (mal_a, mal_b) {
        (MalType::Number(a), MalType::Number(b)) => MalType::Number(a / b),
        _ => panic!("Operation can only be performed on numbers"),
    }
}

fn core_prn(_: &MalType, env: &Rc<RefCell<Env>>) -> MalType {
    super::print(env.borrow().get("a").expect("symbol not found in env"));
    MalType::Nil
}

fn core_list(values: &MalType, env: &Rc<RefCell<Env>>) -> MalType {
    values.clone()
}
