#!/usr/bin/env bash
# =============================================================================
# IntentOS — macOS Universal Installer
# Supports: macOS 12 (Monterey) and later, Intel and Apple Silicon
# Preserves ALL existing software, files, and settings.
# =============================================================================

set -euo pipefail

# ── Colours ──────────────────────────────────────────────────────────────────
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
BLUE='\033[0;34m'; BOLD='\033[1m'; NC='\033[0m'

log_info()  { echo -e "${GREEN}[✓]${NC} $*"; }
log_warn()  { echo -e "${YELLOW}[!]${NC} $*"; }
log_error() { echo -e "${RED}[✗]${NC} $*" >&2; }
log_step()  { echo -e "\n${BOLD}${BLUE}──${NC} ${BOLD}$*${NC}"; }
banner() {
    echo -e "${BOLD}"
    echo "  ╔══════════════════════════════════════════╗"
    echo "  ║         IntentOS Upgrade Layer           ║"
    echo "  ║   AI · Security · Communication · Safe   ║"
    echo "  ╚══════════════════════════════════════════╝"
    echo -e "${NC}"
}

# ── Configuration ─────────────────────────────────────────────────────────────
INTENTOS_HOME="/opt/intentos"
INTENTOS_VENV="${INTENTOS_HOME}/venv"
INTENTOS_LOG="/var/log/intentos"
INTENTOS_RUN="/var/run/intentos"
PLIST_LABEL="com.intentos.daemon"
PLIST_FILE="/Library/LaunchDaemons/${PLIST_LABEL}.plist"
REPO_URL="https://github.com/dmang69/cautious-octo-dollop"

# Source directory (when installing from local clone)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

# ── Checks ────────────────────────────────────────────────────────────────────
check_root() {
    if [[ $EUID -ne 0 ]]; then
        log_error "This installer must be run as root (use: sudo bash install.sh)"
        exit 1
    fi
}

check_macos() {
    if [[ "$(uname -s)" != "Darwin" ]]; then
        log_error "This installer is for macOS only."
        exit 1
    fi

    OS_VER=$(sw_vers -productVersion)
    MAJOR=$(echo "${OS_VER}" | cut -d. -f1)
    if [[ "${MAJOR}" -lt 12 ]]; then
        log_error "macOS 12 (Monterey) or later is required. Found: ${OS_VER}"
        exit 1
    fi
    log_info "macOS ${OS_VER} detected"
}

check_existing() {
    if launchctl list "${PLIST_LABEL}" &>/dev/null; then
        log_warn "IntentOS is already running.  Upgrading in place…"
        launchctl unload "${PLIST_FILE}" 2>/dev/null || true
    fi
}

# ── Install Homebrew and system packages ──────────────────────────────────────
install_dependencies() {
    log_step "Installing system dependencies"

    if ! command -v brew &>/dev/null; then
        log_warn "Homebrew not found.  Installing Homebrew…"
        # Run brew install as the invoking user (sudo strips $USER)
        BREW_USER="${SUDO_USER:-${USER}}"
        su - "${BREW_USER}" -c \
            'NONINTERACTIVE=1 /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"'
        # Add brew to PATH for the remainder of this script
        if [[ "$(uname -m)" == "arm64" ]]; then
            eval "$(/opt/homebrew/bin/brew shellenv)"
        else
            eval "$(/usr/local/bin/brew shellenv)"
        fi
        log_info "Homebrew installed"
    fi

    BREW_USER="${SUDO_USER:-${USER}}"
    if ! su - "${BREW_USER}" -c "brew install python3 wget git"; then
        log_warn "Some Homebrew packages may not have installed correctly"
    fi

    if ! command -v python3 &>/dev/null; then
        log_error "Python 3 installation failed.  Please install python3 manually."
        exit 1
    fi

    PY_VER=$(python3 --version 2>&1)
    log_info "Python: ${PY_VER}"
}

# ── Copy application files ─────────────────────────────────────────────────────
install_app() {
    log_step "Installing application to ${INTENTOS_HOME}"

    mkdir -p "${INTENTOS_HOME}" "${INTENTOS_LOG}" "${INTENTOS_RUN}"

    if [[ -d "${REPO_ROOT}/platform" ]]; then
        # Installing from local clone
        cp -r "${REPO_ROOT}/platform/." "${INTENTOS_HOME}/"
        log_info "Copied from local repository"
    else
        # Fetch from GitHub
        log_info "Downloading from GitHub…"
        if command -v git &>/dev/null; then
            if git clone --depth=1 "${REPO_URL}" /tmp/intentos-src; then
                if [[ -d /tmp/intentos-src/platform ]]; then
                    cp -r /tmp/intentos-src/platform/. "${INTENTOS_HOME}/"
                    rm -rf /tmp/intentos-src
                else
                    rm -rf /tmp/intentos-src
                    log_error "Repository does not contain a 'platform' directory."
                    exit 1
                fi
            else
                log_error "Failed to clone repository from ${REPO_URL}"
                exit 1
            fi
        else
            log_error "git is not installed and no local source found.  Cannot install application files."
            exit 1
        fi
    fi

    chmod -R 755 "${INTENTOS_HOME}"
    log_info "Application files installed"
}

