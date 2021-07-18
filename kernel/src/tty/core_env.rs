use super::env::Env;
use super::types::MalType;
use crate::{exit_qemu, file_system, print, println, QemuExitCode};
use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::{string::*, vec};
use core::cell::RefCell;

pub fn init_core_env(env: &Rc<RefCell<Env>>) {
    env.borrow_mut().set("+", init_num_op(core_add, env));
    env.borrow_mut().set("-", init_num_op(core_sub, env));
    env.borrow_mut().set("*", init_num_op(core_mul, env));
    env.borrow_mut().set("/", init_num_op(core_div, env));
    env.borrow_mut().set("<", init_num_op(core_lt, env));
    env.borrow_mut().set("<=", init_num_op(core_le, env));
    env.borrow_mut().set("=", init_num_op(core_eq, env));
    env.borrow_mut().set("=>", init_num_op(core_ge, env));
    env.borrow_mut().set(">", init_num_op(core_gt, env));
    env.borrow_mut().set("list", init_num_op(core_list, env));
    env.borrow_mut().set(
        "prn",
        MalType::Func {
            eval: core_prn,
            args: Box::new(MalType::List(vec![MalType::Symbol("a".to_string())])),
            body: Box::new(MalType::Nil),
            env: env.clone(),
        },
    );
    /*
    env.borrow_mut().set(
        "shutdown",
        MalType::Func {
            eval: shutdown,
            args: Box::new(MalType::List(vec![])),
            body: Box::new(MalType::Nil),
            env: env.clone(),
        },
    );
    */
}

fn init_num_op(
    eval_func: fn(ast: &MalType, env: &Rc<RefCell<Env>>) -> MalType,
    env: &Rc<RefCell<Env>>,
) -> MalType {
    MalType::Func {
        eval: eval_func,
        args: Box::new(MalType::List(vec![MalType::Symbol("&".to_string())])),
        body: Box::new(MalType::Nil),
        env: env.clone(),
    }
}

fn core_add(_: &MalType, env: &Rc<RefCell<Env>>) -> MalType {
    if let MalType::List(values) = env.borrow().get("&").expect("symbol not found in env") {
        let res = values
            .iter()
            .map(|value| match value {
                MalType::Number(num) => *num,
                _ => panic!("Value is not a number"),
            })
            .sum();
        MalType::Number(res)
    } else {
        unreachable!()
    }
}

fn core_sub(_: &MalType, env: &Rc<RefCell<Env>>) -> MalType {
    if let MalType::List(values) = env.borrow().get("&").expect("symbol not found in env") {
        let res = values
            .iter()
            .map(|value| match value {
                MalType::Number(num) => *num,
                _ => panic!("Value is not a number"),
            })
            .reduce(|a, b| a - b)
            .unwrap();
        MalType::Number(res)
    } else {
        unreachable!()
    }
}

fn core_mul(_: &MalType, env: &Rc<RefCell<Env>>) -> MalType {
    if let MalType::List(values) = env.borrow().get("&").expect("symbol not found in env") {
        let res = values
            .iter()
            .map(|value| match value {
                MalType::Number(num) => *num,
                _ => panic!("Value is not a number"),
            })
            .reduce(|a, b| a * b)
            .unwrap();
        MalType::Number(res)
    } else {
        unreachable!()
    }
}

fn core_div(_: &MalType, env: &Rc<RefCell<Env>>) -> MalType {
    if let MalType::List(values) = env.borrow().get("&").expect("symbol not found in env") {
        let res = values
            .iter()
            .map(|value| match value {
                MalType::Number(num) => *num,
                _ => panic!("Value is not a number"),
            })
            .reduce(|a, b| a / b)
            .unwrap();
        MalType::Number(res)
    } else {
        unreachable!()
    }
}

fn core_prn(_: &MalType, env: &Rc<RefCell<Env>>) -> MalType {
    super::print(env.borrow().get("a").expect("symbol not found in env"));
    MalType::Nil
}

fn core_list(_: &MalType, env: &Rc<RefCell<Env>>) -> MalType {
    env.borrow()
        .get("&")
        .expect("symbol not found in env")
        .clone()
}

fn core_lt(_: &MalType, env: &Rc<RefCell<Env>>) -> MalType {
    let args = env.borrow().get("&").expect("symbol not found in env");
    if let MalType::List(list) = args {
        if let [MalType::Number(left), MalType::Number(right)] = &list[..] {
            return MalType::Bool(left < right);
        }
    }
    unreachable!()
}

fn core_le(_: &MalType, env: &Rc<RefCell<Env>>) -> MalType {
    let args = env.borrow().get("&").expect("symbol not found in env");
    if let MalType::List(list) = args {
        if let [MalType::Number(left), MalType::Number(right)] = &list[..] {
            return MalType::Bool(left <= right);
        }
    }
    unreachable!()
}

fn core_eq(_: &MalType, env: &Rc<RefCell<Env>>) -> MalType {
    let args = env.borrow().get("&").expect("symbol not found in env");
    if let MalType::List(list) = args {
        if let [MalType::Number(left), MalType::Number(right)] = &list[..] {
            return MalType::Bool(left == right);
        }
    }
    unreachable!()
}

fn core_ge(_: &MalType, env: &Rc<RefCell<Env>>) -> MalType {
    let args = env.borrow().get("&").expect("symbol not found in env");
    if let MalType::List(list) = args {
        if let [MalType::Number(left), MalType::Number(right)] = &list[..] {
            return MalType::Bool(left >= right);
        }
    }
    unreachable!()
}

fn core_gt(_: &MalType, env: &Rc<RefCell<Env>>) -> MalType {
    let args = env.borrow().get("&").expect("symbol not found in env");
    if let MalType::List(list) = args {
        if let [MalType::Number(left), MalType::Number(right)] = &list[..] {
            return MalType::Bool(left > right);
        }
    }
    unreachable!()
}

fn shutdown(_: &MalType, _: &Rc<RefCell<Env>>) -> MalType {
    exit_qemu(QemuExitCode::Success)
}
