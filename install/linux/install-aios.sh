#!/bin/bash
# AI OS Linux Installation Script

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo -e "${RED}Error: This script must be run as root${NC}"
    exit 1
fi

echo -e "${GREEN}================================${NC}"
echo -e "${GREEN}AI OS Installation${NC}"
echo -e "${GREEN}================================${NC}\n"

# Create user and group
echo -e "${YELLOW}Creating ai-os user...${NC}"
if ! id -u ai-os > /dev/null 2>&1; then
    useradd --system --no-create-home --shell /bin/false ai-os
    echo -e "${GREEN}✓ User created${NC}"
else
    echo -e "${YELLOW}User already exists${NC}"
fi

# Create directories
echo -e "${YELLOW}Creating directories...${NC}"
mkdir -p /usr/local/bin
mkdir -p /usr/local/lib
mkdir -p /etc/ai-os
mkdir -p /var/lib/ai-os
mkdir -p /var/log/ai-os
mkdir -p /opt/ai-os/models

chown -R ai-os:ai-os /var/lib/ai-os
chown -R ai-os:ai-os /var/log/ai-os
chmod 755 /etc/ai-os

echo -e "${GREEN}✓ Directories created${NC}"

# Install binaries
echo -e "${YELLOW}Installing binaries...${NC}"
if [ ! -f "dist/ai-runtime" ] || [ ! -f "dist/libkernel_interface.so" ]; then
    echo -e "${RED}Error: dist/ binaries not found. Build the project first (make all).${NC}"
    exit 1
fi
cp dist/ai-runtime /usr/local/bin/
cp dist/libkernel_interface.so /usr/local/lib/
chmod +x /usr/local/bin/ai-runtime

echo -e "${GREEN}✓ Binaries installed${NC}"

# Install ONNX Runtime
echo -e "${YELLOW}Installing ONNX Runtime...${NC}"
if [ ! -d "/usr/local/lib/onnxruntime" ]; then
    if ! wget -q https://github.com/microsoft/onnxruntime/releases/download/v1.16.0/onnxruntime-linux-x64-1.16.0.tgz; then
        echo -e "${RED}Error: Failed to download ONNX Runtime. Check your network connection.${NC}"
        exit 1
    fi
    tar -xzf onnxruntime-linux-x64-1.16.0.tgz
    cp -r onnxruntime-linux-x64-1.16.0/lib/* /usr/local/lib/
    rm -rf onnxruntime-linux-x64-1.16.0*
    ldconfig
    echo -e "${GREEN}✓ ONNX Runtime installed${NC}"
else
    echo -e "${YELLOW}ONNX Runtime already installed${NC}"
fi

# Install models
echo -e "${YELLOW}Installing AI models...${NC}"
if [ -d "models/pretrained" ]; then
    cp models/pretrained/*.onnx /opt/ai-os/models/
    chown -R ai-os:ai-os /opt/ai-os/models
    echo -e "${GREEN}✓ Models installed${NC}"
else
    echo -e "${YELLOW}No pretrained models found (will need to train)${NC}"
fi

# Install configuration
echo -e "${YELLOW}Installing configuration...${NC}"
if [ ! -f "/etc/ai-os/runtime.toml" ]; then
    cp config/runtime.toml /etc/ai-os/
    chown ai-os:ai-os /etc/ai-os/runtime.toml
    chmod 640 /etc/ai-os/runtime.toml
    echo -e "${GREEN}✓ Configuration installed${NC}"
else
    echo -e "${YELLOW}Configuration already exists (not overwriting)${NC}"
fi

# Install systemd service
echo -e "${YELLOW}Installing systemd service...${NC}"
cp platform/linux/systemd/ai-context-manager.service /etc/systemd/system/
systemctl daemon-reload

echo -e "${GREEN}✓ Service installed${NC}"

# Enable and start service
echo -e "${YELLOW}Enabling service...${NC}"
systemctl enable ai-context-manager.service

echo -e "\n${GREEN}================================${NC}"
echo -e "${GREEN}Installation Complete!${NC}"
echo -e "${GREEN}================================${NC}\n"

echo -e "To start the service:"
echo -e "  ${GREEN}sudo systemctl start ai-context-manager${NC}\n"

echo -e "To check status:"
echo -e "  ${GREEN}sudo systemctl status ai-context-manager${NC}\n"

echo -e "To view logs:"
echo -e "  ${GREEN}sudo journalctl -u ai-context-manager -f${NC}\n"

echo -e "Configuration file:"
echo -e "  ${GREEN}/etc/ai-os/runtime.toml${NC}\n"
