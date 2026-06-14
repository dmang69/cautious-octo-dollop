#!/bin/bash
# QEMU Test Infrastructure for IntentKernel
# Builds the kernel and test harness, runs capability tests, then boots in QEMU

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}================================${NC}"
echo -e "${BLUE}QEMU Test Infrastructure${NC}"
echo -e "${BLUE}IntentKernel Capability OS${NC}"
echo -e "${BLUE}================================${NC}\n"

# Check dependencies
echo -e "${YELLOW}Checking dependencies...${NC}"

MISSING=0

if ! command -v qemu-system-x86_64 &> /dev/null; then
    echo -e "${RED}Error: qemu-system-x86_64 not found${NC}"
    echo "Install with: sudo apt-get install qemu-system-x86"
    MISSING=1
fi

if ! command -v make &> /dev/null; then
    echo -e "${RED}Error: make not found${NC}"
    echo "Install with: sudo apt-get install build-essential"
    MISSING=1
fi

if ! command -v x86_64-elf-gcc &> /dev/null; then
    echo -e "${RED}Error: x86_64-elf-gcc not found${NC}"
    echo "Install a cross-compiler toolchain (e.g. from https://github.com/nativeos/i386-elf-toolchain)"
    MISSING=1
fi

if [ "$MISSING" -eq 1 ]; then
    exit 1
fi

echo -e "${GREEN}✓ All dependencies found${NC}\n"

# Build kernel
echo -e "${YELLOW}Building IntentKernel...${NC}"
cd "$PROJECT_ROOT"
make kernel
echo -e "${GREEN}✓ Kernel built successfully${NC}\n"

# Build and run host-side capability test harness
echo -e "${YELLOW}Building capability test harness...${NC}"
make test_harness
echo -e "${GREEN}✓ Test harness built${NC}\n"

echo -e "${BLUE}================================${NC}"
echo -e "${BLUE}Test 1: Capability System${NC}"
echo -e "${BLUE}================================${NC}"
"$PROJECT_ROOT/test_harness"
echo -e "${GREEN}✓ Capability test harness passed${NC}\n"

# Launch QEMU
echo -e "${BLUE}================================${NC}"
echo -e "${BLUE}Test 2: Kernel Boot (QEMU)${NC}"
echo -e "${BLUE}================================${NC}"
echo -e "${YELLOW}Booting IntentKernel.bin in QEMU...${NC}"

PIDFILE="/tmp/intentkernel-qemu.pid"

qemu-system-x86_64 \
    -m 512M \
    -kernel "$PROJECT_ROOT/IntentKernel.bin" \
    -serial mon:stdio \
    -display none \
    -daemonize \
    -pidfile "$PIDFILE"

QEMU_PID=$(cat "$PIDFILE")
echo -e "${GREEN}✓ QEMU started (PID: $QEMU_PID)${NC}"

# Give the kernel a moment to boot, then check the process is still alive
sleep 2
if kill -0 "$QEMU_PID" 2>/dev/null; then
    echo -e "${GREEN}✓ Kernel is running in QEMU${NC}"
else
    echo -e "${RED}✗ QEMU process exited unexpectedly${NC}"
    rm -f "$PIDFILE"
    exit 1
fi

# Cleanup
echo -e "\n${YELLOW}Cleaning up...${NC}"
kill "$QEMU_PID" 2>/dev/null || true
rm -f "$PIDFILE"

echo -e "\n${GREEN}================================${NC}"
echo -e "${GREEN}All tests passed!${NC}"
echo -e "${GREEN}================================${NC}"
