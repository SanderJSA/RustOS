# RustOS

A minimalist x86_64 kernel written in Rust with a custom bootloader with as few dependencies as possible.

## Features

- A custom single stage bootloader that loads the kernel, enters protected mode, sets up paging and then enters long mode
- A simple thread-safe VGA driver
- Support for CPU interrupts
- Support for unit tests executed on the target system

## Requirements

- Qemu
- llvm-tools
- Rust toolchain
- A UNIX system

## Getting Started

```
git clone https://github.com/SanderJSA/RustOS.git
cd RustOS
make run
```

`make run` Compiles and runs the OS on qemu  
`make debug` Compiles, starts the emulator and attaches gdb to it  
`make check` Runs unit tests  

## Resources

This project wouldn't have been possible without these ressources:
- [OSDev.org](wiki.osdev.org)
- [Philipp Oppermann's blog](os.phil-opp.com)
- [Bare Metal Rust](randomhacks.net/bare-metal-rust)
