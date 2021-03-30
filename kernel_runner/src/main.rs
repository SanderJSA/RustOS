use std::fs::File;
use std::io::{prelude::*, Seek, SeekFrom, Write};
use std::ops::Add;
use std::path::{Path, PathBuf};
use std::process::{exit, Command};
use std::{env, process};

const GDB: bool = false;
const SECTOR_SIZE: usize = 512;
const FS_SPACE: &[u8] = &[0; 100 * 512];
const QEMU_SUCCESS: i32 = 33;

fn main() {
    let kernel = env::args().nth(1).expect("Not enough arguments");
    let config = BuildConfig::new(kernel);

    config.create_kernel_bin(&objcopy_path());
    config.create_image();

    let status = config.run_qemu();
    if status != QEMU_SUCCESS {
        exit(status);
    }
}

struct BuildConfig {
    kernel: String,
    kernel_bin: String,
    image: String,
    is_test: bool,
}

impl BuildConfig {
    fn new(kernel: String) -> BuildConfig {
        let kernel_bin = kernel.clone().add(".bin");
        let image = kernel.clone().add(".img");
        let is_test = Path::new(&kernel).parent().unwrap().ends_with("deps");

        BuildConfig {
            kernel,
            kernel_bin,
            image,
            is_test,
        }
    }

    fn create_kernel_bin(&self, objcopy: &PathBuf) {
        let mut cmd = Command::new(objcopy);
        cmd.arg("-O")
            .arg("binary")
            .arg("--binary-architecture=i386:x86-64")
            .arg(&self.kernel)
            .arg(&self.kernel_bin);

        if !cmd.status().expect("Could not run objcopy").success() {
            eprintln!("Error while running objcopy");
            process::exit(1);
        }
    }

    fn create_image(&self) {
        let mut image_file = File::create(&self.image).expect("Could not create image file");

        let mut kernel_file = File::open(&self.kernel_bin).unwrap();

        let mut bootloader = [0; 2 * SECTOR_SIZE];
        let mut kernel = Vec::new();
        kernel_file
            .read_exact(&mut bootloader)
            .expect("Could not load bootloader");

        kernel_file
            .seek(SeekFrom::Start(0x100000 - 0x7C00))
            .expect("Could not find kernel");
        kernel_file
            .read_to_end(&mut kernel)
            .expect("Could not load kernel");

        image_file.write_all(&bootloader).unwrap();
        image_file.write_all(&kernel).unwrap();
        pad_to_sector(&mut image_file);

        // Add space for files to be written to
        image_file
            .write_all(FS_SPACE)
            .expect("Could not add space for FS");
    }

    fn run_qemu(&self) -> i32 {
        let mut cmd = Command::new("qemu-system-x86_64");
        cmd.arg("-drive")
            .arg(format!("file={},format=raw", &self.image))
            .arg("-boot")
            .arg("c")
            .arg("-device")
            .arg("isa-debug-exit,iobase=0xf4,iosize=0x04")
            .arg("-serial")
            .arg("stdio");
        if self.is_test {
            cmd.arg("-display").arg("none");
        }
        if GDB {
            cmd.arg("-s").arg("-S");
        }

        cmd.status()
            .expect("Could not run qemu-system-x86_64, is it installed ?")
            .code()
            .unwrap_or(1)
    }
}

fn objcopy_path() -> PathBuf {
    let llvm_tools = match llvm_tools::LlvmTools::new() {
        Ok(tools) => tools,
        Err(err) => {
            eprintln!("llvm-tools: {:?}", err);
            eprintln!("llvm-tools-preview might be missing");
            eprintln!("install it using `rustup component add llvm-tools-preview`");
            process::exit(1);
        }
    };

    let objcopy = llvm_tools
        .tool(&llvm_tools::exe("llvm-objcopy"))
        .expect("llvm-objcopy not found in llvm-tools");
    objcopy
}

fn pad_to_sector(target: &mut File) {
    let bytes_written = target.seek(SeekFrom::Current(0)).unwrap() as usize;
    let bytes_to_pad = SECTOR_SIZE - (bytes_written % SECTOR_SIZE);
    target
        .write_all(vec![0; bytes_to_pad].as_slice())
        .expect("Could not pad remaining bytes");
}
