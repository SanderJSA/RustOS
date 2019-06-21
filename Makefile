NAME = OS

SRC_DIR := src
BUILD_DIR := build

IMAGE := $(BUILD_DIR)/OS.img 

GCC := /usr/local/cross/bin/i386-elf-gcc
LD := /usr/local/cross/bin/i386-elf-ld
GDB := /usr/local/cross/bin/i386-elf-gdb

.PHONY: base bootLoader kernel run clean

all: | base bootLoader kernel


base:

	#Creating build folder
	mkdir -p $(BUILD_DIR)

	#Preparing image file
	dd if=/dev/zero of=$(IMAGE) bs=512 count=2880


bootLoader: $(SRC_DIR)/bootLoader.asm
	
	#Compiling bootLoader
	nasm -fbin -o $(BUILD_DIR)/bootLoader.bin $(SRC_DIR)/bootLoader.asm

	#inserting bootLoader into first sector
	dd if=$(BUILD_DIR)/bootLoader.bin of=$(IMAGE) conv=notrunc


kernel: $(SRC_DIR)/kernel.c

	#Compiling C code
	$(GCC) -ffreestanding -c -Wall -Wextra -o $(BUILD_DIR)/kernel.o $(SRC_DIR)/kernel.c
	
	nasm -f elf -o $(BUILD_DIR)/kernel_start.o $(SRC_DIR)/kernel_start.asm
	$(LD) -o $(BUILD_DIR)/kernel.bin -Ttext 0x1000 --oformat binary $(BUILD_DIR)/kernel.o $(BUILD_DIR)/kernel_start.o

	#Inserting kernel in second sector
	dd if=$(BUILD_DIR)/kernel.bin of=$(IMAGE) conv=notrunc bs=512 seek=1


run: all
	qemu-system-i386 -fda $(IMAGE) -boot a


kernel.elf: $(SRC_DIR)/kernel.c

	#Compiling C code
	$(GCC) -ffreestanding -c -Wall -Wextra -O0 -g -o $(BUILD_DIR)/kernel.o $(SRC_DIR)/kernel.c
	
	nasm -f elf -o $(BUILD_DIR)/kernel_start.o $(SRC_DIR)/kernel_start.asm
	$(LD) -o $(BUILD_DIR)/kernel.elf -Ttext 0x0 $(BUILD_DIR)/kernel.o $(BUILD_DIR)/kernel_start.o

debug: all kernel.elf
	qemu-system-i386 -s -S -fda $(IMAGE) -boot a & $(GDB) -ex "target remote localhost:1234" -ex "symbol-file $(BUILD_DIR)/kernel.elf"

clean:
	rm -rf $(BUILD_DIR)
