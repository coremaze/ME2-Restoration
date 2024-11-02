use windows::core::PCSTR;
use windows::Win32::Foundation::*;
use windows::Win32::Storage::FileSystem::CreateFileA;
use windows::Win32::Storage::FileSystem::FILE_ATTRIBUTE_NORMAL;
use windows::Win32::Storage::FileSystem::FILE_SHARE_WRITE;
use windows::Win32::Storage::FileSystem::OPEN_EXISTING;
use windows::Win32::System::Console::{
    AllocConsole, SetStdHandle, STD_ERROR_HANDLE, STD_OUTPUT_HANDLE,
};

pub fn alloc_console() -> bool {
    unsafe {
        if AllocConsole().is_err() {
            return false;
        }

        // Get console output handle
        let stdout = CreateFileA(
            PCSTR(b"CONOUT$\0".as_ptr()),
            (GENERIC_READ | GENERIC_WRITE).0,
            FILE_SHARE_WRITE,
            None,
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            None,
        );

        let stdout = match stdout {
            Ok(stdout) => stdout,
            Err(_) => return false,
        };

        let stderr = CreateFileA(
            PCSTR(b"CONOUT$\0".as_ptr()),
            (GENERIC_READ | GENERIC_WRITE).0,
            FILE_SHARE_WRITE,
            None,
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            None,
        );

        let stderr = match stderr {
            Ok(stderr) => stderr,
            Err(_) => return false,
        };

        // Set as stdout handle
        if SetStdHandle(STD_OUTPUT_HANDLE, stdout).is_err() {
            return false;
        }

        if SetStdHandle(STD_ERROR_HANDLE, stderr).is_err() {
            return false;
        }

        true
    }
}
