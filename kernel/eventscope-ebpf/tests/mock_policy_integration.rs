//! Integration test: simulate BPF map write + LSM policy check without root.

use eventscope_ebpf::bridge::{replace_global_bridge, MockKernelBridge};
use eventscope_ebpf::policy::{
    evaluate_hook, HandleMapEntry, SyscallHook, RESOURCE_FILE, RESOURCE_NETWORK,
};

#[test]
fn map_write_then_openat_allowed() {
    replace_global_bridge(Box::new(MockKernelBridge::new()));

    let pid = 4242u32;
    let handle = 0x0001_0002_00ABu64;
    eventscope_ebpf::publish_handle(HandleMapEntry::new(pid, handle, RESOURCE_FILE))
        .expect("publish");

    let map = eventscope_ebpf::global_bridge()
        .lock()
        .expect("lock")
        .snapshot();

    let verdict = evaluate_hook(SyscallHook::OpenAt, pid, &map);
    assert!(verdict.is_allow(), "openat should allow with file handle");
}

#[test]
fn map_write_connect_requires_network_type() {
    replace_global_bridge(Box::new(MockKernelBridge::new()));

    let pid = 5150u32;
    eventscope_ebpf::publish_handle(HandleMapEntry::new(pid, 99, RESOURCE_FILE))
        .expect("publish file handle");

    let map = eventscope_ebpf::global_bridge()
        .lock()
        .expect("lock")
        .snapshot();

    assert!(
        evaluate_hook(SyscallHook::Connect, pid, &map).is_deny(),
        "connect must deny file-only handle"
    );

    eventscope_ebpf::publish_handle(HandleMapEntry::new(pid, 99, RESOURCE_NETWORK))
        .expect("publish network handle");

    let map = eventscope_ebpf::global_bridge()
        .lock()
        .expect("lock")
        .snapshot();

    assert!(
        evaluate_hook(SyscallHook::Connect, pid, &map).is_allow(),
        "connect should allow network handle"
    );
}

#[test]
fn revoke_denies_subsequent_open() {
    replace_global_bridge(Box::new(MockKernelBridge::new()));

    let pid = 6000u32;
    eventscope_ebpf::publish_handle(HandleMapEntry::new(pid, 1, RESOURCE_FILE))
        .expect("publish");
    eventscope_ebpf::revoke_pid(pid).expect("revoke");

    let map = eventscope_ebpf::global_bridge()
        .lock()
        .expect("lock")
        .snapshot();

    assert!(evaluate_hook(SyscallHook::OpenAt, pid, &map).is_deny());
}

#[test]
fn no_map_entry_denies_by_default() {
    replace_global_bridge(Box::new(MockKernelBridge::new()));
    let map = eventscope_ebpf::global_bridge()
        .lock()
        .expect("lock")
        .snapshot();

    assert!(evaluate_hook(SyscallHook::OpenAt, 1, &map).is_deny());
    assert!(evaluate_hook(SyscallHook::Connect, 1, &map).is_deny());
}