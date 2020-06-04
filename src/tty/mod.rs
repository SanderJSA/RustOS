use crate::println;

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
    loop   {}
}
