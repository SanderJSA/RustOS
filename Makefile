NAME = RustOS

LD := ld.lld
BUILD_DIR := build

IMAGE := $(BUILD_DIR)/$(NAME).img
IMAGE_DEBUG := $(BUILD_DIR)/$(NAME)_debug.img
IMAGE_CHECK := $(BUILD_DIR)/$(NAME)_check.img

KERNEL = target/x86_64-RustOS/release/rust_os
KERNEL_DEBUG = target/x86_64-RustOS/debug/rust_os
KERNEL_CHECK = target/x86_64-RustOS/debug/rust_os

SRC = $(shell find src -name *.rs)

.PHONY: run debug clean


#
# Release
#

run: $(IMAGE)
	qemu-system-x86_64 -drive file=$(IMAGE),format=raw -boot c

# Create image with bootloader on first sector and kernel on the first sector onwards
$(IMAGE): $(BUILD_DIR)/bootloader.bin $(BUILD_DIR)/kernel.bin
	dd if=/dev/zero of=$@ bs=512 count=256
	dd if=$< of=$@ conv=notrunc
	dd if=$(BUILD_DIR)/kernel.bin of=$@ conv=notrunc bs=512 seek=1

# Convert kernel to binary
$(BUILD_DIR)/kernel.bin:$(KERNEL)
	llvm-objcopy -O binary --binary-architecture=i386:x86-64 $< $@

# Compile rust kernel
$(KERNEL): $(SRC)
	cargo xbuild --bin rust_os --release

#
# Debug
#

# Launch qemu and attach gdb to it
debug: $(IMAGE_DEBUG)
	qemu-system-x86_64 -S -s -drive file=$(IMAGE_DEBUG),format=raw -boot c &
	gdb -ex "target remote localhost:1234" -ex "file $(KERNEL_DEBUG)"

# Create image with bootloader on first sector and kernel on the first sector onwards
$(IMAGE_DEBUG): $(BUILD_DIR)/bootloader.bin $(BUILD_DIR)/kernel_debug.bin
	dd if=/dev/zero of=$@ bs=512 count=256
	dd if=$< of=$@ conv=notrunc
	dd if=$(BUILD_DIR)/kernel_debug.bin of=$@ conv=notrunc bs=512 seek=1

# link kernel and kernel start to binary
$(BUILD_DIR)/kernel_debug.bin: $(KERNEL_DEBUG)
	llvm-objcopy -O binary --binary-architecture=i386:x86-64 $< $@

# Compile rust kernel in debug mode
$(KERNEL_DEBUG): $(SRC)
	cargo xbuild --bin rust_os

#
# Check
#

test: $(IMAGE_CHECK)
	qemu-system-x86_64 -drive file=$(IMAGE_CHECK),format=raw -device isa-debug-exit,iobase=0xf4,iosize=0x04 -boot c

# Create image with bootloader on first sector and kernel on the first sector onwards
$(IMAGE_CHECK): $(BUILD_DIR)/bootloader.bin $(BUILD_DIR)/kernel_check.bin
	dd if=/dev/zero of=$@ bs=512 count=256
	dd if=$< of=$@ conv=notrunc
	dd if=$(BUILD_DIR)/kernel_check.bin of=$@ conv=notrunc bs=512 seek=1

# Convert kernel to binary
$(BUILD_DIR)/kernel_check.bin: $(KERNEL_TEST)
	llvm-objcopy -O binary --binary-architecture=i386:x86-64 $< $@

#
# Intermediate
#

#.bin from .asm
$(BUILD_DIR)/%.bin: src/boot/%.asm
	mkdir -p $(BUILD_DIR)
	nasm -f bin -o $@ $<


clean:
	cargo clean
	${RM} $(BUILD_DIR)/*
