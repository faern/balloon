use core::ffi;
use std::io;

#[cfg(unix)]
pub fn mlock(memory: &[u8]) -> io::Result<()> {
    match unsafe { libc::mlock(memory.as_ptr() as *const ffi::c_void, memory.len()) } {
        0 => Ok(()),
        -1 => Err(io::Error::last_os_error()),
        i => panic!("Unexpected return value from mlock: {}", i),
    }
}

#[cfg(windows)]
pub fn mlock(memory: &[u8]) -> io::Result<()> {
    // FIXME: This is still completely untested!
    if 0 == winapi::um::winbase::SetProcessWorkingSetSize(
        winapi::um::processthreadsapi::GetCurrentProcess(),
        memory.len(),
        memory.len(),
    ) {
        return Err(io::Error::last_os_error());
    }
    if 0 == winapi::um::memoryapi::VirtualLock(
        memory.as_ptr() as *mut winapi::ctypes::c_void,
        memory.len(),
    ) {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}
