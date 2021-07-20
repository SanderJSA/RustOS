//! This module implements a tty

/*
/// Start and run tty
pub fn run_tty() {
    // Set up shell
    println!(
        "     .~~~~`\\~~\\
     ;       ~~ \\
     |           ;
 ,--------,______|---.
/          \\-----`    \\
`.__________`-_______-'
           {}\n",
        1 as char
    );

    println!("Howdy, welcome to RustOS");

    // Run shell
    loop {
        print!("> ");
        let input = readline();

        match input.split_whitespace().nth(0).unwrap() {
            "poweroff" => exit_qemu(QemuExitCode::Success),
            "ls" => file_system::ls(),
            "touch" => {
                let data: [u8; 0] = [];
                file_system::add_file(input.split_whitespace().nth(1).unwrap(), &data, 0)
            }
            "help" => println!(
                "RustOS tty v1.0\n\
                ls         list files in current directory\n\
                touch FILE Update the access and modification times of each FILE to the current time.\n\
                poweroff   Power off the machine\n\
                "
            ),
            _ => print!("Unknown command: {}", input),
        }
    }
}
*/

mod core_env;
mod env;
mod reader;
mod types;

use crate::alloc::vec::Vec;
use crate::driver::ps2_keyboard::readline;
use crate::{file_system, print, println};
use alloc::boxed::Box;
use alloc::rc::Rc;
use core::cell::RefCell;
use env::Env;
use reader::Reader;
use types::*;

fn read() -> MalType {
    print!("root> ");
    let line = readline();
    Reader::new(&line[..line.len() - 1]).read_form()
}

fn eval_ast(ast: &MalType, env: &Rc<RefCell<Env>>) -> MalType {
    match ast {
        MalType::Symbol(sym) => env.borrow().get(sym).expect("Symbol not found in env"),
        MalType::List(list) => MalType::List(list.iter().map(|val| eval(val, env)).collect()),
        _ => ast.clone(),
    }
}

fn eval(mut ast: &MalType, mut env: &Rc<RefCell<Env>>) -> MalType {
    loop {
        match ast {
            MalType::List(list) => match list.as_slice() {
                [] => eval_ast(ast, env),
                [MalType::Symbol(sym), MalType::Symbol(key), value] if sym == "def!" => {
                    let value = eval(value, env);
                    env.borrow_mut().set(key, value.clone());
                    value
                }
                [MalType::Symbol(sym), MalType::List(bindings), exp] if sym == "let*" => {
                    env = &Rc::new(RefCell::new(Env::new(Some(env.clone()))));
                    env::bind_list(env, bindings);
                }
                [MalType::Symbol(sym), ..] if sym == "do" => {
                    let mut ret = MalType::Nil;
                    for exp in list.iter().skip(1) {
                        ret = eval(exp, env);
                    }
                    ret
                }
                [MalType::Symbol(sym), ..] if sym == "fn*" => MalType::Func {
                    eval,
                    args: Box::new(list[1].clone()),
                    body: Box::new(list[2].clone()),
                    env: env.clone(),
                },
                [MalType::Symbol(sym), cond, success, failure] if sym == "if" => {
                    match eval(cond, env) {
                        MalType::Nil | MalType::Bool(false) => eval(failure, env),
                        _ => eval(success, env),
                    }
                }
                _ => {
                    if let MalType::List(list) = eval_ast(ast, env) {
                        let mut values = list.into_iter();
                        let func = values.next().unwrap();
                        func.eval_func(MalType::List(values.collect()))
                    } else {
                        unreachable!();
                    }
                }
            },

            _ => eval_ast(ast, env),
        }
    }
}

fn print(ast: MalType) {
    println!("{}", ast);
}

pub fn run_tty() {
    // Greet message
    println!(
        "     .~~~~`\\~~\\
     ;       ~~ \\
     |           ;
 ,--------,______|---.
/          \\-----`    \\
`.__________`-_______-'
           {}\n",
        1 as char
    );

    println!("Howdy, welcome to RustOS");
    // Initialize environment
    let env = Rc::new(RefCell::new(Env::new(None)));
    core_env::init_core_env(&env);

    // REPL
    loop {
        let input = read();
        let ast = eval(&input, &env);
        print(ast);
    }
}
