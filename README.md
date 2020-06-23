![](https://github.com/SanderJSA/RustOS/workflows/Build/badge.svg)
# RustOS

A minimalist x86_64 kernel written in Rust with a custom bootloader with as few dependencies as possible.

## Features

- A custom single stage bootloader that loads the kernel, enters protected mode, sets up paging and then enters long mode
- A VGA and PS2 Keyboard driver
- Support for interrupts
- Support for page allocation
- Support for tests executed on the target system

## Requirements

- Qemu
- Rust toolchain
- Llvm-tools-preview
- Cargo-xbuild
- UNIX system

## Getting Started

```
git clone https://github.com/SanderJSA/RustOS.git
cd RustOS
make run
```

`make run` Compiles and runs the OS on qemu  
`make debug` Compiles, starts the emulator and attaches gdb to it  
`cargo xtest` Runs unit and integration tests  

## Resources

This project wouldn't have been possible without these ressources:
- [OSDev.org](wiki.osdev.org)
- [Philipp Oppermann's blog](os.phil-opp.com)
- [Bare Metal Rust](randomhacks.net/bare-metal-rust)
