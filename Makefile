NAME = OS


SRC_DIR := src
BUILD_DIR := build

.PHONY: boot clean

boot:
	mkdir -p $(BUILD_DIR)
	
	
	nasm -o $(BUILD_DIR)/bootLoader.bin $(SRC_DIR)/bootLoader.asm
	
	gcc -ffreestanding -m32 -c -o $(BUILD_DIR)/kernel.bin $(SRC_DIR)/kernel.c
	
	dd if=/dev/zero of=$(BUILD_DIR)/OS.flp bs=512 count=2880
	
	dd if=$(BUILD_DIR)/bootLoader.bin of=$(BUILD_DIR)/OS.flp conv=notrunc
	dd if=$(BUILD_DIR)/kernel.bin of=$(BUILD_DIR)/OS.flp conv=notrunc bs=512 seek=1

clean:
	rm -rf $(BUILD_DIR)
