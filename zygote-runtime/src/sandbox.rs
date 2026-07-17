use std::io;

const MEM_COMMIT: u32 = 0x00001000;
const MEM_RESERVE: u32 = 0x00002000;
const MEM_RELEASE: u32 = 0x00008000;
const PAGE_EXECUTE_READWRITE: u32 = 0x40;

extern "system" {
    fn VirtualAlloc(
        lpAddress: *mut std::ffi::c_void,
        dwSize: usize,
        flAllocationType: u32,
        flProtect: u32,
    ) -> *mut std::ffi::c_void;
    fn VirtualFree(
        lpAddress: *mut std::ffi::c_void,
        dwSize: usize,
        dwFreeType: u32,
    ) -> i32;
}

pub fn alloc_exec(size: usize) -> io::Result<*mut u8> {
    unsafe {
        let ptr = VirtualAlloc(
            std::ptr::null_mut(),
            size,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_EXECUTE_READWRITE,
        );
        if ptr.is_null() {
            return Err(io::Error::last_os_error());
        }
        Ok(ptr as *mut u8)
    }
}

pub fn free_exec(ptr: *mut u8, size: usize) -> io::Result<()> {
    unsafe {
        if VirtualFree(ptr as *mut std::ffi::c_void, size, MEM_RELEASE) == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }
}

pub struct ExecCode {
    ptr: *mut u8,
    size: usize,
}

impl ExecCode {
    pub fn new(code: &[u8]) -> io::Result<Self> {
        let size = code.len();
        let ptr = alloc_exec(size)?;
        unsafe {
            std::ptr::copy_nonoverlapping(code.as_ptr(), ptr, size);
        }
        Ok(Self { ptr, size })
    }

    pub fn as_fn_ptr(&self) -> unsafe extern "C" fn(*mut u8, *mut u8, *mut u8, *mut u8) {
        unsafe { std::mem::transmute(self.ptr) }
    }
}

impl Drop for ExecCode {
    fn drop(&mut self) {
        let _ = free_exec(self.ptr, self.size);
    }
}
