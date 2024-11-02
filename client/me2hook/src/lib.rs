#[cfg(not(all(target_arch = "x86", target_os = "windows")))]
compile_error!("This program only supports x86 Windows");

mod console;
mod hooks;
use console::alloc_console;

use core::ffi::c_void;
use std::path::{Path, PathBuf};
use windows::Win32::Foundation::*;
use windows::Win32::System::LibraryLoader::*;
use windows::Win32::System::SystemServices::*;

use config::{Config, File};

#[derive(Debug, Clone)]
struct Settings {
    ip: String,
    session: String,
}

static mut SETTINGS: Settings = Settings {
    ip: String::new(),
    session: String::new(),
};

fn get_settings() -> Settings {
    unsafe { SETTINGS.clone() }
}

type DWORD = u32;
type LPVOID = *mut c_void;

fn sanitize_cr_and_lf(data: &str) -> String {
    // replace \r and \n with \\r and \\n
    data.chars()
        .map(|c| match c {
            '\r' => "\\r".to_string(),
            '\n' => "\\n".to_string(),
            _ => c.to_string(),
        })
        .collect::<String>()
}

fn handle_send(buffer: &[u8]) -> Option<Vec<u8>> {
    if let Ok(str_data) = String::from_utf8(buffer.to_vec()) {
        println!("[Hook] send() called, sending {} bytes", buffer.len());
        println!("[Hook] Buffer data: {}", sanitize_cr_and_lf(&str_data));

        let settings = get_settings();

        if str_data == "JMUS_AUTH\r\r" {
            let new_data = format!("JMUS_AUTH\r{}\r", settings.session);
            println!("[Hook] Modified data: {}", sanitize_cr_and_lf(&new_data));
            return Some(new_data.as_bytes().to_vec());
        } else {
            println!("[Hook] Unmodified data: {}", sanitize_cr_and_lf(&str_data));
        }
    }

    None
}

fn handle_connect(addr: u32) -> Option<u32> {
    // Convert redirect IP to network byte order
    println!(
        "[Hook] Attempting to connect to IP: {}.{}.{}.{}",
        addr & 0xFF,
        (addr >> 8) & 0xFF,
        (addr >> 16) & 0xFF,
        (addr >> 24) & 0xFF,
    );

    let settings = get_settings();
    if !settings.ip.is_empty() {
        let ip_parts: Vec<u8> = settings
            .ip
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();

        if ip_parts.len() == 4 {
            println!("[Hook] Redirecting to {}", settings.ip);
            return Some(u32::from_le_bytes([
                ip_parts[0],
                ip_parts[1],
                ip_parts[2],
                ip_parts[3],
            ]));
        }
    }

    None
}

fn parse_settings(filename: impl AsRef<Path>) -> (String, String) {
    let filename = filename.as_ref();
    let filename_str = filename.to_str().unwrap();
    let settings = Config::builder()
        .add_source(File::with_name(filename_str))
        .build()
        .unwrap_or_else(|_| {
            // Create default launch_config.ini if it doesn't exist
            std::fs::write(filename, "[Settings]\nip=127.0.0.1\nsession=MyName")
                .expect("Failed to create launch_config.ini");

            // Reload the settings after creating the file
            Config::builder()
                .add_source(File::with_name(filename_str))
                .build()
                .expect("Failed to load launch_config.ini after creation")
        });

    let ip = settings
        .get::<String>("Settings.ip")
        .unwrap_or_else(|_| "127.0.0.1".to_string());
    let session = settings
        .get::<String>("Settings.session")
        .unwrap_or_else(|_| "MyName".to_string());

    (ip, session)
}

#[no_mangle]
pub extern "system" fn DllMain(
    hinst_dll: HINSTANCE,
    fdw_reason: DWORD,
    _lpv_reserved: LPVOID,
) -> BOOL {
    unsafe fn set_settings(settings: Settings) {
        SETTINGS = settings;
    }

    if fdw_reason == DLL_PROCESS_ATTACH {
        if cfg!(feature = "console") {
            alloc_console();
        }
        let mut module_path_buf = [0u8; MAX_PATH as usize];
        unsafe { GetModuleFileNameA(hinst_dll, &mut module_path_buf) };
        let module_path_str = String::from_utf8(module_path_buf.to_vec())
            .expect("[Hook] Failed to convert module path to string");
        let module_path = PathBuf::from(&module_path_str);

        let dll_container_path = module_path
            .parent()
            .expect("[Hook] Failed to get module path");
        let config_path = dll_container_path.join("launch_config.ini");

        println!("[Hook] Config path: {:?}", config_path);

        let (ip, session) = parse_settings(&config_path);
        unsafe { set_settings(Settings { ip, session }) };

        if hooks::setup_hooks() {
            println!("[Hook] Hooks installed successfully");
        } else {
            println!("[Hook] Failed to install hooks");
        }
    }

    TRUE
}
