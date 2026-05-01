<img align="left" width="180" height="180" src="https://wsrv.nl/?url=https://i.ibb.co/3y9ChRnS/1777606677573-modified.png&output=webp&maxage=1y&q=40&w=550" alt="Eidolon Logo" style="margin-right: 20px;">

<h2 align="center">EDLN-Mem (Eidolon)</h2>
<p align="center"><i>A Study in User-Space Memory Orchestration and Zero-Copy Fault Resolution</i></p>

<div align="center">
  <img src="https://img.shields.io/badge/version-0.1.0-blue.svg?style=flat-square" alt="Version">
  <img src="https://img.shields.io/badge/language-Rust%2FC-orange.svg?style=flat-square&logo=rust" alt="Language">
  <img src="https://img.shields.io/badge/license-GPL--3.0-red.svg?style=flat-square" alt="License">
  <img src="https://img.shields.io/badge/status-Academic--PoC-yellow.svg?style=flat-square" alt="Status">
</div>

<br>

<div align="center">

### Abstract
  
**This research project presents EDLN-Mem (Eidolon), a high-performance framework designed for hybrid user-space memory management on the Linux kernel. By leveraging the `userfaultfd` subsystem, the framework demonstrates the feasibility of delegating page-fault resolution to an external monitor process, achieving sub-40µs latencies. This implementation explores the trade-offs between Rust's safety guarantees in the control plane and C's low-level performance in the hot-path kernel interface.**
  
</div>

<br>

<div align="center">
  <a href="docs/usage.md">Deployment Protocol</a> • 
  <a href="docs/architecture.md">Theoretical Framework</a> • 
  <a href="docs/benchmarks/release_128mb.md">Empirical Results</a> • 
  <a href="docs/plan.md">Future Directions</a>
</div>

<br clear="left"/>

---

## 1. Introduction and Theoretical Framework

EDLN-Mem (Eidolon) serves as an experimental platform for investigating memory delegation patterns in distributed and high-performance computing. The framework facilitates a decoupled architecture where target applications offload memory page-fault handling to a specialized monitor. This study prioritizes a **Zero-Copy-First** methodology, aiming to eliminate redundant kernel-to-user transitions and minimize memory pressure during high-throughput fault resolution.

## 2. Empirical Evaluation

Experimental data derived from a high-pressure workload (32,768 unique page faults across a 128MB dataset) indicates significant performance efficiency:

- **Mean Resolution Latency ($\mu$):** **~35.3 µs**
- **Sustained Resolution Throughput:** **~114 MB/s** (Single-threaded)
- **Initial Cold-Start Penalty:** ~1.3 ms (Attributed to hardware I/O latency)

*Comprehensive environment-specific datasets are documented in the [Empirical Results](docs/benchmarks/release_128mb.md) report.*

---

## 3. Methodological Specifications

The Eidolon implementation integrates several critical technologies to achieve its performance benchmarks:

- **Hybrid Control/Data Planes:** Orchestration and IPC are managed via a safe Rust control plane, while performance-critical `ioctl` operations are executed through an FFI-bridged C data plane.
- **Adaptive Page-Table Mapping:** The system employs runtime kernel capability detection to prioritize `UFFDIO_MOVE` (Zero-Copy) over traditional `UFFDIO_COPY` fallback mechanisms.
- **Multi-tiered Memory Support:** Native integration for standard 4KB pages and optimized 2MB HugePages (HugeTLB), including autonomous tier-switching protocols.
- **Introspective Telemetry:** Real-time analysis of address-space heat maps and latency distributions for precise performance auditing.
- **Hardened Handshake Protocol:** Secure file descriptor transfer via Unix Domain Sockets using the `SCM_RIGHTS` mechanism, ensuring process isolation.

---

## 4. Documentation Index

The following technical supplements provide exhaustive detail on the system's internal mechanics:

- [**Theoretical Framework**](docs/architecture.md): Formal analysis of internal state machines, IPC protocols, and fault-resolution logic.
- [**Deployment Protocol**](docs/usage.md): Procedural instructions for instrumenting and deploying target applications.
- [**Benchmarking Methodology**](docs/benchmarks/release_128mb.md): Rigorous definition of test workloads and analytical results.
- [**Future Directions**](docs/plan.md): Project genesis, academic RFCs, and the planned feature escalation roadmap.

---

## 5. Experimental Setup and Execution

### 5.1 Environment Requirements
The laboratory environment requires `rustc` (2024 edition), `clang`, and a Linux kernel version 5.4 or higher.

```bash
cargo build --release --bins
```

### 5.2 Deployment Protocol
Initiate the monitor as a background daemon, followed by the target application to trigger lazy-loading behaviors.

```bash
./target/release/eidolon &
./target/release/target_app
```

### 5.3 Observation and Data Collection
The monitor outputs real-time metrics to `stdout` for empirical observation:
```text
[Monitor] Performance: 20000 faults, Avg Latency: 35.30µs, Data Resolved: 78.12MB
--- [Telemetry Report] ---
Total Faults: 32768
Avg Latency:  35.421µs
Hot Pages (top 5):
  0x6df293b000: 1 accesses
```

---

## 6. Component Inventory

- `src/main.rs`: High-concurrency event multiplexer.
- `c_src/`: Low-level kernel interface primitives.
- `src/pool.rs`: Page-alignment and HugePages management logic.
- `docs/`: Supplemental technical reports and experimental data.

---

<div align="center">

Licensed under **GPL-3.0** [LICENSE](LICENSE)

Research led by [Iris Seravelle](https://github.com/Iris-Seravelle)

</div>
