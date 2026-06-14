# Target OS Compiler Setup
CC = x86_64-elf-gcc
AS = nasm
CFLAGS = -ffreestanding -mno-red-zone -Wall -Wextra -pedantic -std=c11 -O2
ASFLAGS = -f elf64

# Host Compiler Setup (for testing)
HOST_CC = gcc
HOST_CFLAGS = -Wall -Wextra -pedantic -std=c11 -O2 -D_POSIX_C_SOURCE=200809L

DEBUG_FLAGS = -g -DDEBUG
INCLUDES = -Isrc

# Main targets
all: test_harness intentkernel_suite securecurl

KERNEL_OBJS = src/arch/x86_64/boot/boot.o src/kernel/console/console.o src/kernel/init/main.o

src/arch/x86_64/boot/boot.o: src/arch/x86_64/boot/boot.asm
	$(AS) $(ASFLAGS) -o $@ $<

src/kernel/init/main.o: src/kernel/init/main.c
	$(CC) $(CFLAGS) $(INCLUDES) -c -o $@ $<

src/kernel/console/console.o: src/kernel/console/console.c
	$(CC) $(CFLAGS) $(INCLUDES) -c -o $@ $<

kernel: $(KERNEL_OBJS)
	$(CC) -T src/arch/x86_64/linker.ld -o IntentKernel.bin -ffreestanding -O2 -nostdlib $(KERNEL_OBJS) -lgcc
	@echo "Kernel built successfully as IntentKernel.bin"

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

# ------------------------------------------------------------------ #
# IntentKernel Suite — intentd + capd + ip-descramblerd + libik       #
# ------------------------------------------------------------------ #

IK_SRCS = src/intentd/sha256.c \
           src/intentd/intentd.c \
           src/capd/capd.c \
           src/ip_descramblerd/ip_descramblerd.c \
           src/libintentkernel/libintentkernel.c

IK_OBJS = $(IK_SRCS:.c=.o)

src/intentd/sha256.o: src/intentd/sha256.c src/intentd/sha256.h
	$(HOST_CC) $(HOST_CFLAGS) $(INCLUDES) -c -o $@ $<

src/intentd/intentd.o: src/intentd/intentd.c src/intentd/intentd.h src/intentd/token.h src/intentd/sha256.h
	$(HOST_CC) $(HOST_CFLAGS) $(INCLUDES) -c -o $@ $<

src/capd/capd.o: src/capd/capd.c src/capd/capd.h src/intentd/token.h src/intentd/sha256.h
	$(HOST_CC) $(HOST_CFLAGS) $(INCLUDES) -c -o $@ $<

src/ip_descramblerd/ip_descramblerd.o: src/ip_descramblerd/ip_descramblerd.c src/ip_descramblerd/ip_descramblerd.h
	$(HOST_CC) $(HOST_CFLAGS) $(INCLUDES) -c -o $@ $<

src/libintentkernel/libintentkernel.o: src/libintentkernel/libintentkernel.c src/libintentkernel/libintentkernel.h
	$(HOST_CC) $(HOST_CFLAGS) $(INCLUDES) -c -o $@ $<

# Static library
intentkernel_suite: $(IK_OBJS)
	ar rcs libintentkernel.a $(IK_OBJS)
	@echo "libintentkernel.a built successfully"

# securecurl demo
securecurl: src/securecurl/securecurl.c intentkernel_suite
	$(HOST_CC) $(HOST_CFLAGS) $(INCLUDES) -o securecurl \
	    src/securecurl/securecurl.c libintentkernel.a -lrt
	@echo "securecurl built successfully"

# Run test harness (reference capability system)
test: test_harness
	./test_harness

# Run end-to-end securecurl demo
demo: securecurl
	./securecurl https://example.com

# Emulation
run: kernel
	qemu-system-x86_64 -m 512M -kernel IntentKernel.bin

# Clean build artifacts
clean:
	rm -f test_harness securecurl libintentkernel.a *.o \
	    src/intentd/*.o src/capd/*.o src/ip_descramblerd/*.o \
	    src/libintentkernel/*.o src/securecurl/*.o \
	    *.elf *.bin *.iso \
	    src/arch/x86_64/boot/*.o src/kernel/init/*.o src/kernel/console/*.o

.PHONY: all debug clean kernel run test demo intentkernel_suite