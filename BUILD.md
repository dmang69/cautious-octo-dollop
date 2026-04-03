# IntentKernel Build Instructions

This repository contains the IntentKernel architecture specification and reference implementation.

## Building the Reference Implementation

To build the reference implementation and test harness:

```bash
make
```

This will compile the capability_core.c file and the test harness.

## Running the Test Harness

After building, you can run the test harness to see the capability system in action:

```bash
./test_harness
```

This will demonstrate:
1. Creating capabilities with different types, TTLs, and use counts
2. Validating capabilities
3. Observing how single-use capabilities are automatically revoked
4. Manually revoking capabilities

## Debugging

To build with debug symbols:

```bash
make debug
```

## Cleaning Up

To clean build artifacts:

```bash
make clean
```

## Project Structure

- `src/reference/capability_core.c` - Original reference implementation
- `src/reference/capability_core_modified.c` - Modified implementation for testing
- `src/reference/capability_core.h` - Header file with capability definitions
- `src/test_harness.c` - Test program demonstrating the capability system

## System Requirements

- C compiler (gcc recommended)
- POSIX-compliant system (Linux, macOS, or Windows with MinGW/Cygwin)
- make utility
