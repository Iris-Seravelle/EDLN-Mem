#include "eidolon.h"
#include <linux/userfaultfd.h>
#include <sys/ioctl.h>
#include <unistd.h>
#include <errno.h>

#ifndef UFFDIO_MOVE
#define UFFDIO_MOVE _IOWR(UFFDIO, _UFFDIO_MOVE, struct uffdio_move)
#define _UFFDIO_MOVE (0x05)
struct uffdio_move {
    __u64 dst;
    __u64 src;
    __u64 len;
    __u64 mode;
    __s64 move;
};
#define UFFDIO_MOVE_MODE_ALLOW_SRC_HOLES (1<<0)
#endif

int resolve_fault_copy(int uffd, uint64_t fault_addr, uint64_t len, void* source_page) {
    struct uffdio_copy uffd_copy = {
        .dst = fault_addr & ~(len - 1),
        .src = (uintptr_t)source_page,
        .len = len,
        .mode = 0,
    };

    if (ioctl(uffd, UFFDIO_COPY, &uffd_copy) == -1) {
        return -errno;
    }
    return 0;
}

int resolve_fault_move(int uffd, uint64_t fault_addr, uint64_t len, void* source_page) {
    struct uffdio_move uffd_move = {
        .dst = fault_addr & ~(len - 1),
        .src = (uintptr_t)source_page,
        .len = len,
        .mode = UFFDIO_MOVE_MODE_ALLOW_SRC_HOLES,
    };

    if (ioctl(uffd, UFFDIO_MOVE, &uffd_move) == -1) {
        return -errno;
    }
    return 0;
}

int resolve_fault_wp(int uffd, uint64_t fault_addr, uint64_t len, int protect) {
    struct uffdio_writeprotect uffd_wp = {
        .range = {
            .start = fault_addr & ~(len - 1),
            .len = len,
        },
        .mode = protect ? UFFDIO_WRITEPROTECT_MODE_WP : 0,
    };

    if (ioctl(uffd, UFFDIO_WRITEPROTECT, &uffd_wp) == -1) {
        return -errno;
    }
    return 0;
}

#ifndef UFFDIO_CONTINUE
#define UFFDIO_CONTINUE _IOWR(UFFDIO, _UFFDIO_CONTINUE, struct uffdio_continue)
#define _UFFDIO_CONTINUE (0x07)
struct uffdio_continue {
    struct uffdio_range range;
    __u64 mode;
    __s64 mapped;
};
#endif

int resolve_fault_continue(int uffd, uint64_t fault_addr, uint64_t len) {
    struct uffdio_continue uffd_cont = {
        .range = {
            .start = fault_addr & ~(len - 1),
            .len = len,
        },
        .mode = 0,
    };

    if (ioctl(uffd, UFFDIO_CONTINUE, &uffd_cont) == -1) {
        return -errno;
    }
    return 0;
}
