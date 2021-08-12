use super::env::RcEnv;
use super::types::MalType;
use crate::file_system::File;
use crate::{exit_qemu, QemuExitCode};
use alloc::string::String;

pub fn init_core_env(env: &RcEnv) {
    init_builtins(env);
    super::rep("(def! not (fn* (a) (if a false true)))", env.clone());
    super::rep(
        "(def! load-file (fn* (f) (eval (read-string (str \"(do \" (slurp f) \"\nnil)\")))))",
        env.clone(),
    );
}
fn init_builtins(env: &RcEnv) {
    let builtins = [
        ("+", MalType::new_builtin(core_add, &["&"], env)),
        ("-", MalType::new_builtin(core_sub, &["&"], env)),
        ("*", MalType::new_builtin(core_mul, &["&"], env)),
        ("/", MalType::new_builtin(core_div, &["&"], env)),
        ("<", MalType::new_builtin(core_lt, &["&"], env)),
        ("<=", MalType::new_builtin(core_le, &["&"], env)),
        ("=", MalType::new_builtin(core_eq, &["&"], env)),
        ("=>", MalType::new_builtin(core_ge, &["&"], env)),
        (">", MalType::new_builtin(core_gt, &["&"], env)),
        ("list", MalType::new_builtin(core_list, &["&"], env)),
        ("prn", MalType::new_builtin(core_prn, &["a"], env)),
        ("str", MalType::new_builtin(core_str, &["&"], env)),
        ("shutdown", MalType::new_builtin(shutdown, &[], env)),
        (
            "read-string",
            MalType::new_builtin(read_string, &["a"], env),
        ),
        ("slurp", MalType::new_builtin(slurp, &["filename"], env)),
        ("ls", MalType::new_builtin(ls, &[], env)),
        (
            "write-to-file",
            MalType::new_builtin(write_to_file, &["filename", "content"], env),
        ),
        ("eval", MalType::new_builtin(eval, &["exp"], env)),
    ];
    let mut env_mut = env.borrow_mut();
    for (builtin_name, name) in builtins {
        env_mut.set(builtin_name, name);
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
    if let Some(MalType::String(filename)) = env.borrow().get("filename") {
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
fn eval(env: &RcEnv) -> MalType {
    super::eval(env.borrow().get("exp").unwrap(), env.clone())
}

fn core_str(env: &RcEnv) -> MalType {
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
