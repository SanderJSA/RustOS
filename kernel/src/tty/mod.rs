//! This module implements a tty

mod core_env;
mod env;
mod reader;
mod types;

use crate::driver::ps2_keyboard::readline;
use crate::{print, println};
use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::string::{String, ToString};
use core::cell::RefCell;
use env::{Env, RcEnv};
use reader::Reader;
use types::*;

fn greet_msg() {
    println!(
        "     .~~~~`\\~~\\
     ;       ~~ \\
     |           ;
 ,--------,______|---.
/          \\-----`    \\
`.__________`-_______-'
           {}
Howdy, welcome to RustOS",
        1 as char
    );
}

fn quasiquote(ast: MalType) -> MalType {
    match ast {
        MalType::List(list) => match list.as_slice() {
            [MalType::Symbol(sym), arg] if sym == "unquote" => arg.clone(),
            _ => {
                let mut res = alloc::vec![];
                for elt in list.into_iter().rev() {
                    if let MalType::List(ref el_list) = elt {
                        if el_list.get(0) == Some(&MalType::splice_unquote()) {
                            res = alloc::vec![
                                MalType::Symbol("conj".to_string()),
                                el_list[1].clone(),
                                MalType::List(res)
                            ];
                            continue;
                        }
                    }

                    res = alloc::vec![
                        MalType::Symbol("cons".to_string()),
                        quasiquote(elt),
                        MalType::List(res)
                    ];
                }
                MalType::List(res)
            }
        },
        ast => MalType::List(alloc::vec![MalType::quote(), ast]),
    }
}

fn get_macro(ast: &MalType, env: &RcEnv) -> Option<MalType> {
    if let MalType::List(ref list) = ast {
        if let Some(MalType::Symbol(sym)) = list.get(0) {
            let env_sym = env.borrow().get(sym);
            if let Some(MalType::Func { is_macro: true, .. }) = env_sym {
                return env_sym;
            }
        }
    }
    None
}

fn eval_ast(ast: MalType, env: RcEnv) -> MalType {
    match ast {
        MalType::Symbol(sym) => env
            .borrow()
            .get(&sym)
            .unwrap_or_else(|| panic!("{} not found in env: {}", sym, env.borrow())),
        MalType::List(list) => {
            MalType::List(list.into_iter().map(|val| eval(val, env.clone())).collect())
        }
        _ => ast,
    }
}

fn macroexpand(mut ast: MalType, env: &RcEnv) -> MalType {
    while let Some(MalType::Func {
        args,
        body,
        env: outer,
        ..
    }) = get_macro(&ast, env)
    {
        if let MalType::List(ref list) = ast {
            let mut env = Env::new(Some(outer.clone()));
            env.bind(&args, &list[1..]);
            ast = eval(*body, Rc::new(RefCell::new(env)));
        } else {
            unreachable!()
        }
    }
    ast
}

fn eval(mut ast: MalType, mut env: RcEnv) -> MalType {
    loop {
        ast = macroexpand(ast, &env);
        if let MalType::List(_) = ast {
        } else {
            ast = eval_ast(ast, env.clone());
        }
        match ast {
            MalType::List(ref list) => match list.as_slice() {
                [] => return eval_ast(ast, env),
                [MalType::Symbol(sym), arg] if sym == "quote" => {
                    return arg.clone();
                }
                [MalType::Symbol(sym), MalType::Symbol(key), value] if sym == "def!" => {
                    let value = eval(value.clone(), env.clone());
                    env.borrow_mut().set(key, value.clone());
                    return value;
                }
                [MalType::Symbol(sym), MalType::List(bindings), exp] if sym == "let*" => {
                    env = Rc::new(RefCell::new(Env::new(Some(env))));
                    env::bind_list(env.clone(), bindings);
                    ast = exp.clone();
                }
                [MalType::Symbol(sym)] if sym == "do" => {
                    return MalType::Nil;
                }
                [MalType::Symbol(sym), middle @ .., tail] if sym == "do" => {
                    for exp in middle {
                        eval(exp.clone(), env.clone());
                    }
                    ast = tail.clone();
                }
                [MalType::Symbol(sym), MalType::Symbol(key), value] if sym == "defmacro!" => {
                    let value = eval(value.clone(), env.clone());
                    if let MalType::Func {
                        args,
                        body,
                        env: fun,
                        ..
                    } = value
                    {
                        let value = MalType::Func {
                            args,
                            body,
                            env: fun,
                            is_macro: true,
                        };
                        env.borrow_mut().set(key, value.clone());
                        return value;
                    }
                }
                [MalType::Symbol(sym), args, body] if sym == "fn*" => {
                    return MalType::Func {
                        args: Box::new(args.clone()),
                        body: Box::new(body.clone()),
                        env,
                        is_macro: false,
                    };
                }
                [MalType::Symbol(sym), cond, success, failure] if sym == "if" => {
                    ast = match eval(cond.clone(), env.clone()) {
                        MalType::Nil | MalType::Bool(false) => failure.clone(),
                        _ => success.clone(),
                    }
                }
                [MalType::Symbol(sym), arg] if sym == "quasiquote" => {
                    ast = quasiquote(arg.clone());
                }
                [MalType::Symbol(sym), arg] if sym == "macroexpand" => {
                    return macroexpand(arg.clone(), &env);
                }
                _ => {
                    if let MalType::List(list) = eval_ast(ast, env) {
                        match list.as_slice() {
                            [MalType::Builtin {
                                eval,
                                args,
                                env: outer,
                            }, tail @ ..] => {
                                env = Rc::new(RefCell::new(Env::new(Some(outer.clone()))));
                                env.borrow_mut().bind(args, tail);
                                return eval(&env);
                            }
                            [MalType::Func {
                                args,
                                body,
                                env: outer,
                                is_macro,
                            }, tail @ ..] => {
                                ast = *body.clone();
                                env = Rc::new(RefCell::new(Env::new(Some(outer.clone()))));
                                env.borrow_mut().bind(args, tail);
                            }
                            list => panic!("Invalid function: {}", list[0]),
                        }
                    } else {
                        unreachable!();
                    }
                }
            },

            _ => return eval_ast(ast, env),
        }
    }
}

fn read_str(line: &str) -> MalType {
    Reader::new(line).read_form()
}

fn pr_str(ast: &MalType, print_readably: bool) -> String {
    match ast {
        MalType::Number(num) => num.to_string(),
        MalType::Symbol(sym) => sym.to_string(),
        MalType::List(list) => {
            alloc::format!(
                "({})",
                list.iter()
                    .map(|ast| pr_str(ast, print_readably))
                    .intersperse(' '.to_string())
                    .collect::<String>()
            )
        }
        MalType::Func { .. } => "#<function>".to_string(),
        MalType::Builtin { .. } => "#<builtin>".to_string(),
        MalType::Nil => "nil".to_string(),
        MalType::Bool(true) => "true".to_string(),
        MalType::Bool(false) => "false".to_string(),
        MalType::String(string) => {
            if print_readably {
                alloc::format!("\"{}\"", string)
            } else {
                //TODO: Escape special chars
                string.clone()
            }
        }
        MalType::File(_) => "#<File>".to_string(),
    }
}

fn print(ast: &MalType) {
    println!("{}", pr_str(ast, true));
}

fn rep(input: &str, env: RcEnv) -> String {
    let ast_in = read_str(input);
    let ast_out = eval(ast_in, env);
    pr_str(&ast_out, true)
}

pub fn run_tty() {
    greet_msg();

    let env = Rc::new(RefCell::new(Env::new(None)));
    core_env::init_core_env(&env);

    loop {
        print!("root> ");
        let input = readline().trim_end();
        let output = rep(input, env.clone());
        println!("{}", output);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn simple_sum() {
        let env = Rc::new(RefCell::new(Env::new(None)));
        core_env::init_core_env(&env);
        let input = "(+ 503 2130)";

        let tokens = Reader::new(input).read_form();
        let ast = eval(tokens, env);

        match ast {
            MalType::Number(2633) => {}
            _ => panic!(),
        }
    }

    #[test_case]
    fn simple_fn() {
        let env = Rc::new(RefCell::new(Env::new(None)));
        core_env::init_core_env(&env);
        let input = "((fn* (a b) (+ a b)) 10 20)";

        let tokens = Reader::new(input).read_form();
        let ast = eval(tokens, env);

        match ast {
            MalType::Number(30) => {}
            _ => panic!(),
        }
    }

    #[test_case]
    fn simple_def() {
        let env = Rc::new(RefCell::new(Env::new(None)));
        core_env::init_core_env(&env);
        let input = &["(def! a 10)", "a"];

        let tokens = Reader::new(input[0]).read_form();
        eval(tokens, env.clone());
        let tokens = Reader::new(input[1]).read_form();
        let ast = eval(tokens, env);

        match ast {
            MalType::Number(10) => {}
            _ => panic!(),
        }
    }

    #[test_case]
    fn recursive_sum() {
        let env = Rc::new(RefCell::new(Env::new(None)));
        core_env::init_core_env(&env);
        let input = &[
            "(def! sum (fn* (n) (if (> n 0) (+ n (sum (- n 1))) 0)))",
            "(sum 10)",
        ];

        let tokens = Reader::new(input[0]).read_form();
        eval(tokens, env.clone());
        let tokens = Reader::new(input[1]).read_form();
        let ast = eval(tokens, env);

        match ast {
            MalType::Number(55) => {}
            _ => panic!(),
        }
    }
}
