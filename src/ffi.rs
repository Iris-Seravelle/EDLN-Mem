use std::os::raw::{c_int, c_void};

/*
#[repr(C)]
pub struct EdlnEventPacket {
    pub fault_addr: usize,
    pub fault_tid: i32,
    pub timestamp_ns: u64,
    pub flags: u8,
}
*/

unsafe extern "C" {
    pub fn resolve_fault_copy(
        uffd: c_int,
        fault_addr: u64,
        len: u64,
        source_page: *mut c_void,
    ) -> c_int;
    pub fn resolve_fault_move(
        uffd: c_int,
        fault_addr: u64,
        len: u64,
        source_page: *mut c_void,
    ) -> c_int;
    pub fn resolve_fault_wp(uffd: c_int, fault_addr: u64, len: u64, protect: c_int) -> c_int;
    pub fn resolve_fault_continue(uffd: c_int, fault_addr: u64, len: u64) -> c_int;
}
