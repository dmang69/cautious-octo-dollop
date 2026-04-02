CC = gcc
CFLAGS = -Wall -Wextra -pedantic -std=c11 -O2
DEBUG_FLAGS = -g -DDEBUG
INCLUDES = -Isrc

# Main targets
all: test_harness

# Debug build
debug: CFLAGS += $(DEBUG_FLAGS)
debug: test_harness

# Build the reference implementation
capability_core.o: src/reference/capability_core_modified.c src/reference/capability_core.h
	$(CC) $(CFLAGS) $(INCLUDES) -c -o capability_core.o src/reference/capability_core_modified.c

# Build the test harness
test_harness: src/test_harness.c capability_core.o
	$(CC) $(CFLAGS) $(INCLUDES) -o test_harness src/test_harness.c capability_core.o -lrt

# Clean build artifacts
clean:
	rm -f test_harness *.o

.PHONY: all debug clean