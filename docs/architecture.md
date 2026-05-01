# Theoretical Framework: User-Space Memory Orchestration

## 1. Abstract
This document outlines the architectural specifications of EDLN-Mem (Eidolon), focusing on the decoupling of memory management from the kernel space to the user space. The framework utilizes the `userfaultfd` subsystem to intercept page faults and resolve them through a dedicated monitor process.

## 2. Component Architecture
The system is bifurcated into two primary operational entities:

### 2.1 The Target Process
The application under observation. It initializes a memory region with restricted permissions (typically anonymous or `MAP_SHARED` with `PROT_NONE` or unpopulated states) and registers this region with a `userfaultfd` object.

### 2.2 The Eidolon Monitor (EDLN-Mem)
A high-concurrency daemon that handles intercepted faults.
*   **Control Plane (Rust):** Manages IPC, lifecycle events, and target multiplexing using safe abstractions.
*   **Data Plane (C/FFI):** Executes high-performance `ioctl` operations for page mapping and remapping.

## 3. Communication Protocol
Process isolation is maintained through a Unix Domain Socket handshake.

1.  **Handshake:** The Target process establishes a connection to `eidolon.sock`.
2.  **Descriptor Passing:** The Target transmits the `userfaultfd` file descriptor to the Monitor via `SCM_RIGHTS` (ancillary data).
3.  **Metadata Exchange:** The Target provides the virtual address range, total length, and expected page size (standard or HugePages).

## 4. Fault Resolution State Machine
When a thread in the Target process accesses an unmapped page:

1.  **Interception:** The kernel suspends the faulting thread and generates an `UFFD_EVENT_PAGEFAULT`.
2.  **Notification:** The Monitor, polling the `userfaultfd` descriptor, receives the event.
3.  **Data Retrieval:** The Monitor fetches the required page data from a `DataSource` (e.g., `FileSource`).
4.  **Resolution:**
    *   **Primary Path (`UFFDIO_MOVE`):** Zero-copy page-table remapping. Available on Linux 6.7+.
    *   **Fallback Path (`UFFDIO_COPY`):** Data copy into the kernel's address space.
5.  **Resumption:** The kernel wakes the faulting thread, which resumes execution transparently.

## 5. Performance Optimizations
*   **Pre-allocated Page Pools:** Minimizes allocation overhead during the hot-path resolution.
*   **Thread Affinity:** The Monitor is pinned to a specific CPU core to reduce cache misses and context-switching jitter.
*   **HugePages (2MB):** Support for `hugetlbfs` to reduce TLB pressure and fault frequency.
