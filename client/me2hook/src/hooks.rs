#[cfg(not(all(target_arch = "x86", target_os = "windows")))]
compile_error!("This program only supports x86 Windows");

use core::ffi::c_void;
use std::ffi::CString;
use std::ptr;
use windows::core::PCSTR;
use windows::Win32::Foundation::*;
use windows::Win32::Networking::WinSock::*;
use windows::Win32::System::LibraryLoader::*;
use windows::Win32::System::Memory::*;

use crate::{handle_connect, handle_send};

static mut ORIGINAL_CONNECT: usize = 0;
static mut ORIGINAL_SEND: usize = 0;

// Original function signatures
type ConnectFn = unsafe extern "system" fn(SOCKET, *const SOCKADDR, i32) -> i32;
type SendFn = unsafe extern "system" fn(SOCKET, *const u8, i32, i32) -> i32;

#[derive(Debug)]
struct Hook {
    original_bytes: [u8; 5],
    target_fn: usize,
}

impl Hook {
    fn new(target_fn: usize) -> Self {
        Self {
            original_bytes: [0; 5],
            target_fn,
        }
    }

    fn install(&mut self, hook_fn: usize) -> bool {
        let mut old_protect = PAGE_PROTECTION_FLAGS(0);

        // Allow the target function to be overwritten
        if let Err(why) = unsafe {
            VirtualProtect(
                self.target_fn as *mut c_void,
                5,
                PAGE_EXECUTE_READWRITE,
                &mut old_protect,
            )
        } {
            println!(
                "[Hook] Failed to change page protection for target function: {:?}",
                why
            );
            return false;
        }

        // Save original bytes
        unsafe {
            ptr::copy_nonoverlapping(
                self.target_fn as *const u8,
                self.original_bytes.as_mut_ptr(),
                5,
            );
        }

        // Calculate relative jump
        let rel_addr = (hook_fn as isize - self.target_fn as isize - 5) as u32;
        let mut jump = vec![0xE9_u8];
        jump.extend_from_slice(&rel_addr.to_le_bytes());

        // Write jump instruction
        unsafe {
            ptr::copy_nonoverlapping(jump.as_ptr(), self.target_fn as *mut u8, 5);
            let _ = VirtualProtect(self.target_fn as *mut _, 5, old_protect, &mut old_protect);
        }
        true
    }

    fn remove(&self) -> bool {
        let mut old_protect = PAGE_PROTECTION_FLAGS(0);

        if unsafe {
            VirtualProtect(
                self.target_fn as *mut _,
                5,
                PAGE_EXECUTE_READWRITE,
                &mut old_protect,
            )
        }
        .is_err()
        {
            return false;
        }

        // Restore original bytes
        unsafe {
            ptr::copy_nonoverlapping(self.original_bytes.as_ptr(), self.target_fn as *mut u8, 5);
            let _ = VirtualProtect(self.target_fn as *mut _, 5, old_protect, &mut old_protect);
        }
        true
    }
}

static mut CONNECT_HOOK: Hook = Hook {
    original_bytes: [0; 5],
    target_fn: 0,
};

static mut SEND_HOOK: Hook = Hook {
    original_bytes: [0; 5],
    target_fn: 0,
};

unsafe extern "system" fn hooked_connect(
    socket: SOCKET,
    name: *const SOCKADDR,
    namelen: i32,
) -> i32 {
    //println!("[Hook] hooked_connect called");

    if !name.is_null() && (*name).sa_family == AF_INET {
        // println!("[Hook] name is not null and sa_family is AF_INET");
        let addr = name as *mut SOCKADDR_IN;

        let modified_addr = handle_connect((*addr).sin_addr.S_un.S_addr);

        if let Some(new_addr) = modified_addr {
            (*addr).sin_addr.S_un.S_addr = new_addr;
        }
    }

    // Temporarily remove hook to avoid infinite loop
    CONNECT_HOOK.remove();

    let original_fn = std::mem::transmute::<_, ConnectFn>(ORIGINAL_CONNECT);
    let result = original_fn(socket, name, namelen);

    CONNECT_HOOK.install(hooked_connect as usize);
    result
}

