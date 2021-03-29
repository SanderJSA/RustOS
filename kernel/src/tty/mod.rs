//! This module implements a tty

use crate::driver::ps2_keyboard::readline;
use crate::{exit_qemu, file_system, print, println, QemuExitCode};

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
