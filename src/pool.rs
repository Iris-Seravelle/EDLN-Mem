use std::os::raw::c_void;
use std::ptr;

pub struct PagePool;

impl PagePool {
    pub fn new() -> Self {
        PagePool
    }

    /// Acquires a new anonymous page of the given size.
    pub fn acquire_page(&self, size: usize) -> Result<*mut u8, String> {
        unsafe {
            let mut flags = libc::MAP_PRIVATE | libc::MAP_ANONYMOUS;
            if size > 4096 {
                flags |= libc::MAP_HUGETLB;
            }

            let addr = libc::mmap(
                ptr::null_mut(),
                size,
                libc::PROT_READ | libc::PROT_WRITE,
                flags,
                -1,
                0,
            );

            if addr == libc::MAP_FAILED {
                return Err(format!(
                    "mmap failed for size {}: {}",
                    size,
                    std::io::Error::last_os_error()
                ));
            }

            Ok(addr as *mut u8)
        }
    }

    /// Recycles or frees a page if it wasn't moved.
    pub fn release_page(&self, ptr: *mut u8, size: usize) {
        if !ptr.is_null() {
            unsafe {
                libc::munmap(ptr as *mut c_void, size);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_allocation() {
        let pool = PagePool::new();
        let page = pool.acquire_page(4096).expect("Failed to acquire page");
        assert!(!page.is_null());
        assert_eq!(page as usize % 4096, 0, "Page must be 4096-aligned");

        // Write to page to ensure it's backed by memory
        unsafe {
            ptr::write_volatile(page, 0xAA);
            assert_eq!(ptr::read_volatile(page), 0xAA);
        }

        pool.release_page(page, 4096);
    }
}
