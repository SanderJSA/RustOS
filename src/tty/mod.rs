//! This module implements a tty

use crate::{println, print};
use driver::ps2_keyboard::readline;
use ::{fs, exit_qemu};
use QemuExitCode;

/// Start and run tty
pub fn run_tty() {
    // Set up shell
    println!("     .~~~~`\\~~\\
     ;       ~~ \\
     |           ;
 ,--------,______|---.
/          \\-----`    \\
`.__________`-_______-'
           {}\n", 1 as char);
    println!("Howdy, welcome to RustOS");

    // Run shell
    loop {
        print!("> ");
        let input = readline();

        match input.split_whitespace().nth(0).unwrap() {
            "help" => println!("RustOS tty v1.0\nNo other commands are supported for now."),
            "shutdown" => exit_qemu(QemuExitCode::Success),
            "ls" => fs::ls(),
            "touch" => {
                let data: [u8; 0] = [];
                fs::add_file(input.split_whitespace().nth(1).unwrap(), &data, 0)},
            _ => print!("Unknown command: {}", input),
        }
    }
}
