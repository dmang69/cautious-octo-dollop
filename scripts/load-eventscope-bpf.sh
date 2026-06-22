#!/usr/bin/env bash
# IntentKernel EventScope Phase 2 — build BPF object and optionally attach LSM programs.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BPF_SRC="${ROOT}/kernel/eventscope-ebpf/bpf/eventscope.bpf.c"
BPF_OUT="${ROOT}/kernel/eventscope-ebpf/target/bpf/eventscope.bpf.o"
ARCH="$(uname -m)"

usage() {
  cat <<'EOF'
Usage: load-eventscope-bpf.sh <command>

Commands:
  build     Compile eventscope.bpf.c → target/bpf/eventscope.bpf.o
  probe     Show loader/BPF readiness (no root required)
  load      Build + attach via eventscope-lsm (requires root/CAP_BPF)
  mock      Run LSM daemon in mock mode (no kernel attach)

Requirements (build):
  clang, llvm, kernel headers (linux-headers-$(uname -r)), libbpf-dev

Requirements (load):
  CAP_BPF or root, CONFIG_BPF_LSM=y, bpftool (optional diagnostics)

WSL2 notes:
  Full LSM attach often fails without a custom kernel. Use `mock` for policy tests.
EOF
}

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "error: missing $1" >&2
    exit 1
  }
}

map_arch() {
  case "${ARCH}" in
    x86_64) echo x86 ;;
    aarch64) echo arm64 ;;
    *) echo "unsupported arch: ${ARCH}" >&2; exit 1 ;;
  esac
}

cmd_build() {
  need_cmd clang
  mkdir -p "$(dirname "${BPF_OUT}")"
  local target_arch
  target_arch="$(map_arch)"
  local inc="/usr/include/${ARCH}-linux-gnu"
  if [[ ! -d "${inc}" ]]; then
    inc="/usr/include"
  fi
  echo "==> compiling ${BPF_SRC}"
  clang -g -O2 -target bpf -D__TARGET_ARCH_${target_arch} \
    -I"${inc}" \
    -c "${BPF_SRC}" -o "${BPF_OUT}"
  echo "==> wrote ${BPF_OUT}"
  if command -v llvm-objdump >/dev/null 2>&1; then
    llvm-objdump -h "${BPF_OUT}" | head -20 || true
  fi
}

cmd_probe() {
  export EVENTSCOPE_BPF_OBJ="${BPF_OUT}"
  if [[ -f "${BPF_OUT}" ]]; then
    echo "BPF object: present (${BPF_OUT})"
  else
    echo "BPF object: missing — run: $0 build"
  fi
  if [[ -r /proc/config.gz ]]; then
    zcat /proc/config.gz | grep -E 'CONFIG_BPF_LSM|CONFIG_LSM' || true
  elif [[ -f "/boot/config-$(uname -r)" ]]; then
    grep -E 'CONFIG_BPF_LSM|CONFIG_LSM' "/boot/config-$(uname -r)" || true
  else
    echo "kernel config: unavailable (check CONFIG_BPF_LSM manually)"
  fi
  cargo run -p eventscope-lsm -- --mock 2>/dev/null || \
    cargo run -p eventscope-lsm -- --mock
}

cmd_load() {
  cmd_build
  export EVENTSCOPE_BPF_OBJ="${BPF_OUT}"
  export RUST_LOG="${RUST_LOG:-info}"
  echo "==> attaching LSM programs (root recommended)"
  if [[ "$(id -u)" -ne 0 ]]; then
    echo "warning: not root — attach may fail with EPERM" >&2
  fi
  cargo build -p eventscope-ebpf --features bpf --bin eventscope-bpf-loader
  cargo run -p eventscope-ebpf --features bpf --bin eventscope-bpf-loader -- --probe || true
  cargo run -p eventscope-lsm --features bpf -- --load-bpf --mock --stdin-json
}

cmd_mock() {
  export RUST_LOG="${RUST_LOG:-info}"
  echo "==> mock LSM daemon (userspace map only)"
  cargo run -p eventscope-lsm -- --mock --stdin-json
}

main() {
  local cmd="${1:-probe}"
  case "${cmd}" in
    build) cmd_build ;;
    probe) cmd_probe ;;
    load) cmd_load ;;
    mock) cmd_mock ;;
    -h|--help|help) usage ;;
    *)
      echo "unknown command: ${cmd}" >&2
      usage
      exit 1
      ;;
  esac
}

main "$@"