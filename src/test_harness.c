#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>
#include <time.h>
#ifdef _WIN32
#include <windows.h>
#endif
#include "reference/capability_core.h"

static int failures = 0;

static void expect_equal(const char *label, int expected, int actual) {
    if (expected == actual) {
        printf("[PASS] %s\n", label);
        return;
    }

    printf("[FAIL] %s (expected %d, got %d)\n", label, expected, actual);
    failures++;
}

uint64_t get_time(void) {
#ifdef _WIN32
    LARGE_INTEGER frequency, counter;
    QueryPerformanceFrequency(&frequency);
    QueryPerformanceCounter(&counter);
    return (uint64_t)(counter.QuadPart * 1000000000ULL / frequency.QuadPart);
#else
    struct timespec ts;
    if (timespec_get(&ts, TIME_UTC) != TIME_UTC) {
        fputs("timespec_get failed\n", stderr);
        abort();
    }

    return (uint64_t)ts.tv_sec * 1000000000ULL + (uint64_t)ts.tv_nsec;
#endif
}

int getrandom(void *buf, size_t len, int f) {
    static int seeded = 0;
    unsigned char *bytes = (unsigned char *)buf;
    (void)f;

    if (!seeded) {
        srand((unsigned int)time(NULL));
        seeded = 1;
    }

    for (size_t i = 0; i < len; i++) {
        bytes[i] = (unsigned char)rand();
    }

    return 0;
}

int main(void) {
    int file_cap_id;
    int net_cap_id;
    int result;
    struct Capability file_cap;
    struct Capability net_cap;
    struct Capability forged_cap;
    struct Capability forged_type_cap;

    printf("IntentKernel Capability System Test Harness\n");
    printf("==========================================\n\n");

    file_cap_id = capability_create(1, 5000000000ULL, 1);
    if (file_cap_id < 0) {
        printf("[FAIL] Failed to create file capability\n");
        return 1;
    }

    net_cap_id = capability_create(2, 10000000000ULL, 3);
    if (net_cap_id < 0) {
        printf("[FAIL] Failed to create network capability\n");
        return 1;
    }

    file_cap = cap_table[file_cap_id];
    result = capability_validate(&file_cap);
    expect_equal("Single-use capability validates once", 1, result);
    result = capability_validate(&file_cap);
    expect_equal("Single-use capability is rejected on second use", -1, result);

    net_cap = cap_table[net_cap_id];
    forged_cap = net_cap;
    memset(forged_cap.key, 0, sizeof(forged_cap.key));
    expect_equal("Forged capability is rejected", -1, capability_validate(&forged_cap));
    forged_type_cap = net_cap;
    forged_type_cap.type = 99;
    expect_equal("Capability with mismatched type is rejected", -1, capability_validate(&forged_type_cap));
    expect_equal("Multi-use capability validates on first use", 2, capability_validate(&net_cap));
    expect_equal("Multi-use capability validates on second use", 2, capability_validate(&net_cap));
    expect_equal("Multi-use capability validates on third use", 2, capability_validate(&net_cap));
    expect_equal("Multi-use capability is rejected after exhausting uses", -1, capability_validate(&net_cap));

    expect_equal("Capability revoke succeeds", 0, capability_revoke(net_cap_id));
    net_cap = cap_table[net_cap_id];
    expect_equal("Revoked capability is rejected", -1, capability_validate(&net_cap));

    if (failures == 0) {
        printf("\nAll host capability tests passed.\n");
        return 0;
    }

    printf("\n%d host capability test(s) failed.\n", failures);
    return 1;
}
