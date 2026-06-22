// SPDX-License-Identifier: GPL-2.0 OR MIT
// IntentKernel EventScope Phase 2 — LSM BPF hooks for openat/connect enforcement.
//
// Build (see scripts/load-eventscope-bpf.sh):
//   clang -g -O2 -target bpf -D__TARGET_ARCH_x86 \
//     -I/usr/include/$(uname -m)-linux-gnu \
//     -c bpf/eventscope.bpf.c -o target/bpf/eventscope.bpf.o

#include <linux/bpf.h>
#include <linux/lsm_hooks.h>
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_tracing.h>

#define IK_RESOURCE_FILE    1
#define IK_RESOURCE_NETWORK 2

struct ik_handle_entry {
	__u64 handle;
	__u32 pid;
	__u32 resource_type;
	__u8  valid;
	__u8  _pad[3];
};

struct {
	__uint(type, BPF_MAP_TYPE_HASH);
	__uint(max_entries, 4096);
	__type(key, __u32);
	__type(value, struct ik_handle_entry);
} handle_map SEC(".maps");

static __always_inline int ik_check(__u32 required_type)
{
	__u32 pid = bpf_get_current_pid_tgid() >> 32;
	struct ik_handle_entry *entry = bpf_map_lookup_elem(&handle_map, &pid);

	if (!entry || !entry->valid || entry->handle == 0)
		return -EPERM;

	if (entry->resource_type != required_type)
		return -EPERM;

	return 0;
}

// openat(2) path — LSM file_open hook
SEC("lsm/file_open")
int BPF_PROG(eventscope_file_open, struct file *file)
{
	(void)file;
	return ik_check(IK_RESOURCE_FILE);
}

// connect(2) path — LSM socket_connect hook
SEC("lsm/socket_connect")
int BPF_PROG(eventscope_socket_connect, struct socket *sock,
	     struct sockaddr *address, int addrlen)
{
	(void)sock;
	(void)address;
	(void)addrlen;
	return ik_check(IK_RESOURCE_NETWORK);
}

char LICENSE[] SEC("license") = "Dual MIT/GPL";