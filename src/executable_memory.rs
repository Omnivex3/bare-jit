use std::{io, ptr, slice};

pub struct ExecutableMemory {
    ptr: *mut u8,
    len: usize,
}

impl ExecutableMemory {
    pub fn new(code: &[u8]) -> io::Result<Self> {
        // This implementation intentionally targets Linux x86-64, like the original.
        let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
        if page_size <= 0 {
            return Err(io::Error::last_os_error());
        }
        let page_size = page_size as usize;
        let allocation_len = code.len().max(1).div_ceil(page_size) * page_size;
        let ptr = unsafe {
            libc::mmap(
                ptr::null_mut(),
                allocation_len,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
                -1,
                0,
            )
        };
        if ptr == libc::MAP_FAILED {
            return Err(io::Error::last_os_error());
        }
        unsafe { ptr::copy_nonoverlapping(code.as_ptr(), ptr.cast(), code.len()) };
        if unsafe { libc::mprotect(ptr, allocation_len, libc::PROT_READ | libc::PROT_EXEC) } != 0 {
            let error = io::Error::last_os_error();
            unsafe {
                libc::munmap(ptr, allocation_len);
            }
            return Err(error);
        }
        Ok(Self {
            ptr: ptr.cast(),
            len: allocation_len,
        })
    }

    pub unsafe fn call(&self, argument: i64) -> i64 {
        type Function = extern "C" fn(i64) -> i64;
        let function: Function = std::mem::transmute(self.ptr);
        function(argument)
    }

    #[allow(dead_code)]
    pub fn bytes(&self, length: usize) -> &[u8] {
        assert!(length <= self.len);
        unsafe { slice::from_raw_parts(self.ptr, length) }
    }
}

impl Drop for ExecutableMemory {
    fn drop(&mut self) {
        unsafe {
            libc::munmap(self.ptr.cast(), self.len);
        }
    }
}

// mmap returns process-valid pointers and this wrapper never moves the allocation.
unsafe impl Send for ExecutableMemory {}
