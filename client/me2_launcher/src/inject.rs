#[cfg(not(all(target_arch = "x86", target_os = "windows")))]
compile_error!("This program only supports x86 Windows");

use core::ffi::c_void;
use std::ffi::CString;
use std::path::Path;
use windows::core::{PCSTR, PSTR};
use windows::Win32::Foundation::{CloseHandle, GetLastError, FARPROC, HANDLE, WAIT_TIMEOUT};
use windows::Win32::System::Diagnostics::Debug::WriteProcessMemory;
use windows::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};
use windows::Win32::System::Memory::{
    VirtualAllocEx, VirtualFreeEx, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_READWRITE,
};
use windows::Win32::System::Threading::{
    CreateProcessA, CreateRemoteThread, ResumeThread, TerminateProcess, WaitForSingleObject,
    CREATE_SUSPENDED, INFINITE, LPTHREAD_START_ROUTINE, PROCESS_INFORMATION, STARTUPINFOA,
};

#[derive(Debug)]
pub enum InjectionError {
    CStringFailed(String),
    CreateProcessFailed(String),
    VirtualAllocExFailed(String),
    WriteProcessMemoryFailed(String),
    GetModuleHandleFailed(String),
    GetProcAddressFailed(String),
    CreateRemoteThreadFailed(String),
}

impl std::fmt::Display for InjectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InjectionError::CStringFailed(msg) => write!(f, "CString failed: {}", msg),
            InjectionError::CreateProcessFailed(msg) => write!(f, "CreateProcessA failed: {}", msg),
            InjectionError::VirtualAllocExFailed(msg) => {
                write!(f, "VirtualAllocEx failed: {}", msg)
            }
            InjectionError::WriteProcessMemoryFailed(msg) => {
                write!(f, "WriteProcessMemory failed: {}", msg)
            }
            InjectionError::GetModuleHandleFailed(msg) => {
                write!(f, "GetModuleHandleA failed: {}", msg)
            }
            InjectionError::GetProcAddressFailed(msg) => {
                write!(f, "GetProcAddress failed: {}", msg)
            }
            InjectionError::CreateRemoteThreadFailed(msg) => {
                write!(f, "CreateRemoteThread failed: {}", msg)
            }
        }
    }
}

impl std::error::Error for InjectionError {}

