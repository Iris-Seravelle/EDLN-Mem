#ifndef EIDOLON_H
#define EIDOLON_H

#include <stdint.h>
#include <sys/types.h>

struct edln_event_packet {
    uintptr_t fault_addr;
    pid_t fault_tid;
    uint64_t timestamp_ns;
    uint8_t flags;
};

int resolve_fault_copy(int uffd, uint64_t fault_addr, uint64_t len, void* source_page);
int resolve_fault_move(int uffd, uint64_t fault_addr, uint64_t len, void* source_page);
int resolve_fault_wp(int uffd, uint64_t fault_addr, uint64_t len, int protect);
int resolve_fault_continue(int uffd, uint64_t fault_addr, uint64_t len);

#endif