# ── Create virtual environment and install Python deps ───────────────────────
setup_venv() {
    log_step "Setting up Python virtual environment"

    python3 -m venv "${INTENTOS_VENV}"
    "${INTENTOS_VENV}/bin/pip" install --upgrade pip

    if [[ -f "${INTENTOS_HOME}/requirements.txt" ]]; then
        "${INTENTOS_VENV}/bin/pip" install -r "${INTENTOS_HOME}/requirements.txt"
        log_info "Python dependencies installed"
    else
        "${INTENTOS_VENV}/bin/pip" install flask
        log_info "Core dependencies installed"
    fi
}

# ── Install LaunchDaemon ───────────────────────────────────────────────────────
install_service() {
    log_step "Installing LaunchDaemon"

    cat > "${PLIST_FILE}" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
    "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>${PLIST_LABEL}</string>

    <key>ProgramArguments</key>
    <array>
        <string>${INTENTOS_VENV}/bin/python</string>
        <string>${INTENTOS_HOME}/daemon/intentos_daemon.py</string>
        <string>--host</string>
        <string>127.0.0.1</string>
        <string>--port</string>
        <string>5000</string>
    </array>

    <key>WorkingDirectory</key>
    <string>${INTENTOS_HOME}</string>

    <key>EnvironmentVariables</key>
    <dict>
        <key>INTENTOS_LOG_DIR</key>
        <string>${INTENTOS_LOG}</string>
        <key>INTENTOS_RUN</key>
        <string>${INTENTOS_RUN}</string>
        <key>PATH</key>
        <string>${INTENTOS_VENV}/bin:/usr/local/bin:/usr/bin:/bin</string>
    </dict>

    <key>StandardOutPath</key>
    <string>${INTENTOS_LOG}/stdout.log</string>
    <key>StandardErrorPath</key>
    <string>${INTENTOS_LOG}/stderr.log</string>

    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>

    <key>ThrottleInterval</key>
    <integer>10</integer>
</dict>
</plist>
EOF

    chmod 644 "${PLIST_FILE}"
    if ! launchctl load "${PLIST_FILE}"; then
        log_error "Failed to load LaunchDaemon.  Check ${INTENTOS_LOG}/stderr.log for details."
        exit 1
    fi
    log_info "LaunchDaemon installed and started"
}

# ── Create convenience CLI ─────────────────────────────────────────────────────
install_cli() {
    log_step "Installing intentos CLI"

    cat > /usr/local/bin/intentos <<CLISCRIPT
#!/usr/bin/env bash
# IntentOS management CLI
CMD="\${1:-status}"
PLIST_LABEL="${PLIST_LABEL}"
PLIST_FILE="${PLIST_FILE}"
case "\$CMD" in
    start)   launchctl load   "\${PLIST_FILE}" 2>/dev/null; echo "IntentOS started."  ;;
    stop)    launchctl unload "\${PLIST_FILE}" 2>/dev/null; echo "IntentOS stopped."  ;;
    restart) launchctl unload "\${PLIST_FILE}" 2>/dev/null
             launchctl load   "\${PLIST_FILE}" 2>/dev/null
             echo "IntentOS restarted." ;;
    status)  launchctl list "\${PLIST_LABEL}" ;;
    logs)    tail -n "\${2:-50}" ${INTENTOS_LOG}/stdout.log ;;
    open)    open http://localhost:5000 2>/dev/null || \
             echo "Open: http://localhost:5000" ;;
    *)       echo "Usage: intentos {start|stop|restart|status|logs|open}" ;;
esac
CLISCRIPT

    chmod +x /usr/local/bin/intentos
    log_info "CLI installed: 'intentos start|stop|status|logs|open'"
}

# ── Post-install summary ───────────────────────────────────────────────────────
print_summary() {
    echo ""
    echo -e "${BOLD}${GREEN}✓ IntentOS installed successfully!${NC}"
    echo ""
    echo "  Control Surface:  http://localhost:5000"
    echo "  Service status:   intentos status"
    echo "  Logs:             tail -f ${INTENTOS_LOG}/stdout.log"
    echo "  CLI:              intentos {start|stop|restart|status|logs|open}"
    echo ""
    echo -e "${YELLOW}Your existing apps, files, and settings were not modified.${NC}"
    echo ""
}

# ── Main ──────────────────────────────────────────────────────────────────────
main() {
    banner
    check_root
    check_macos
    check_existing
    install_dependencies
    install_app
    setup_venv
    install_service
    install_cli
    print_summary
}

main "$@"