unsafe extern "system" fn hooked_send(socket: SOCKET, buf: *const u8, len: i32, flags: i32) -> i32 {
    // println!("[Hook] hooked_send called");
    let data = std::slice::from_raw_parts(buf, len as usize);
    // println!("[Hook] send() called, sending {} bytes", len);
    // println!("[Hook] Buffer data: {}", str_data);

    let modified_data = handle_send(data);

    // Temporarily remove hook to avoid infinite loop
    SEND_HOOK.remove();

    let original_fn = std::mem::transmute::<_, SendFn>(ORIGINAL_SEND);
    let result = original_fn(
        socket,
        match &modified_data {
            Some(data) => data.as_ptr(),
            None => buf,
        },
        match &modified_data {
            Some(data) => data.len() as i32,
            None => len,
        },
        flags,
    );

    SEND_HOOK.install(hooked_send as usize);
    result
}

fn get_module_handle(module_name: &str) -> Result<HMODULE, ()> {
    let module_name_cstring = CString::new(module_name).map_err(|_| ())?;
    let module_name_pcstr = PCSTR(module_name_cstring.as_ptr() as *const u8);
    let module_handle = unsafe { GetModuleHandleA(module_name_pcstr) };
    module_handle.map_err(|_| ())
}

fn get_proc_address(module_handle: HMODULE, proc_name: &str) -> Result<usize, ()> {
    let proc_name_cstring = CString::new(proc_name).map_err(|_| ())?;
    let proc_name_pcstr = PCSTR(proc_name_cstring.as_ptr() as *const u8);
    let function = unsafe { GetProcAddress(module_handle, proc_name_pcstr).ok_or(())? };
    let addr = unsafe { std::mem::transmute::<_, usize>(function) };

    if addr == 0 {
        return Err(());
    }

    Ok(addr)
}

fn load_library(module_name: &str) -> Result<HMODULE, ()> {
    let module_name_cstring = CString::new(module_name).map_err(|_| ())?;
    unsafe { LoadLibraryA(PCSTR(module_name_cstring.as_ptr() as *const u8)).map_err(|_| ()) }
}

pub fn setup_hooks() -> bool {
    unsafe {
        // Need to make sure this is loaded first. When the program starts, this may or may not be loaded yet.
        if load_library("ws2_32.dll").is_err() {
            println!("[Hook] Failed to load ws2_32.dll");
            return false;
        }

        let Ok(ws2_32) = get_module_handle("ws2_32.dll") else {
            println!("[Hook] Failed to get module handle for ws2_32.dll");
            return false;
        };

        ORIGINAL_CONNECT = match get_proc_address(ws2_32, "connect") {
            Ok(addr) => addr,
            Err(_) => {
                println!("[Hook] Failed to get proc address for connect");
                return false;
            }
        };

        ORIGINAL_SEND = match get_proc_address(ws2_32, "send") {
            Ok(addr) => addr,
            Err(_) => {
                println!("[Hook] Failed to get proc address for send");
                return false;
            }
        };

        // println!("[Hook] ORIGINAL_CONNECT: {:08X}", ORIGINAL_CONNECT);
        // println!("[Hook] ORIGINAL_SEND: {:08X}", ORIGINAL_SEND);

        if ORIGINAL_CONNECT == 0 || ORIGINAL_SEND == 0 {
            println!("[Hook] ORIGINAL_CONNECT or ORIGINAL_SEND is 0");
            return false;
        }

        CONNECT_HOOK = Hook::new(ORIGINAL_CONNECT);
        SEND_HOOK = Hook::new(ORIGINAL_SEND);

        CONNECT_HOOK.install(hooked_connect as usize) && SEND_HOOK.install(hooked_send as usize)
    }
}
