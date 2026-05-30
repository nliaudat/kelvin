//! Memory Access Test — Verifies that protected memory causes SIGSEGV/STATUS_ACCESS_VIOLATION.
//!
//! This binary is spawned by the test runner in `main.rs`. It:
//! 1. Allocates page-aligned memory via OS allocation
//! 2. Writes a known secret pattern
//! 3. Protects the page (PROT_NONE / PAGE_NOACCESS)
//! 4. Attempts to read from the protected page
//!
//! If memory protection works, the process crashes with SIGSEGV/STATUS_ACCESS_VIOLATION.
//! If memory protection is not available, it exits with code 3.

fn main() {
    let page_size: usize = 4096;

    #[cfg(target_os = "windows")]
    let ptr = allocate_windows(page_size);
    #[cfg(not(target_os = "windows"))]
    let ptr = allocate_unix(page_size);

    if ptr.is_null() {
        eprintln!("MEMORY_ALLOCATION_FAILED");
        std::process::exit(1);
    }

    unsafe {
        std::ptr::write_bytes(ptr, 0xAB, page_size);
    }

    let verify = unsafe { std::ptr::read(ptr) };
    if verify != 0xAB {
        eprintln!("MEMORY_WRITE_VERIFICATION_FAILED");
        std::process::exit(2);
    }

    #[cfg(target_os = "windows")]
    protect_windows(ptr, page_size);
    #[cfg(not(target_os = "windows"))]
    protect_unix(ptr, page_size);

    let _attempt: u8 = unsafe { std::ptr::read_volatile(ptr) };

    eprintln!("MEMORY_PROTECTION_INOPERATIVE");
    std::process::exit(0);
}

#[cfg(target_os = "windows")]
fn allocate_windows(size: usize) -> *mut u8 {
    use std::ptr;
    let raw = unsafe {
        windows_sys::Win32::System::Memory::VirtualAlloc(
            ptr::null(),
            size,
            windows_sys::Win32::System::Memory::MEM_COMMIT
                | windows_sys::Win32::System::Memory::MEM_RESERVE,
            windows_sys::Win32::System::Memory::PAGE_READWRITE,
        )
    };
    raw as *mut u8
}

#[cfg(target_os = "windows")]
fn protect_windows(ptr: *mut u8, size: usize) {
    use windows_sys::Win32::System::Memory::{VirtualProtect, PAGE_NOACCESS};
    let mut old_protect = 0u32;
    let result = unsafe {
        VirtualProtect(ptr as *const std::ffi::c_void, size, PAGE_NOACCESS, &mut old_protect)
    };
    if result == 0 {
        eprintln!("MEMORY_PROTECTION_NOT_AVAILABLE");
        std::process::exit(3);
    }
}

#[cfg(not(target_os = "windows"))]
fn allocate_unix(size: usize) -> *mut u8 {
    let raw = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            size,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
            -1,
            0,
        )
    };
    if raw == libc::MAP_FAILED {
        std::ptr::null_mut()
    } else {
        raw as *mut u8
    }
}

#[cfg(not(target_os = "windows"))]
fn protect_unix(ptr: *mut u8, size: usize) {
    let result = unsafe { libc::mprotect(ptr as *mut std::ffi::c_void, size, libc::PROT_NONE) };
    if result != 0 {
        eprintln!("MEMORY_PROTECTION_NOT_AVAILABLE");
        std::process::exit(3);
    }
}
