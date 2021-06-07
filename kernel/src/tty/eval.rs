use rustyline::error::ReadlineError;
use rustyline::Editor;

mod env;
mod reader;
mod types;

use env::Env;
use types::*;

fn read() -> Result<MalType, ReadlineError> {
    rl.readline("user> ")
        .map(|line| Reader::new(&line).read_form())
}

fn eval_ast(ast: MalType, env: &mut Env) -> MalType {
    match ast {
        MalType::Symbol(sym) => env.get(&sym).expect("Symbol not found in env"),
        MalType::List(list) => MalType::List(list.into_iter().map(|val| eval(val, env)).collect()),
        _ => ast,
    }
}

fn eval(ast: MalType, env: &mut Env) -> MalType {
    match ast {
        MalType::List(ref list) => match list.as_slice() {
            [] => eval_ast(ast, env),
            [MalType::Symbol(sym), MalType::Symbol(key), value] if sym == "def!" => {
                let value = eval(value.clone(), env);
                env.set(key.to_string(), value.clone());
                value
            }
            [MalType::Symbol(sym), MalType::List(bindings), exp] if sym == "let*" => {
                eval_let(bindings, exp, env)
            }
            [MalType::Symbol(sym), ..] if sym == "do" => eval_let(bindings, exp, env),
            _ => {
                if let MalType::List(list) = eval_ast(ast, env) {
                    let mut values = list.into_iter();
                    if let MalType::Func(func) = values.next().unwrap() {
                        return func(values.next().unwrap(), values.next().unwrap());
                    }
                }
                unreachable!();
            }
        },

        _ => eval_ast(ast, env),
    }
}

fn eval_let(bindings: &Vec<MalType>, exp: &MalType, env: &mut Env) -> MalType {
    let mut new_env = Env::new(Some(env));
    if let [MalType::Symbol(key), value] = bindings.as_slice() {
        let value = eval(value.clone(), &mut new_env);
        new_env.set(key.to_string(), value);
    } else {
        for pair in bindings {
            if let MalType::List(pair) = pair {
                if let [MalType::Symbol(key), value] = pair.as_slice() {
                    let value = eval(value.clone(), &mut new_env);
                    new_env.set(key.to_string(), value);
                }
            }
        }
    }
    eval(exp.clone(), &mut new_env)
}

fn print(ast: MalType) {
    println!("{}", ast);
}

fn main() {
    let mut rl = Editor::<()>::new();
    let mut env = Env::new(None);
    env.set(
        String::from("+"),
        MalType::Func(|a: MalType, b: MalType| match (a, b) {
            (MalType::Number(a), MalType::Number(b)) => MalType::Number(a + b),
            _ => MalType::Number(0),
        }),
    );
    env.set(
        String::from("-"),
        MalType::Func(|a: MalType, b: MalType| match (a, b) {
            (MalType::Number(a), MalType::Number(b)) => MalType::Number(a - b),
            _ => MalType::Number(0),
        }),
    );
    env.set(
        String::from("*"),
        MalType::Func(|a: MalType, b: MalType| match (a, b) {
            (MalType::Number(a), MalType::Number(b)) => MalType::Number(a * b),
            _ => MalType::Number(0),
        }),
    );
    env.set(
        String::from("/"),
        MalType::Func(|a: MalType, b: MalType| match (a, b) {
            (MalType::Number(a), MalType::Number(b)) => MalType::Number(a / b),
            _ => MalType::Number(0),
        }),
    );

    while let Ok(ast) = read(&mut rl) {
        let ast = eval(ast, &mut env);
        print(ast);
    }
}
