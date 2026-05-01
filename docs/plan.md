# Future Directions: Academic Roadmap and Feature Escalation

## 1. Project Genesis
EDLN-Mem (Eidolon) was conceived as a proof-of-concept for investigating the limits of user-space memory management on Linux. The initial phase focused on achieving sub-50µs latencies using `userfaultfd` and `UFFDIO_MOVE`.

## 2. Phase I: Core Consolidation (Current)
*   **Target achieved:** Functional hybrid Rust/C monitor with `SCM_RIGHTS` handshake.
*   **Target achieved:** Empirical verification of 35µs mean latency.
*   **Target achieved:** Integration of standard and HugePages support.

## 3. Phase II: Distributed Memory Expansion (Q3 2026)
This phase will explore the feasibility of remote memory resolution over RDMA or high-speed Ethernet.
*   **RFC 101:** Implementation of an `RDMSource` for zero-copy remote page retrieval.
*   **RFC 102:** Integration of `io_uring` for asynchronous I/O batching in the monitor's hot-path.

## 4. Phase III: Introspective Memory Management (Q1 2027)
Leveraging `userfaultfd` Write-Protect (WP) and Minor-fault features for advanced observability.
*   **Dynamic Tiering:** Automatically migrating "hot" pages to HugePages and "cold" pages to compressed zswap-like buffers.
*   **Predictive Prefetching:** Utilizing ML-based heuristics to resolve faults before they occur based on observed access patterns.

## 5. Academic Contributions
The research team aims to publish a comprehensive analysis of the performance trade-offs in user-space memory management at USENIX ATC or OSDI. Key areas of focus include:
*   The impact of kernel-to-user context switching on jitter.
*   The efficacy of `UFFDIO_MOVE` in highly fragmented physical memory environments.
*   Scalability analysis of multi-tenant monitor deployments.
