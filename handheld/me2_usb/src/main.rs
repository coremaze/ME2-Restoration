mod me2_usb;

use std::io::Write;

use clap::{Parser, Subcommand};
use clap_num::maybe_hex;
use me2_usb::Me2Device;

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    command: Me2Command,
}

#[derive(Subcommand)]
enum Me2Command {
    ReadFlash {
        file: String,
    },
    WriteFlash {
        file: String,
    },
    ReadPoints,
    SetPoints {
        points: u32,
    },
    ReadGems,
    SetGems {
        gems: u32,
    },
    DumpMemory {
        #[clap(value_parser=maybe_hex::<u32>)]
        address: u32,
        #[clap(value_parser=maybe_hex::<u32>)]
        length: u32,
    },
    WatchControls,
}

fn main() {
    let args = Args::parse();
    match args.command {
        Me2Command::ReadFlash { file } => read_flash(&file),
        Me2Command::WriteFlash { file } => write_flash(&file),
        Me2Command::ReadPoints => read_points(),
        Me2Command::SetPoints { points } => set_points(points),
        Me2Command::ReadGems => read_gems(),
        Me2Command::SetGems { gems } => set_gems(gems),
        Me2Command::DumpMemory { address, length } => dump_memory(address, length),
        Me2Command::WatchControls => watch_controls(),
    }
}

fn read_flash(file: &str) {
    let mut device = match Me2Device::open() {
        Ok(device) => device,
        Err(e) => {
            eprintln!("Failed to open device: {}", e);
            return;
        }
    };

    let mut data = Vec::new();
    for sector in 0..0x400 {
        let Ok(sector_data) = device.read_flash_sector(sector) else {
            eprintln!("Failed to read sector {sector:04X}");
            continue;
        };
        // me2_usb::hexdump(&data);
        data.extend(sector_data);
    }
    if let Err(e) = std::fs::write(file, data) {
        eprintln!("Failed to write to file: {}", e);
    }
}

fn write_flash(file: &str) {
    let mut device = match Me2Device::open() {
        Ok(device) => device,
        Err(e) => {
            eprintln!("Failed to open device: {}", e);
            return;
        }
    };

    let Ok(data) = std::fs::read(file) else {
        eprintln!("Failed to read file: {}", file);
        return;
    };

    // check if the file is the correct size
    let expected_size = 0x400 /* sectors */ * 0x1000 /* bytes per sector */;
    if data.len() != expected_size {
        eprintln!(
            "File is not the correct size (should be {}, got {})",
            expected_size,
            data.len()
        );
        return;
    }

    for (sector, chunk) in data.chunks(0x1000).enumerate().skip(0x11) {
        if let Err(e) = device.erase_flash_sector(sector as u32) {
            eprintln!("Failed to erase sector {sector:04X}: {}", e);
            return;
        }
        if let Err(e) = device.program_flash_sector(sector as u32, chunk) {
            eprintln!("Failed to write sector {sector:04X}: {}", e);
            return;
        }
        println!("Wrote sector {sector:04X}");
    }
}

fn read_points() {
    let mut device = match Me2Device::open() {
        Ok(device) => device,
        Err(e) => {
            eprintln!("Failed to open device: {}", e);
            return;
        }
    };

    let Ok(points_sector) = device.read_flash_sector(0x3E) else {
        eprintln!("Failed to read points sector");
        return;
    };

    let Some(points_bytes) = points_sector.get(4..8) else {
        eprintln!("Failed to get points bytes");
        return;
    };

    let Ok(points_bytes) = points_bytes.try_into() else {
        eprintln!("Failed to convert points bytes to [u8; 4]");
        return;
    };

    let points = u32::from_le_bytes(points_bytes);
    println!("Points: {}", points);
}

fn set_points(points: u32) {
    if points > 99999999 {
        eprintln!(
            "Points must be less than 99999999. The game will crash upon viewing the points if you set more than that."
        );
        return;
    }

    let mut device = match Me2Device::open() {
        Ok(device) => device,
        Err(e) => {
            eprintln!("Failed to open device: {}", e);
            return;
        }
    };

    let Ok(mut points_sector) = device.read_flash_sector(0x3E) else {
        eprintln!("Failed to read points sector");
        return;
    };

    let points_bytes = points.to_le_bytes();

    // write the points bytes to the points sector buffer
    points_sector[4..8].copy_from_slice(&points_bytes);

    // Erase points sector
    if let Err(e) = device.erase_flash_sector(0x3E) {
        eprintln!("Failed to erase points sector: {}", e);
        return;
    }

    // Program points sector
    if let Err(e) = device.program_flash_sector(0x3E, &points_sector) {
        eprintln!("Failed to write points: {}", e);
    }
}

