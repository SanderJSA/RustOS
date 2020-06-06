use crate::{println, print};
use driver::ps2_keyboard::readline;

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

        match input {
            "help\n" => println!("RustOS tty v1.0\nNo other commands are supported for now."),
            _ => println!("Unknown command: {}", input),
        }
    }
}
