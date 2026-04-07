# Target OS Compiler Setup
CC = x86_64-elf-gcc
AS = nasm
CFLAGS = -ffreestanding -mno-red-zone -Wall -Wextra -pedantic -std=c11 -O2
ASFLAGS = -f elf64

# Host Compiler Setup (for testing)
HOST_CC = gcc
HOST_CFLAGS = -Wall -Wextra -pedantic -std=c11 -O2

DEBUG_FLAGS = -g -DDEBUG
INCLUDES = -Isrc

# Main targets
all: kernel test_harness

kernel:
	@echo "Placeholder: x86_64 kernel build rules will go here."

# Debug build
debug: HOST_CFLAGS += $(DEBUG_FLAGS)
debug: CFLAGS += $(DEBUG_FLAGS)
debug: test_harness

# Build the reference implementation
capability_core.o: src/reference/capability_core_modified.c src/reference/capability_core.h
	$(HOST_CC) $(HOST_CFLAGS) $(INCLUDES) -c -o capability_core.o src/reference/capability_core_modified.c

# Build the test harness
test_harness: src/test_harness.c capability_core.o
	$(HOST_CC) $(HOST_CFLAGS) $(INCLUDES) -o test_harness src/test_harness.c capability_core.o -lrt

# Emulation
run: kernel
	qemu-system-x86_64 -m 512M

# Clean build artifacts
clean:
	rm -f test_harness *.o *.elf *.bin *.iso

.PHONY: all debug clean kernel run