fn read_gems() {
    let mut device = match Me2Device::open() {
        Ok(device) => device,
        Err(e) => {
            eprintln!("Failed to open device: {}", e);
            return;
        }
    };

    let Ok(gems_sector) = device.read_flash_sector(0x1FE) else {
        eprintln!("Failed to read gems sector");
        return;
    };

    let Some(gems_bytes) = gems_sector.get(0x100..0x104) else {
        eprintln!("Failed to get gems bytes");
        return;
    };

    let Ok(gems_bytes) = gems_bytes.try_into() else {
        eprintln!("Failed to convert points bytes to [u8; 4]");
        return;
    };

    let gems = u32::from_le_bytes(gems_bytes);
    println!("Gems: {}", gems);
}

fn set_gems(gems: u32) {
    let mut device = match Me2Device::open() {
        Ok(device) => device,
        Err(e) => {
            eprintln!("Failed to open device: {}", e);
            return;
        }
    };

    let Ok(mut gems_sector) = device.read_flash_sector(0x1FE) else {
        eprintln!("Failed to read gems sector");
        return;
    };

    let gems_bytes = gems.to_le_bytes();
    gems_sector[0x100..0x104].copy_from_slice(&gems_bytes);

    if let Err(e) = device.erase_flash_sector(0x1FE) {
        eprintln!("Failed to erase gems sector: {}", e);
    }

    if let Err(e) = device.program_flash_sector(0x1FE, &gems_sector) {
        eprintln!("Failed to write gems: {}", e);
    }
}

fn dump_memory(address: u32, length: u32) {
    let mut device = match Me2Device::open() {
        Ok(device) => device,
        Err(e) => {
            eprintln!("Failed to open device: {}", e);
            return;
        }
    };

    match device.read_address(address, length as usize) {
        Ok(data) => me2_usb::hexdump_u16(&data, address as usize),
        Err(why) => eprintln!("Failed to read: {why:?}"),
    };
}

fn watch_controls() {
    use std::time::{Duration, Instant};

    let mut device = match Me2Device::open() {
        Ok(device) => device,
        Err(e) => {
            eprintln!("Failed to open device: {}", e);
            return;
        }
    };

    let mut poll_count = 0;
    let start_time = Instant::now();
    let mut last_line_length: usize = 0;

    loop {
        let ports = match device.read_address(0x7860, 0x20) {
            Ok(ports) => ports,
            Err(e) => {
                eprintln!("Failed to read ports: {}", e);
                continue;
            }
        };
        let Some(pa) = ports.get(0) else {
            eprintln!("Failed to get pa");
            continue;
        };
        let Some(pb) = ports.get(8) else {
            eprintln!("Failed to get pb");
            continue;
        };
        let Some(pc) = ports.get(16) else {
            eprintln!("Failed to get pc");
            continue;
        };
        let Some(pd) = ports.get(24) else {
            eprintln!("Failed to get pd");
            continue;
        };

        let mut buttons = Vec::<String>::new();
        if pa & 0x0010 != 0 {
            buttons.push("UP".to_string());
        }
        if pa & 0x0080 != 0 {
            buttons.push("RIGHT".to_string());
        }
        if pa & 0x0040 != 0 {
            buttons.push("LEFT".to_string());
        }
        if pa & 0x0020 != 0 {
            buttons.push("DOWN".to_string());
        }
        if pa & 0x0002 != 0 {
            buttons.push("A".to_string());
        }
        if pb & 0x0100 != 0 {
            buttons.push("B".to_string());
        }

        let elapsed = start_time.elapsed().as_secs_f64();
        let poll_rate = poll_count as f64 / elapsed;
        let new_message = format!("\rPoll rate: {poll_rate:.2}Hz | {}", buttons.join(" "));
        let spaces_needed = last_line_length.saturating_sub(new_message.len());
        let message_padding = " ".repeat(spaces_needed);
        print!("{new_message}{message_padding}");
        std::io::stdout().flush().ok();
        last_line_length = new_message.len();

        poll_count += 1;
    }
}
