#!/bin/bash
# Complete AI OS Deployment Script
# Builds, tests, and deploys all components across platforms

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BUILD_DIR="${PROJECT_ROOT}/build"
DIST_DIR="${PROJECT_ROOT}/dist"

echo -e "${BLUE}================================${NC}"
echo -e "${BLUE}AI OS Complete Deployment${NC}"
echo -e "${BLUE}================================${NC}\n"

# Detect OS
OS="unknown"
case "$(uname -s)" in
    Linux*)     OS="linux";;
    Darwin*)    OS="macos";;
    MINGW*|MSYS*|CYGWIN*)    OS="windows";;
esac

echo -e "${YELLOW}Detected OS: $OS${NC}\n"

# Step 1: Train AI Model
echo -e "${YELLOW}[1/7] Training AI scheduler model...${NC}"
cd "${PROJECT_ROOT}/models/scheduler"

if [ ! -f "scheduler_model.onnx" ]; then
    echo "Training new model..."
    if ! python3 train_advanced.py \
        --episodes 500 \
        --batch-size 128 \
        --output scheduler_model.onnx; then
        echo -e "${RED}✗ Model training failed.${NC}"
        echo -e "${YELLOW}  Ensure Python 3 and required ML dependencies are installed:${NC}"
        echo -e "    pip install -r ${PROJECT_ROOT}/models/requirements.txt"
        exit 1
    fi
    
    echo -e "${GREEN}✓ Model trained and exported${NC}\n"
else
    echo -e "${GREEN}✓ Using existing model${NC}\n"
fi

# Copy model to resources
mkdir -p "${PROJECT_ROOT}/models/pretrained"
cp scheduler_model.onnx "${PROJECT_ROOT}/models/pretrained/"

# Step 2: Build Core Runtime
echo -e "${YELLOW}[2/7] Building AI runtime daemon...${NC}"
cd "${PROJECT_ROOT}/core/ai-runtime"
cargo build --release
echo -e "${GREEN}✓ AI runtime built${NC}\n"

# Step 3: Build Kernel Interface
echo -e "${YELLOW}[3/7] Building kernel interface...${NC}"
cd "${PROJECT_ROOT}/core/kernel-interface"
cargo build --release
echo -e "${GREEN}✓ Kernel interface built${NC}\n"

# Step 4: Build gRPC Services
echo -e "${YELLOW}[4/7] Building gRPC services...${NC}"
cd "${PROJECT_ROOT}/core/ipc"
cargo build --release
echo -e "${GREEN}✓ gRPC services built${NC}\n"

# Step 5: Build Tauri Shell
echo -e "${YELLOW}[5/7] Building Tauri shell...${NC}"
cd "${PROJECT_ROOT}/shell/tauri-app"

if [ ! -d "node_modules" ]; then
    echo "Installing npm dependencies..."
    npm install
fi

npm run tauri build
echo -e "${GREEN}✓ Tauri shell built${NC}\n"

# Step 6: Run Tests
echo -e "${YELLOW}[6/7] Running tests...${NC}"
cd "${PROJECT_ROOT}"
cargo test --workspace --release
echo -e "${GREEN}✓ All tests passed${NC}\n"

# Step 7: Package Distribution
echo -e "${YELLOW}[7/7] Packaging distribution...${NC}"
mkdir -p "${DIST_DIR}"

