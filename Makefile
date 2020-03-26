NAME = RustOS

BUILD_DIR := build

IMAGE := $(BUILD_DIR)/$(NAME).img
KERNEL = target/x86_64-RustOS/release/librust_kernel.a

SRC = $(wildcard src/*.rs)

.PHONY: run clean
run: $(IMAGE)
	qemu-system-x86_64 -fda $(IMAGE) -boot a

# Create image with bootloader on first sector and kernel on the first sector onwards
$(IMAGE): $(BUILD_DIR)/boot_loader.bin $(BUILD_DIR)/kernel.bin
	dd if=$< of=$(IMAGE)
	dd if=$(BUILD_DIR)/kernel.bin of=$(IMAGE) conv=notrunc bs=512 seek=1

# Set entry point first then link with kernel
$(BUILD_DIR)/kernel.bin: $(BUILD_DIR)/kernel_start.o $(KERNEL)
	ld -o $@ -T linker.ld $^

# Compile rust kernel
$(KERNEL): $(SRC)
	cargo xbuild --release

#.o from .asm
$(BUILD_DIR)/%.o: src/boot/%.asm
	mkdir -p $(BUILD_DIR)
	nasm -f elf64 -o $@ $<

#.bin from .asm
$(BUILD_DIR)/%.bin: src/boot/%.asm
	mkdir -p $(BUILD_DIR)
	nasm -f bin -o $@ $<

debug: $(IMAGE) $(BUILD_DIR)/kernel.elf
	qemu-system-i386 -S -s -fda $(IMAGE) &
	gdb -ex "target remote localhost:1234" -ex "file $(BUILD_DIR)/kernel.elf"

clean:
	cargo clean
	${RM} $(BUILD_DIR)/*