pub fn start_and_inject_dll(
    exe_path: impl AsRef<Path>,
    dll_path: impl AsRef<Path>,
    args: &[String],
) -> Result<InjectedProcess, InjectionError> {
    let mut command_line = format!(
        "\"{}\" {}",
        exe_path.as_ref().display(),
        args.iter()
            .map(|arg| format!("\"{}\"", arg.replace("\"", "\\\"")))
            .collect::<Vec<String>>()
            .join(" ")
    );

    let mut command_line_cstring =
        CString::new(command_line).map_err(|e| InjectionError::CStringFailed(e.to_string()))?;

    let command_line_pstr = PSTR {
        0: command_line_cstring.as_ptr() as *mut u8,
    };

    let startup_info = STARTUPINFOA::default();
    let mut process_info = PROCESS_INFORMATION::default();

    // Create the process in a suspended state
    let res = unsafe {
        CreateProcessA(
            None,              // lpApplicationName
            command_line_pstr, // lpCommandLine
            None,              // lpProcessAttributes
            None,              // lpThreadAttributes
            false,             // bInheritHandles
            CREATE_SUSPENDED,  // dwCreationFlags
            None,              // lpEnvironment
            None,              // lpCurrentDirectory
            &startup_info,     // lpStartupInfo
            &mut process_info, // lpProcessInformation
        )
    };

    if res.is_err() {
        return Err(InjectionError::CreateProcessFailed(format!(
            "Error code: {:?}",
            unsafe { GetLastError() }
        )));
    }

    let process_handle = process_info.hProcess;

    // Allocate memory in the target process for the DLL path
    let dll_path_string = CString::new(dll_path.as_ref().to_string_lossy().to_string())
        .map_err(|e| InjectionError::CStringFailed(e.to_string()))?;
    let dll_path_len = dll_path_string.as_bytes().len();
    let dll_path_void_ptr = dll_path_string.as_ptr() as *mut c_void;

    let remote_memory = unsafe {
        VirtualAllocEx(
            process_handle,
            None,
            dll_path_len + 1,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE,
        )
    };

    if remote_memory.is_null() {
        let error = unsafe { GetLastError() };
        return Err(InjectionError::VirtualAllocExFailed(format!(
            "Error code: {:?}",
            error
        )));
    }

    // Write the DLL path to the allocated memory
    let write_res = unsafe {
        WriteProcessMemory(
            process_handle,
            remote_memory,
            dll_path_void_ptr,
            dll_path_len + 1,
            None,
        )
    };

    if write_res.is_err() {
        return Err(InjectionError::WriteProcessMemoryFailed(format!(
            "Error code: {:?}",
            unsafe { GetLastError() }
        )));
    }

    // Get the address of LoadLibraryA in kernel32.dll
    let kernel32_dll_cstring =
        CString::new("kernel32.dll").map_err(|e| InjectionError::CStringFailed(e.to_string()))?;
    let kernel32_dll_pcstr = PCSTR {
        0: kernel32_dll_cstring.as_ptr() as *const u8,
    };

    let kernel32_res = unsafe { GetModuleHandleA(kernel32_dll_pcstr) };

    let kernel32_handle = match kernel32_res {
        Ok(handle) => handle,
        Err(_) => {
            return Err(InjectionError::GetModuleHandleFailed(format!(
                "Error code: {:?}",
                unsafe { GetLastError() }
            )));
        }
    };

    // Get the address of LoadLibraryA
    let load_library_a_cstring =
        CString::new("LoadLibraryA").map_err(|e| InjectionError::CStringFailed(e.to_string()))?;
    let load_library_a_pcstr = PCSTR {
        0: load_library_a_cstring.as_ptr() as *const u8,
    };
    let load_library_a_addr_res = unsafe { GetProcAddress(kernel32_handle, load_library_a_pcstr) };

    let load_library_a_addr = match load_library_a_addr_res {
        FARPROC::Some(addr) => addr,
        FARPROC::None => {
            return Err(InjectionError::GetProcAddressFailed(format!(
                "Error code: {:?}",
                unsafe { GetLastError() }
            )));
        }
    };

    // Create a remote thread that calls LoadLibraryA with the DLL path
    let thread_handle = unsafe {
        let lpthread_start_routine =
            LPTHREAD_START_ROUTINE::Some(std::mem::transmute(load_library_a_addr));
        CreateRemoteThread(
            process_handle,
            None,
            0,
            lpthread_start_routine,
            Some(remote_memory),
            0,
            None,
        )
    };

    let thread_handle = match thread_handle {
        Ok(handle) => handle,
        Err(_) => {
            return Err(InjectionError::CreateRemoteThreadFailed(format!(
                "Error code: {:?}",
                unsafe { GetLastError() }
            )));
        }
    };

    // Wait for the remote thread to finish
    let _wait_res = unsafe { WaitForSingleObject(thread_handle, INFINITE) };

    // Clean up
    unsafe {
        let _free_res = VirtualFreeEx(process_handle, remote_memory, 0, MEM_RELEASE);
        let _close_res = CloseHandle(thread_handle);
    }

    // Resume the main thread of the process
    unsafe {
        let _resume_res = ResumeThread(process_info.hThread);
    }

    // Create the InjectedProcess struct to return
    Ok(InjectedProcess {
        process_handle: process_info.hProcess,
        thread_handle,
    })
}

// Define a struct to hold the process and thread handles
#[derive(Debug)]
pub struct InjectedProcess {
    process_handle: HANDLE,
    thread_handle: HANDLE,
}

impl InjectedProcess {
    /// Checks if the injected process is still running.
    pub fn is_running(&self) -> bool {
        unsafe {
            let wait_result = WaitForSingleObject(self.process_handle, 0);
            wait_result == WAIT_TIMEOUT
        }
    }

    /// Kills the injected process.
    pub fn kill(&self) -> Result<(), InjectionError> {
        unsafe {
            if TerminateProcess(self.process_handle, 0).is_err() {
                Err(InjectionError::CreateProcessFailed(format!(
                    "Failed to terminate process: {:?}",
                    GetLastError()
                )))
            } else {
                Ok(())
            }
        }
    }
}

impl Drop for InjectedProcess {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.process_handle);
            CloseHandle(self.thread_handle);
        }
    }
}