case "$OS" in
    linux)
        # Copy binaries
        cp "${PROJECT_ROOT}/target/release/ai-runtime" "${DIST_DIR}/"
        cp "${PROJECT_ROOT}/target/release/libkernel_interface.so" "${DIST_DIR}/"
        
        # Copy Tauri bundle
        if [ -d "${PROJECT_ROOT}/shell/tauri-app/src-tauri/target/release/bundle/appimage" ]; then
            cp "${PROJECT_ROOT}"/shell/tauri-app/src-tauri/target/release/bundle/appimage/*.AppImage "${DIST_DIR}/" 2>/dev/null || true
        fi
        
        if [ -d "${PROJECT_ROOT}/shell/tauri-app/src-tauri/target/release/bundle/deb" ]; then
            cp "${PROJECT_ROOT}"/shell/tauri-app/src-tauri/target/release/bundle/deb/*.deb "${DIST_DIR}/" 2>/dev/null || true
        fi
        
        # Copy systemd service file into dist so install.sh can reference it directly
        mkdir -p "${DIST_DIR}/systemd"
        cp "${PROJECT_ROOT}/platform/linux/systemd/ai-context-manager.service" "${DIST_DIR}/systemd/" 2>/dev/null || true

        # Create installation script
        cat > "${DIST_DIR}/install.sh" << 'EOF'
#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "Installing AI OS..."

# Install runtime
sudo cp "${SCRIPT_DIR}/ai-runtime" /usr/local/bin/
sudo cp "${SCRIPT_DIR}/libkernel_interface.so" /usr/local/lib/

# Install systemd service
sudo cp "${SCRIPT_DIR}/systemd/ai-context-manager.service" /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable ai-context-manager

# Install shell (if DEB package exists)
if ls "${SCRIPT_DIR}"/*.deb 1> /dev/null 2>&1; then
    sudo dpkg -i "${SCRIPT_DIR}"/*.deb
fi

echo "Installation complete!"
echo "Start the service with: sudo systemctl start ai-context-manager"
EOF
        chmod +x "${DIST_DIR}/install.sh"
        ;;
    
    macos)
        cp "${PROJECT_ROOT}/target/release/ai-runtime" "${DIST_DIR}/"
        cp "${PROJECT_ROOT}/target/release/libkernel_interface.dylib" "${DIST_DIR}/"
        
        if [ -d "${PROJECT_ROOT}/shell/tauri-app/src-tauri/target/release/bundle/dmg" ]; then
            cp "${PROJECT_ROOT}"/shell/tauri-app/src-tauri/target/release/bundle/dmg/*.dmg "${DIST_DIR}/" 2>/dev/null || true
        fi
        ;;
    
    windows)
        cp "${PROJECT_ROOT}/target/release/ai-runtime.exe" "${DIST_DIR}/"
        cp "${PROJECT_ROOT}/target/release/kernel_interface.dll" "${DIST_DIR}/"
        
        if [ -d "${PROJECT_ROOT}/shell/tauri-app/src-tauri/target/release/bundle/msi" ]; then
            cp "${PROJECT_ROOT}"/shell/tauri-app/src-tauri/target/release/bundle/msi/*.msi "${DIST_DIR}/" 2>/dev/null || true
        fi
        ;;
esac

# Copy models
mkdir -p "${DIST_DIR}/models"
cp "${PROJECT_ROOT}/models/pretrained"/*.onnx "${DIST_DIR}/models/" 2>/dev/null || true

# Copy configuration
mkdir -p "${DIST_DIR}/config"
cp "${PROJECT_ROOT}/core/ai-runtime/config/default.toml" "${DIST_DIR}/config/runtime.toml"

echo -e "${GREEN}✓ Distribution packaged${NC}\n"

# Print summary
echo -e "${GREEN}================================${NC}"
echo -e "${GREEN}Build Complete!${NC}"
echo -e "${GREEN}================================${NC}\n"

echo -e "${BLUE}Distribution contents:${NC}"
ls -lh "${DIST_DIR}"

echo -e "\n${BLUE}Next steps:${NC}"
echo -e "1. Test locally: ${GREEN}cd dist && ./ai-runtime${NC}"
echo -e "2. Install system-wide: ${GREEN}cd dist && sudo ./install.sh${NC} (Linux only)"
echo -e "3. Launch shell: ${GREEN}Open the .AppImage/.dmg/.msi file${NC}"

echo -e "\n${YELLOW}Documentation:${NC}"
echo -e "  - User Guide: ${DIST_DIR}/../docs/guides/user-guide.md"
echo -e "  - API Reference: ${DIST_DIR}/../docs/api/grpc-reference.md"
echo -e "  - Contributing: ${DIST_DIR}/../CONTRIBUTING.md"

echo -e "\n${GREEN}Deployment script completed successfully!${NC}"
