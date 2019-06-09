NAME = OS


SRC_DIR := src
BUILD_DIR := build

.PHONY: boot clean

boot:
	mkdir $(BUILD_DIR)
	nasm -f bin -o $(BUILD_DIR)/bootLoader.bin $(SRC_DIR)/bootLoader.asm
	dd status=noxfer conv=notrunc if=$(BUILD_DIR)/bootLoader.bin of=$(BUILD_DIR)/OS.flp

clean:
	rm -rf $(BUILD_DIR)
