NAME = RustOS

BUILD_DIR := build

IMAGE := $(BUILD_DIR)/$(NAME).img
KERNEL = target/i686-RustOS/release/librust_kernel.a

SRC = $(wildcard src/*.rs)

# GCC := /usr/local/cross/bin/i386-elf-gcc
LD := /usr/local/cross/bin/i386-elf-ld
# GDB := /usr/local/cross/bin/i386-elf-gdb

.PHONY: run clean
run: $(IMAGE)
	qemu-system-i386 -fda $(IMAGE) -boot a

# Create image with bootloader on first sector and kernel on the first sector onwards
$(IMAGE): $(BUILD_DIR)/boot_loader.bin $(BUILD_DIR)/kernel.bin
	dd if=$< of=$(IMAGE)
	dd if=$(BUILD_DIR)/kernel.bin of=$(IMAGE) conv=notrunc bs=512 seek=1

# Link set entry point first then link the kernel
$(BUILD_DIR)/kernel.bin: $(BUILD_DIR)/kernel_start.o $(KERNEL)
	$(LD) -o $@ -T linker.ld $^

# Compile rust kernel
$(KERNEL): $(SRC)
	cargo xbuild --lib --release --target i686-RustOS.json

#.o from .asm
$(BUILD_DIR)/%.o: src/boot/%.asm
	mkdir -p $(BUILD_DIR)
	nasm -f elf -o $@ $<

#.bin from .asm
$(BUILD_DIR)/%.bin: src/boot/%.asm
	mkdir -p $(BUILD_DIR)
	nasm -f bin -o $@ $<

debug: $(IMAGE) $(BUILD_DIR)/kernel.elf
	qemu-system-i386 -S -s -fda $(IMAGE) &
	$(GDB) -ex "target remote localhost:1234" -ex "file $(BUILD_DIR)/kernel.elf"

clean:
	cargo clean
	${RM} $(BUILD_DIR)/*
