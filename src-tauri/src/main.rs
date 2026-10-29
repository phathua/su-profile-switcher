// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    #[cfg(windows)]
    unsafe {
        extern "system" {
            fn AttachConsole(dwProcessId: u32) -> i32;
            fn SetStdHandle(nStdHandle: u32, hHandle: isize) -> i32;
            fn CreateFileW(
                lpFileName: *const u16,
                dwDesiredAccess: u32,
                dwShareMode: u32,
                lpSecurityAttributes: *const std::ffi::c_void,
                dwCreationDisposition: u32,
                dwFlagsAndAttributes: u32,
                hTemplateFile: isize,
            ) -> isize;
        }
        const ATTACH_PARENT_PROCESS: u32 = 0xFFFFFFFF;
        if AttachConsole(ATTACH_PARENT_PROCESS) != 0 {
            let conout: Vec<u16> = "CONOUT$\0".encode_utf16().collect();
            let handle = CreateFileW(
                conout.as_ptr(),
                0x40000000 | 0x80000000,
                1 | 2,
                std::ptr::null(),
                3,
                0,
                0,
            );
            if handle != -1 {
                SetStdHandle(0xFFFFFFF5, handle); // STD_OUTPUT_HANDLE
                SetStdHandle(0xFFFFFFF4, handle); // STD_ERROR_HANDLE
            }
        }
    }

    su_profile_switcher_lib::run()
}
