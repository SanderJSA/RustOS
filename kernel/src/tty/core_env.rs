use super::env::RcEnv;
use super::types::MalType;
use crate::file_system::{read_dir, File};
use crate::{exit_qemu, QemuExitCode};
use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::RefCell;

pub fn init_core_env(env: &RcEnv) {
    init_builtins(env);
    super::rep("(def! not (fn* (a) (if a false true)))", env.clone());
    super::rep(
        "(def! load-file (fn* (f) (eval (read-string (str \"(do \" (slurp f) \"\nnil)\")))))",
        env.clone(),
    );

    super::rep(
        "(defmacro! cond (fn* (& xs)
            (if (> (count xs) 0)
                (list 'if (first xs)
                    (if (> (count xs) 1)
                        (nth xs 1)
                        (throw \"odd number of forms to cond\"))
                    (cons 'cond (rest (rest xs)))))))",
        env.clone(),
    );

    super::rep(
        "(defmacro! doseq (fn* (seq-exprs & body)
            (if (= (count seq-exprs) 2)
                `(let*
                    (values ~(nth seq-exprs 1)
                    ~(first seq-exprs) (first values))

                    (if (not (= (count values) 0))
                        (do
                            ~@body
                            (doseq (~(first seq-exprs) (rest values)) ~@body)))))))",
        env.clone(),
    );
}
fn init_builtins(env: &RcEnv) {
    let builtins = [
        // Operators
        ("+", MalType::new_builtin(core_add, &["&", "vals"], env)),
        ("-", MalType::new_builtin(core_sub, &["&", "vals"], env)),
        ("*", MalType::new_builtin(core_mul, &["&", "vals"], env)),
        ("/", MalType::new_builtin(core_div, &["&", "vals"], env)),
        ("<", MalType::new_builtin(core_lt, &["&", "vals"], env)),
        ("<=", MalType::new_builtin(core_le, &["&", "vals"], env)),
        ("=", MalType::new_builtin(core_eq, &["&", "vals"], env)),
        ("=>", MalType::new_builtin(core_ge, &["&", "vals"], env)),
        (">", MalType::new_builtin(core_gt, &["&", "vals"], env)),
        // List
        ("list", MalType::new_builtin(core_list, &["&", "args"], env)),
        ("first", MalType::new_builtin(core_first, &["list"], env)),
        ("rest", MalType::new_builtin(core_rest, &["list"], env)),
        ("cons", MalType::new_builtin(core_cons, &["x", "seq"], env)),
        ("conj", MalType::new_builtin(core_conj, &["coll", "x"], env)),
        ("count", MalType::new_builtin(core_count, &["coll"], env)),
        (
            "nth",
            MalType::new_builtin(core_nth, &["coll", "index"], env),
        ),
        // String
        ("str", MalType::new_builtin(core_str, &["&", "args"], env)),
        // IO
        ("prn", MalType::new_builtin(core_prn, &["a"], env)),
        (
            "read-string",
            MalType::new_builtin(read_string, &["a"], env),
        ),
        ("slurp", MalType::new_builtin(slurp, &["filename"], env)),
        (
            "spit",
            MalType::new_builtin(spit, &["filename", "content"], env),
        ),
        ("File", MalType::new_builtin(file, &["filename"], env)),
        (
            ".listFiles",
            MalType::new_builtin(list_files, &["file"], env),
        ),
        // Misc
        ("eval", MalType::new_builtin(eval, &["exp"], env)),
        ("ls", MalType::new_builtin(ls, &[], env)),
        ("shutdown", MalType::new_builtin(shutdown, &[], env)),
    ];
    let mut env_mut = env.borrow_mut();
    for (builtin_name, name) in builtins {
        env_mut.set(builtin_name, name);
    }
}

fn get_arg(env: &RcEnv, arg: &str) -> MalType {
    env.borrow()
        .get(arg)
        .unwrap_or_else(|| panic!("symbol \"{}\" not found in env", arg))
}

fn get_variadic(env: &RcEnv, arg: &str) -> Vec<MalType> {
    if let MalType::List(list) = get_arg(env, arg) {
        list
    } else {
        unreachable!()
    }
}

fn core_add(env: &RcEnv) -> MalType {
    let res = get_variadic(env, "vals")
        .iter()
        .map(|value| match value {
            MalType::Number(num) => *num,
            _ => panic!("Value is not a number"),
        })
        .sum();
    MalType::Number(res)
}

fn core_sub(env: &RcEnv) -> MalType {
    let res = get_variadic(env, "vals")
        .iter()
        .map(|value| match value {
            MalType::Number(num) => *num,
            _ => panic!("Value is not a number"),
        })
        .reduce(|a, b| a - b)
        .unwrap();
    MalType::Number(res)
}

fn core_mul(env: &RcEnv) -> MalType {
    let res = get_variadic(env, "vals")
        .iter()
        .map(|value| match value {
            MalType::Number(num) => *num,
            _ => panic!("Value is not a number"),
        })
        .reduce(|a, b| a * b)
        .unwrap();
    MalType::Number(res)
}

fn core_div(env: &RcEnv) -> MalType {
    let res = get_variadic(env, "vals")
        .iter()
        .map(|value| match value {
            MalType::Number(num) => *num,
            _ => panic!("Value is not a number"),
        })
        .reduce(|a, b| a / b)
        .unwrap();
    MalType::Number(res)
}

fn core_prn(env: &RcEnv) -> MalType {
    super::print(&get_arg(env, "a"));
    MalType::Nil
}

fn core_lt(env: &RcEnv) -> MalType {
    if let [MalType::Number(left), MalType::Number(right)] = get_variadic(env, "vals").as_slice() {
        MalType::Bool(left < right)
    } else {
        panic!("Can only perform comparison on Numbers");
    }
}

fn core_le(env: &RcEnv) -> MalType {
    if let [MalType::Number(left), MalType::Number(right)] = get_variadic(env, "vals").as_slice() {
        MalType::Bool(left <= right)
    } else {
        panic!("Can only perform comparison on Numbers");
    }
}

fn core_eq(env: &RcEnv) -> MalType {
    if let [MalType::Number(left), MalType::Number(right)] = get_variadic(env, "vals").as_slice() {
        MalType::Bool(left == right)
    } else {
        panic!("Can only perform comparison on Numbers");
    }
}

fn core_ge(env: &RcEnv) -> MalType {
    if let [MalType::Number(left), MalType::Number(right)] = get_variadic(env, "vals").as_slice() {
        MalType::Bool(left >= right)
    } else {
        panic!("Can only perform comparison on Numbers");
    }
}

fn core_gt(env: &RcEnv) -> MalType {
    if let [MalType::Number(left), MalType::Number(right)] = get_variadic(env, "vals").as_slice() {
        MalType::Bool(left > right)
    } else {
        panic!("Can only perform comparison on Numbers");
    }
}

fn core_list(env: &RcEnv) -> MalType {
    get_arg(env, "args")
}

fn core_first(env: &RcEnv) -> MalType {
    match get_arg(env, "list") {
        MalType::Nil => MalType::Nil,
        MalType::List(list) => list.into_iter().next().unwrap_or(MalType::Nil),
        _ => panic!("first: Unexpected argument type"),
    }
}

fn core_rest(env: &RcEnv) -> MalType {
    match get_arg(env, "list") {
        MalType::Nil => MalType::List(alloc::vec![]),
        MalType::List(mut list) => {
            list.remove(0);
            MalType::List(list)
        }
        _ => panic!("rest: Unexpected argument type"),
    }
}

fn core_cons(env: &RcEnv) -> MalType {
    match (get_arg(env, "x"), get_arg(env, "seq")) {
        (x, MalType::List(mut list)) => {
            list.insert(0, x);
            MalType::List(list)
        }
        _ => panic!("cons: Unexpected argument type"),
    }
}

fn core_conj(env: &RcEnv) -> MalType {
    match (get_arg(env, "coll"), get_arg(env, "x")) {
        (MalType::List(mut coll), MalType::List(x)) => {
            coll.extend(x);
            MalType::List(coll)
        }
        _ => panic!("conj: Unexpected argument type"),
    }
}

fn core_count(env: &RcEnv) -> MalType {
    match get_arg(env, "coll") {
        MalType::List(list) => MalType::Number(list.len() as i64),
        _ => panic!("count: Unexpected argument type"),
    }
}

fn core_nth(env: &RcEnv) -> MalType {
    match (get_arg(env, "coll"), get_arg(env, "index")) {
        (MalType::List(mut coll), MalType::Number(index)) => coll.swap_remove(index as usize),
        _ => panic!("cons: Unexpected argument type"),
    }
}

fn shutdown(_: &RcEnv) -> MalType {
    exit_qemu(QemuExitCode::Success)
}

fn read_string(env: &RcEnv) -> MalType {
    if let MalType::String(str) = get_arg(env, "a") {
        super::read_str(&str)
    } else {
        panic!("read-string: Expected a string argument");
    }
}

fn slurp(env: &RcEnv) -> MalType {
    if let MalType::String(filename) = get_arg(env, "filename") {
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

fn spit(env: &RcEnv) -> MalType {
    if let (MalType::String(filename), MalType::String(content)) =
        (get_arg(env, "filename"), get_arg(env, "content"))
    {
        let mut file = File::create(&filename).expect("Could not open file");
        file.write(content.as_bytes());
        MalType::Nil
    } else {
        panic!("spit: Invalid argument type");
    }
}

fn eval(env: &RcEnv) -> MalType {
    super::eval(get_arg(env, "exp"), env.clone())
}

fn core_str(env: &RcEnv) -> MalType {
    MalType::String(
        get_variadic(env, "args")
            .iter()
            .map(|ast| super::pr_str(ast, false))
            .collect(),
    )
}

fn file(env: &RcEnv) -> MalType {
    if let MalType::String(filename) = get_arg(env, "filename") {
        let file = File::open(&filename).expect("Could not open file");
        MalType::File(Rc::new(RefCell::new(file)))
    } else {
        panic!("File: Invalid argument type");
    }
}

fn list_files(env: &RcEnv) -> MalType {
    if let MalType::File(file) = get_arg(env, "file") {
        let entries = read_dir(file.borrow().get_path()).unwrap();
        MalType::List(
            entries
                .map(|entry| MalType::File(Rc::new(RefCell::new(File::new(entry)))))
                .collect(),
        )
    } else {
        panic!(".listFiles: Invalid argument type");
    }
}

//TEMP
fn ls(_: &RcEnv) -> MalType {
    crate::file_system::ls();
    MalType::Nil
}
