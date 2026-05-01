use nix::sys::socket::{ControlMessage, MsgFlags, sendmsg};
use std::io::IoSlice;
use std::os::unix::io::AsRawFd;
use std::os::unix::net::UnixStream;
use std::ptr;
use userfaultfd::{RegisterMode, UffdBuilder};

// Importing the same Registration struct logic (inline for simplicity in this MVP)
#[derive(serde::Serialize)]
struct Registration {
    addr: usize,
    len: usize,
    page_size: usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let page_size = 4096;
    let num_pages = 32768; // 128MB total with 4KB pages
    let len = page_size * num_pages;

    println!(
        "[Target] Attempting HugePages allocation ({} bytes)...",
        len
    );
    let mut addr = unsafe {
        libc::mmap(
            ptr::null_mut(),
            len,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED | libc::MAP_ANONYMOUS | libc::MAP_HUGETLB,
            -1,
            0,
        )
    };
    if addr == libc::MAP_FAILED {
        println!("[Target] HugePages not available, using 4KB pages");
        // page_size and num_pages already set for 4KB/128MB
        addr = unsafe {
            libc::mmap(
                ptr::null_mut(),
                len,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED | libc::MAP_ANONYMOUS,
                -1,
                0,
            )
        };
    }

    if addr == libc::MAP_FAILED {
        return Err("mmap failed".into());
    }

    let actual_len = len; // Correctly use the full length

    println!(
        "[Target] Allocation successful: addr=0x{:x}, page_size={}",
        addr as usize, page_size
    );

    println!("[Target] Initializing userfaultfd...");
    let uffd = UffdBuilder::new()
        .close_on_exec(true)
        .non_blocking(true)
        .create()?;

    // Register for MISSING, WRITE_PROTECT, and MINOR faults, with fallbacks
    let mode = RegisterMode::MISSING | RegisterMode::WRITE_PROTECT | RegisterMode::MINOR;
    if let Err(e) = uffd.register_with_mode(addr as *mut _, actual_len, mode) {
        println!(
            "[Target] Full mode registration failed ({}), falling back to MISSING | WP",
            e
        );
        if let Err(e2) = uffd.register_with_mode(
            addr as *mut _,
            actual_len,
            RegisterMode::MISSING | RegisterMode::WRITE_PROTECT,
        ) {
            println!(
                "[Target] WP registration failed ({}), falling back to MISSING only",
                e2
            );
            uffd.register_with_mode(addr as *mut _, actual_len, RegisterMode::MISSING)?;
        }
    }

    println!("[Target] Connecting to monitor...");
    let stream = UnixStream::connect("eidolon.sock")?;
    let fd = stream.as_raw_fd();

    let reg = Registration {
        addr: addr as usize,
        len: actual_len,
        page_size,
    };
    let encoded = bincode::serialize(&reg)?;
    let iov = [IoSlice::new(&encoded)];

    let uffd_raw = uffd.as_raw_fd();
    let cmsgs = [ControlMessage::ScmRights(&[uffd_raw])];

    println!("[Target] Sending uffd via SCM_RIGHTS...");
    sendmsg::<()>(fd, &iov, &cmsgs, MsgFlags::empty(), None)?;

    // Give the monitor a moment to register the FD
    std::thread::sleep(std::time::Duration::from_millis(100));

    println!("[Target] Triggering page faults...");
    let slice = unsafe { std::slice::from_raw_parts_mut(addr as *mut u8, actual_len) };

    let actual_num_pages = actual_len / page_size;
    let backing_file_len = 128 * 1024 * 1024;

    for iteration in 0..10 {
        println!("[Target] Iteration {}...", iteration);
        for i in 0..actual_num_pages {
            let offset = i * page_size;
            let val = slice[offset];

            // The monitor's FileSource reads from (addr % backing_file_len)
            // So at offset i, it should be ((addr + offset) % backing_file_len) & 0xFF
            let expected = (((addr as usize + offset) % backing_file_len) & 0xFF) as u8;
            if val != expected {
                return Err(format!(
                    "Validation failed at page {} (iter {}): expected 0x{:02x}, got 0x{:02x}",
                    i, iteration, expected, val
                )
                .into());
            }

            // Write back to the same page to ensure it's "dirty" and mapped
            slice[offset] = val;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    println!("[Target] All faults resolved and validated!");
    Ok(())
}
