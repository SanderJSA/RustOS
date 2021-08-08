use super::env::RcEnv;
use super::types::MalType;
use crate::file_system::File;
use crate::{exit_qemu, QemuExitCode};
use alloc::boxed::Box;
use alloc::{string::*, vec};

pub fn init_core_env(env: &RcEnv) {
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
        MalType::Builtin {
            eval: core_prn,
            args: Box::new(MalType::List(vec![MalType::Symbol("a".to_string())])),
            env: env.clone(),
        },
    );
    env.borrow_mut().set(
        "str",
        MalType::Builtin {
            eval: str_builtin,
            args: Box::new(MalType::List(vec![MalType::Symbol("&".to_string())])),
            env: env.clone(),
        },
    );
    env.borrow_mut().set(
        "shutdown",
        MalType::Builtin {
            eval: shutdown,
            args: Box::new(MalType::List(vec![])),
            env: env.clone(),
        },
    );
    env.borrow_mut().set(
        "read-string",
        MalType::Builtin {
            eval: read_string,
            args: Box::new(MalType::List(vec![MalType::Symbol("a".to_string())])),
            env: env.clone(),
        },
    );
    env.borrow_mut().set(
        "slurp",
        MalType::Builtin {
            eval: slurp,
            args: Box::new(MalType::List(vec![MalType::Symbol("a".to_string())])),
            env: env.clone(),
        },
    );
    env.borrow_mut().set(
        "ls",
        MalType::Builtin {
            eval: ls,
            args: Box::new(MalType::List(vec![])),
            env: env.clone(),
        },
    );
    env.borrow_mut().set(
        "write-to-file",
        MalType::Builtin {
            eval: write_to_file,
            args: Box::new(MalType::List(vec![
                MalType::Symbol("filename".to_string()),
                MalType::Symbol("content".to_string()),
            ])),
            env: env.clone(),
        },
    );

    super::rep("(def! not (fn* (a) (if a false true)))", env.clone());
}

fn init_num_op(eval_func: fn(env: &RcEnv) -> MalType, env: &RcEnv) -> MalType {
    MalType::Builtin {
        eval: eval_func,
        args: Box::new(MalType::List(vec![MalType::Symbol("&".to_string())])),
        env: env.clone(),
    }
}

fn core_add(env: &RcEnv) -> MalType {
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

fn core_sub(env: &RcEnv) -> MalType {
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

fn core_mul(env: &RcEnv) -> MalType {
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

fn core_div(env: &RcEnv) -> MalType {
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

fn core_prn(env: &RcEnv) -> MalType {
    super::print(&env.borrow().get("a").expect("symbol not found in env"));
    MalType::Nil
}

fn core_list(env: &RcEnv) -> MalType {
    env.borrow().get("&").expect("symbol not found in env")
}

fn core_lt(env: &RcEnv) -> MalType {
    let args = env.borrow().get("&").expect("symbol not found in env");
    if let MalType::List(list) = args {
        if let [MalType::Number(left), MalType::Number(right)] = &list[..] {
            return MalType::Bool(left < right);
        }
    }
    unreachable!()
}

fn core_le(env: &RcEnv) -> MalType {
    let args = env.borrow().get("&").expect("symbol not found in env");
    if let MalType::List(list) = args {
        if let [MalType::Number(left), MalType::Number(right)] = &list[..] {
            return MalType::Bool(left <= right);
        }
    }
    unreachable!()
}

fn core_eq(env: &RcEnv) -> MalType {
    let args = env.borrow().get("&").expect("symbol not found in env");
    if let MalType::List(list) = args {
        if let [MalType::Number(left), MalType::Number(right)] = &list[..] {
            return MalType::Bool(left == right);
        }
    }
    unreachable!()
}

fn core_ge(env: &RcEnv) -> MalType {
    let args = env.borrow().get("&").expect("symbol not found in env");
    if let MalType::List(list) = args {
        if let [MalType::Number(left), MalType::Number(right)] = &list[..] {
            return MalType::Bool(left >= right);
        }
    }
    unreachable!()
}

fn core_gt(env: &RcEnv) -> MalType {
    let args = env.borrow().get("&").expect("symbol not found in env");
    if let MalType::List(list) = args {
        if let [MalType::Number(left), MalType::Number(right)] = &list[..] {
            return MalType::Bool(left > right);
        }
    }
    unreachable!()
}

fn shutdown(_: &RcEnv) -> MalType {
    exit_qemu(QemuExitCode::Success)
}

fn read_string(env: &RcEnv) -> MalType {
    if let Some(MalType::String(str)) = env.borrow().get("a") {
        super::read_str(&str)
    } else {
        panic!("read-string: Expected a string argument");
    }
}

fn slurp(env: &RcEnv) -> MalType {
    if let Some(MalType::String(filename)) = env.borrow().get("a") {
        let mut file = File::open(&filename).expect("Could not open file");
        let mut content = String::with_capacity(file.get_size());
        for _ in 0..file.get_size() {
            content.push('\0');
        }
        unsafe {
            // as_bytes_mut is safe as we can only handle ASCII
            file.read(content.as_bytes_mut());
        }
        MalType::String(content)
    } else {
        panic!("slurp: Expected a string argument");
    }
}

fn write_to_file(env: &RcEnv) -> MalType {
    if let (Some(MalType::String(filename)), Some(MalType::String(content))) =
        (env.borrow().get("filename"), env.borrow().get("content"))
    {
        let mut file = File::create(&filename).expect("Could not open file");
        file.write(content.as_bytes());
        MalType::Nil
    } else {
        panic!("write-to-file: Expected a string argument");
    }
}

fn str_builtin(env: &RcEnv) -> MalType {
    if let Some(MalType::List(list)) = env.borrow().get("&") {
        MalType::String(list.iter().map(|ast| super::pr_str(ast, false)).collect())
    } else {
        unreachable!();
    }
}

//TEMP
fn ls(_: &RcEnv) -> MalType {
    crate::file_system::ls();
    MalType::Nil
}
