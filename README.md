![](https://github.com/SanderJSA/RustOS/workflows/Build/badge.svg)
# RustOS

A pure Rust and inline assembly x86_64 kernel with a custom bootloader with as few dependencies as possible.

## Features

- A custom two stage bootloader that loads the kernel, enters protected mode, sets up paging and then enters long mode
- Interrupts
- Page allocation
- VGA driver
- PS2 Keyboard driver
- ATA driver
- Support for unit and integration tests executed on the target system

## Requirements

Can be compiled on any Operating System with the following dependencies:
- Qemu
- Rust nightly ```rustup default nightly```
- Llvm-tools-preview, rust-src ```rustup component add llvm-tools-preview rust-src```

## Getting Started

This might be one of the easiest OS to get up and running
```
git clone https://github.com/SanderJSA/RustOS.git
cd RustOS
cargo build -p kernel_runner --release
```

`cargo xrun` Compiles and runs the OS in release mode on qemu  
`cargo xdebug` Compiles and runs the OS in debug mode on qemu  
`cargo xtest` Runs unit and integration tests  

## Resources

This project wouldn't have been possible without these ressources:
- [OSDev.org](wiki.osdev.org)
- [Philipp Oppermann's blog](os.phil-opp.com)
- [Bare Metal Rust](randomhacks.net/bare-metal-rust)
