# Deployment Protocol: Experimental Setup and Instrumentation

## 1. Scope
This protocol defines the procedures for instrumenting a target application and deploying the Eidolon monitor to facilitate user-space memory management.

## 2. Target Application Instrumentation

### 2.1 Registration Metadata
Target applications must provide the monitor with specific metadata regarding the memory region to be managed.

```rust
#[derive(serde::Serialize)]
struct Registration {
    addr: usize,      // Base virtual address
    len: usize,       // Total length of the region
    page_size: usize, // Architecture-specific page size (e.g., 4096 or 2097152)
}
```

### 2.2 System Call Configuration
Initialize the `userfaultfd` subsystem and register the target memory range.

```rust
let uffd = UffdBuilder::new()
    .close_on_exec(true)
    .non_blocking(true)
    .create()?;

// Register for MISSING and optionally WRITE_PROTECT/MINOR faults
uffd.register(addr, len)?;
```

### 2.3 Handshake and Descriptor Transfer
The `userfaultfd` file descriptor is transferred to the monitor using the `SCM_RIGHTS` mechanism over a Unix Domain Socket (`eidolon.sock`).

```rust
let stream = UnixStream::connect("eidolon.sock")?;
let reg = Registration { addr, len, page_size: 4096 };
let encoded = bincode::serialize(&reg)?;

// Transfer FD via ScmRights to bypass process boundaries
sendmsg::<()>(stream.as_raw_fd(), &[IoSlice::new(&encoded)], 
             &[ControlMessage::ScmRights(&[uffd.as_raw_fd()])], 
             MsgFlags::empty(), None)?;
```

## 3. Monitor Deployment

### 3.1 Pre-deployment Configuration
The monitor requires a backing data source. For standard experiments, ensure `backing_data.bin` exists or is configured via the `DataSource` trait.

### 3.2 Execution
Deploy the monitor with appropriate privileges (e.g., `CAP_SYS_PTRACE` or `vm.unprivileged_userfaultfd=1`).

```bash
# Compile and execute the monitor
cargo build --release
./target/release/eidolon
```

## 4. Empirical Data Collection
The monitor provides real-time telemetry regarding fault resolution performance.
*   **Avg Latency:** The arithmetic mean of resolution time for the last $N$ faults.
*   **Hot Pages:** Address-level access frequency mapping.
*   **Throughput:** Total data resolved in MB/s.
