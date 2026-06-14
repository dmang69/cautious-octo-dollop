#!/bin/bash
# scripts/build-all.sh
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}================================${NC}"
echo -e "${GREEN}AI OS Cross-Platform Build${NC}"
echo -e "${GREEN}================================${NC}\n"

# Detect OS
OS="unknown"
case "$(uname -s)" in
    Linux*)     OS="linux";;
    Darwin*)    OS="macos";;
    MINGW*|MSYS*|CYGWIN*)    OS="windows";;
esac

echo -e "${YELLOW}Detected OS: $OS${NC}\n"

# Check prerequisites
echo -e "${YELLOW}Checking prerequisites...${NC}"

if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: Rust/Cargo not found. Install from https://rustup.rs/${NC}"
    exit 1
fi

if ! command -v node &> /dev/null; then
    echo -e "${RED}Error: Node.js not found. Install from https://nodejs.org/${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Rust $(cargo --version)${NC}"
echo -e "${GREEN}✓ Node $(node --version)${NC}\n"

# Build core components
echo -e "${YELLOW}Building core AI runtime...${NC}"
cd core/ai-runtime || { echo -e "${RED}Error: Directory core/ai-runtime not found${NC}"; exit 1; }
cargo build --release
echo -e "${GREEN}✓ AI runtime built${NC}\n"

cd ../..

# Build kernel interface
echo -e "${YELLOW}Building kernel interface...${NC}"
cd core/kernel-interface || { echo -e "${RED}Error: Directory core/kernel-interface not found${NC}"; exit 1; }
cargo build --release
echo -e "${GREEN}✓ Kernel interface built${NC}\n"

cd ../..

# Build gRPC services
echo -e "${YELLOW}Building gRPC services...${NC}"
cd core/ipc || { echo -e "${RED}Error: Directory core/ipc not found${NC}"; exit 1; }
cargo build --release
echo -e "${GREEN}✓ gRPC services built${NC}\n"

cd ../..

# Build Tauri shell
echo -e "${YELLOW}Building Tauri shell...${NC}"
cd shell/tauri-app || { echo -e "${RED}Error: Directory shell/tauri-app not found${NC}"; exit 1; }

if [ ! -d "node_modules" ]; then
    echo -e "${YELLOW}Installing npm dependencies...${NC}"
    npm install
fi

npm run tauri build
echo -e "${GREEN}✓ Tauri shell built${NC}\n"

cd ../..

# Run tests
echo -e "${YELLOW}Running tests...${NC}"
cargo test --workspace --release
echo -e "${GREEN}✓ All tests passed${NC}\n"

# Package artifacts
echo -e "${YELLOW}Packaging artifacts...${NC}"
mkdir -p dist

case "$OS" in
    linux)
        cp target/release/ai-runtime dist/
        cp target/release/libkernel_interface.so dist/
        cp shell/tauri-app/src-tauri/target/release/bundle/appimage/*.AppImage dist/ 2>/dev/null || true
        cp shell/tauri-app/src-tauri/target/release/bundle/deb/*.deb dist/ 2>/dev/null || true
        ;;
    macos)
        cp target/release/ai-runtime dist/
        cp target/release/libkernel_interface.dylib dist/
        cp -r shell/tauri-app/src-tauri/target/release/bundle/macos/*.app dist/ 2>/dev/null || true
        cp shell/tauri-app/src-tauri/target/release/bundle/dmg/*.dmg dist/ 2>/dev/null || true
        ;;
    windows)
        cp target/release/ai-runtime.exe dist/
        cp target/release/kernel_interface.dll dist/
        cp shell/tauri-app/src-tauri/target/release/bundle/msi/*.msi dist/ 2>/dev/null || true
        ;;
esac

echo -e "${GREEN}✓ Artifacts packaged in dist/${NC}\n"

# Print summary
echo -e "${GREEN}================================${NC}"
echo -e "${GREEN}Build Complete!${NC}"
echo -e "${GREEN}================================${NC}"
echo -e "\nArtifacts:"
ls -lh dist/

echo -e "\n${YELLOW}Next steps:${NC}"
echo -e "1. Run AI runtime: ${GREEN}./dist/ai-runtime${NC}"
echo -e "2. Launch shell: ${GREEN}Open dist/ai-os-shell${NC}"
echo -e "3. Collect training data: ${GREEN}python3 scripts/collect-telemetry.py${NC}"
