mod ffi;
mod ipc;
mod pool;
mod source;
mod telemetry;

use ipc::IpcServer;
use nix::sched::{CpuSet, sched_setaffinity};
use nix::unistd::Pid;
use pool::PagePool;
use source::DataSource;
use std::collections::HashMap;
use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};
use std::time::Instant;
use telemetry::TelemetryTracker;
use userfaultfd::{FaultKind, Uffd};

struct TargetInfo {
    uffd: Uffd,
    page_size: usize,
}

use std::os::unix::fs::FileExt;

fn create_backing_file(path: &str, size: usize) -> Result<(), Box<dyn std::error::Error>> {
    let file = std::fs::File::create(path)?;
    file.set_len(size as u64)?;

    let mut buffer = vec![0u8; size];
    for (i, byte) in buffer.iter_mut().enumerate() {
        *byte = (i & 0xFF) as u8;
    }
    file.write_at(&buffer, 0)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("EDLN-Mem Monitor Starting...");

    // Create a 128MB backing file
    create_backing_file("backing_data.bin", 128 * 1024 * 1024)?;

    // Optimization: Thread Pinning (pin to core 0)
    let mut cpu_set = CpuSet::new();
    if let Err(e) = cpu_set.set(0) {
        eprintln!("Warning: Failed to set CPU bit: {}", e);
    } else if let Err(e) = sched_setaffinity(Pid::from_raw(0), &cpu_set) {
        eprintln!("Warning: Failed to set thread affinity: {}", e);
    } else {
        println!("[Monitor] Thread pinned to core 0");
    }

    let ipc_server = IpcServer::new("eidolon.sock")?;
    let pool = PagePool::new();
    let source = source::FileSource::new("backing_data.bin")?;
    let mut telemetry = TelemetryTracker::new();

    let mut uffds: HashMap<RawFd, TargetInfo> = HashMap::new();
    let mut pollfds: Vec<libc::pollfd> = Vec::new();
    let mut move_supported = None;

    // Initial pollfd: the IPC listener
    pollfds.push(libc::pollfd {
        fd: ipc_server.listener_fd(),
        events: libc::POLLIN,
        revents: 0,
    });

    loop {
        let ret = unsafe { libc::poll(pollfds.as_mut_ptr(), pollfds.len() as libc::nfds_t, -1) };
        if ret <= 0 {
            if ret < 0 {
                let err = std::io::Error::last_os_error();
                if err.kind() != std::io::ErrorKind::Interrupted {
                    return Err(err.into());
                }
            }
            continue;
        }

        let mut new_targets = Vec::new();

        for pollfd in &mut pollfds {
            if pollfd.revents & libc::POLLIN != 0 {
                if pollfd.fd == ipc_server.listener_fd() {
                    // New client registration
                    if let Some((reg, fd)) = ipc_server.accept_registration()? {
                        println!(
                            "[Monitor] Registered new target: addr=0x{:x}, len={}, page_size={}",
                            reg.addr, reg.len, reg.page_size
                        );
                        let uffd = unsafe { Uffd::from_raw_fd(fd) };
                        new_targets.push(TargetInfo {
                            uffd,
                            page_size: reg.page_size,
                        });
                    }
                } else {
                    // Page fault on an existing uffd
                    let fd = pollfd.fd;
                    if let Some(target) = uffds.get(&fd) {
                        while let Some(event) = target.uffd.read_event()? {
                            if let userfaultfd::Event::Pagefault { addr, kind, .. } = event {
                                let start = Instant::now();
                                let fault_addr_usize = addr as usize;

                                match kind {
                                    FaultKind::Missing => {
                                        handle_page_fault(
                                            &target.uffd,
                                            target.page_size,
                                            &pool,
                                            &source,
                                            fault_addr_usize,
                                            &mut move_supported,
                                        )?;
                                    }
                                    FaultKind::WriteProtected => {
                                        // Handle WP fault: remove write protection to allow the write to proceed
                                        unsafe {
                                            let res = ffi::resolve_fault_wp(
                                                target.uffd.as_raw_fd(),
                                                fault_addr_usize as u64,
                                                target.page_size as u64,
                                                0, // protect = 0 (remove)
                                            );
                                            if res < 0 {
                                                eprintln!(
                                                    "Failed to remove WP at 0x{:x}: {}",
                                                    fault_addr_usize, res
                                                );
                                            }
                                        }
                                    }
                                    FaultKind::Minor => {
                                        // Handle Minor fault: map existing page cache entry
                                        unsafe {
                                            let res = ffi::resolve_fault_continue(
                                                target.uffd.as_raw_fd(),
                                                fault_addr_usize as u64,
                                                target.page_size as u64,
                                            );
                                            if res < 0 {
                                                eprintln!(
                                                    "Failed to continue minor fault at 0x{:x}: {}",
                                                    fault_addr_usize, res
                                                );
                                            }
                                        }
                                    }
                                }

                                let duration = start.elapsed();
                                telemetry.record_fault(fault_addr_usize, duration);

                                // Periodically print performance every 1,000 faults
                                if telemetry.total_faults.is_multiple_of(1000) {
                                    let avg_latency = telemetry.total_latency.as_micros() as f64
                                        / telemetry.total_faults as f64;
                                    let total_mb = (telemetry.total_faults as f64
                                        * target.page_size as f64)
                                        / (1024.0 * 1024.0);
                                    println!(
                                        "[Monitor] Performance: {} faults, Avg Latency: {:.2}µs, Data Resolved: {:.2}MB",
                                        telemetry.total_faults, avg_latency, total_mb
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        // Add new targets to our tracking
        for target in new_targets {
            let fd = target.uffd.as_raw_fd();
            uffds.insert(fd, target);
            pollfds.push(libc::pollfd {
                fd,
                events: libc::POLLIN,
                revents: 0,
            });
        }

        // Reset revents
        for pfd in pollfds.iter_mut() {
            pfd.revents = 0;
        }
    }
}

fn handle_page_fault(
    uffd: &Uffd,
    page_size: usize,
    pool: &PagePool,
    source: &dyn DataSource,
    fault_addr: usize,
    move_supported: &mut Option<bool>,
) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Acquire a page from the pool
    let page_ptr = pool.acquire_page(page_size)?;

    // 2. Fetch data from the source
    let buffer = unsafe { std::slice::from_raw_parts_mut(page_ptr, page_size) };
    if let Err(e) = source.fetch_page(fault_addr, buffer) {
        eprintln!("Source fetch failed: {}", e);
        pool.release_page(page_ptr, page_size);
        return Ok(());
    }

    // 3. Resolve the fault adaptively
    unsafe {
        let mut res = -1;

        // Try UFFDIO_MOVE if it's supported or unknown
        if move_supported.unwrap_or(true) {
            res = ffi::resolve_fault_move(
                uffd.as_raw_fd(),
                fault_addr as u64,
                page_size as u64,
                page_ptr as *mut _,
            );

            // If it failed with EINVAL (22) or ENOTTY (25), it's likely not supported
            if res == -22 || res == -25 {
                if move_supported.is_none() {
                    println!(
                        "[Monitor] UFFDIO_MOVE not supported by kernel, falling back to UFFDIO_COPY"
                    );
                    *move_supported = Some(false);
                }
            } else if res == 0 && move_supported.is_none() {
                println!("[Monitor] UFFDIO_MOVE supported, using zero-copy mode");
                *move_supported = Some(true);
            }
        }

        // Fallback to UFFDIO_COPY if MOVE failed or is known to be unsupported
        if !move_supported.unwrap_or(true) || (res != 0 && (res == -22 || res == -25)) {
            res = ffi::resolve_fault_copy(
                uffd.as_raw_fd(),
                fault_addr as u64,
                page_size as u64,
                page_ptr as *mut _,
            );
        }

        if res < 0 {
            eprintln!("Failed to resolve fault at 0x{:x}: {}", fault_addr, res);
        }

        // Always release the page from the monitor's space for safety.
        // munmap is safe to call even if UFFDIO_MOVE already unmapped it.
        pool.release_page(page_ptr, page_size);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffi_linkage() {
        unsafe {
            let res_copy = ffi::resolve_fault_copy(-1, 0, 4096, std::ptr::null_mut());
            assert!(res_copy <= 0);
            let res_move = ffi::resolve_fault_move(-1, 0, 4096, std::ptr::null_mut());
            assert!(res_move <= 0);
        }
    }
}
