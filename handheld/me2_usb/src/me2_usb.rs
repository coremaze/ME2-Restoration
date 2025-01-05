use std::time::Duration;

use rusb::{Context, Device, DeviceDescriptor, DeviceHandle, UsbContext};

const ME2_VENDOR_ID: u16 = 0x1b3f;
const ME2_PRODUCT_ID: u16 = 0x2002;

#[derive(Debug)]
pub struct Me2Device {
    context: Context,
    device: Device<Context>,
    handle: DeviceHandle<Context>,
    tag: u32,
}

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Me2Error {
    #[error("USB error: {0}")]
    UsbError(#[from] rusb::Error),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

impl Me2Device {
    pub fn open() -> Result<Self, Me2Error> {
        let mut context = Context::new().unwrap();
        let (device, device_desc, handle) =
            open_device(&mut context, ME2_VENDOR_ID, ME2_PRODUCT_ID)?
                .ok_or(Me2Error::UsbError(rusb::Error::NotFound))?;
        handle.set_auto_detach_kernel_driver(true).unwrap();
        handle.claim_interface(0).unwrap();
        Ok(Self {
            context,
            device,
            handle,
            tag: 0,
        })
    }

    pub fn read_flash_sector(&mut self, sector: u32) -> Result<Vec<u8>, Me2Error> {
        let block = ((sector & 0xFF00) >> 8) as u8;
        let sector = (sector & 0xFF) as u8;

        let tag1 = (self.tag & 0xFF) as u8;
        let tag2 = ((self.tag >> 8) & 0xFF) as u8;
        let tag3 = ((self.tag >> 16) & 0xFF) as u8;
        let tag4 = ((self.tag >> 24) & 0xFF) as u8;

        let write_buffer = [
            /* 0x00 */ 0x55, 0x53, 0x42, 0x43, // USBC Signature
            /* 0x04 */ tag1, tag2, tag3, tag4, // Tag
            /* 0x08 */ 0x00, 0x00, 0x00, // Data transfer length
            /* 0x0B */ 0x01, // Don't know what this does
            /* 0x0C */ 0x80, // Flags
            /* 0x0D */ 0x00, // LUN
            /* 0x0E */ 0x0a, // CDB Length
            /* 0x0F */ 0xFF, // SCSI Opcode
            /* 0x10 */ 0x12, // Custom SCSI Opcode
            /* 0x11 */ 0x00, 0x00, 0x00, 0x00, // Logical Block Address
            /* 0x15 */ 0x00, // Group
            /* 0x16 */ 0x00, 0x08, // Transfer length
            /* 0x18 */ block, // Block
            /* 0x19 */ sector, // Sector
            /* 0x1A */ 0x00, 0x00, 0x00, 0x00, 0x00, // Unused
        ];
        self.handle
            .write_bulk(2, &write_buffer, Duration::from_secs(1))?;
        self.tag += 1;

        let mut read_buffer = [0; 0x1000];
        let read_size = self
            .handle
            .read_bulk(129, &mut read_buffer, Duration::from_secs(1))?;
        // hexdump(&read_buffer[..read_size]);

        let mut result = Vec::new();
        result.extend(&read_buffer[..read_size]);

        // read status
        let _read_size = self
            .handle
            .read_bulk(129, &mut read_buffer, Duration::from_secs(1))?;
        // hexdump(&read_buffer[..read_size]);

        Ok(result)
    }

    pub fn program_flash_sector_unsafe(
        &mut self,
        sector: u32,
        data: &[u8],
    ) -> Result<(), Me2Error> {
        if data.len() != 0x1000 {
            return Err(Me2Error::UsbError(rusb::Error::Other));
        }
        let block = ((sector & 0xFF00) >> 8) as u8;
        let sector = (sector & 0xFF) as u8;

        let tag1 = (self.tag & 0xFF) as u8;
        let tag2 = ((self.tag >> 8) & 0xFF) as u8;
        let tag3 = ((self.tag >> 16) & 0xFF) as u8;
        let tag4 = ((self.tag >> 24) & 0xFF) as u8;

        let write_buffer = [
            /* 0x00 */ 0x55, 0x53, 0x42, 0x43, // USBC Signature
            /* 0x04 */ tag1, tag2, tag3, tag4, // Tag
            /* 0x08 */ 0x00, 0x00, 0x00, // Data transfer length
            /* 0x0B */ 0x01, // Don't know what this does
            /* 0x0C */ 0x80, // Flags
            /* 0x0D */ 0x00, // LUN
            /* 0x0E */ 0x0a, // CDB Length
            /* 0x0F */ 0xFF, // SCSI Opcode
            /* 0x10 */ 0x2A, // Custom SCSI Opcode
            /* 0x11 */ 0x00, 0x00, 0x00, 0x00, // Logical Block Address
            /* 0x15 */ 0x00, // Group
            /* 0x16 */ 0x00, 0x08, // Transfer length
            /* 0x18 */ block, // Block
            /* 0x19 */ sector, // Sector
            /* 0x1A */ 0x00, 0x00, 0x00, 0x00, 0x00, // Unused
        ];
        // println!("writing {:?}", write_buffer);
        self.handle
            .write_bulk(2, &write_buffer, Duration::from_secs(1))?;

        // println!("writing data {:?}", data);
        self.handle.write_bulk(2, data, Duration::from_secs(1))?;

        let mut read_buffer = [0; 0x1000];
        // read status
        // println!("reading status");
        let _read_size = self
            .handle
            .read_bulk(129, &mut read_buffer, Duration::from_secs(1))?;
        // println!("reading status done");
        // hexdump(&read_buffer[..read_size]);
        Ok(())
    }

    pub fn program_flash_sector(&mut self, sector: u32, data: &[u8]) -> Result<(), Me2Error> {
        if !is_safe_sector(sector) {
            println!("Prevented unsafe program of sector {}", sector);
            return Ok(());
        }
        self.program_flash_sector_unsafe(sector, data)?;
        Ok(())
    }

    pub fn erase_flash_sector_unsafe(&mut self, sector: u32) -> Result<(), Me2Error> {
        let block = ((sector & 0xFF00) >> 8) as u8;
        let sector = (sector & 0xFF) as u8;

        let tag1 = (self.tag & 0xFF) as u8;
        let tag2 = ((self.tag >> 8) & 0xFF) as u8;
        let tag3 = ((self.tag >> 16) & 0xFF) as u8;
        let tag4 = ((self.tag >> 24) & 0xFF) as u8;

        let write_buffer = [
            /* 0x00 */ 0x55, 0x53, 0x42, 0x43, // USBC Signature
            /* 0x04 */ tag1, tag2, tag3, tag4, // Tag
            /* 0x08 */ 0x00, 0x00, 0x00, // Data transfer length
            /* 0x0B */ 0x01, // Don't know what this does
            /* 0x0C */ 0x80, // Flags
            /* 0x0D */ 0x00, // LUN
            /* 0x0E */ 0x0a, // CDB Length
            /* 0x0F */ 0xFF, // SCSI Opcode
            /* 0x10 */ 0x20, // Flags
            /* 0x11 */ 0x00, 0x00, 0x00, 0x00, // Logical Block Address
            /* 0x15 */ 0x00, // Group
            /* 0x16 */ 0x00, 0x08, // Transfer length
            /* 0x18 */ block, // Block
            /* 0x19 */ sector, // Sector
            /* 0x1A */ 0x00, 0x00, 0x00, 0x00, 0x00, // Unused
        ];
        self.handle
            .write_bulk(2, &write_buffer, Duration::from_secs(1))?;

        let mut read_buffer = [0; 0x1000];
        // read status
        let _read_size = self
            .handle
            .read_bulk(129, &mut read_buffer, Duration::from_secs(1))?;
        // hexdump(&read_buffer[..read_size]);
        Ok(())
    }

    pub fn erase_flash_sector(&mut self, sector: u32) -> Result<(), Me2Error> {
        if !is_safe_sector(sector) {
            println!("Prevented unsafe erase of sector {}", sector);
            return Ok(());
        }
        self.erase_flash_sector_unsafe(sector)?;
        Ok(())
    }

    pub fn read_address_sector(&mut self, address: u32) -> Result<Vec<u16>, Me2Error> {
        let address = address * 2; // 16-bit words
        let address = 0x8000000 + address - 0x6_0000;
        let sector_l = ((address & 0xFF000) >> 12);
        let sector_h = ((address & 0xFF00000) >> 20);

        let sector = (sector_h << 8) | sector_l;
        let data_u8s = self.read_flash_sector(sector)?;
        let data_u16s: Vec<u16> = data_u8s
            .chunks(2)
            .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
            .collect();
        Ok(data_u16s)
    }

    pub fn read_address(&mut self, address: u32, length: usize) -> Result<Vec<u16>, Me2Error> {
        // Start at an arbitrary address
        // Increment address by the length of the result each time,
        // until the result is longer than what was asked.
        // Then, trim the start to be at the correct address, trim the end to be the correct length.
        let mut result = Vec::new();
        let mut current_address = (address / 0x800) * 0x800;
        let target_address = address + length as u32;
        while current_address < target_address {
            let page_data = self.read_address_sector(current_address)?;
            current_address += page_data.len() as u32;
            result.extend(page_data);
        }

        let start_index = address % 0x800;
        if start_index > 0 {
            result.drain(0..start_index as usize);
        }
        result.drain(length..);
        Ok(result)
    }
}

fn is_safe_sector(sector: u32) -> bool {
    (sector >= 0x12 && sector < 0x17) /* Image */
    || (sector == 0x3E) /* Points */
    || (sector == 0x1FE) /* Gems */
}

fn open_device<T: UsbContext>(
    context: &mut T,
    vid: u16,
    pid: u16,
) -> Result<Option<(Device<T>, DeviceDescriptor, DeviceHandle<T>)>, Me2Error> {
    let devices = match context.devices() {
        Ok(d) => d,
        Err(_) => return Err(Me2Error::UsbError(rusb::Error::Other)),
    };

    for device in devices.iter() {
        let device_desc = match device.device_descriptor() {
            Ok(d) => d,
            Err(_) => continue,
        };

        if device_desc.vendor_id() == vid && device_desc.product_id() == pid {
            match device.open() {
                Ok(handle) => return Ok(Some((device, device_desc, handle))),
                Err(e) => return Err(Me2Error::UsbError(e)),
            }
        }
    }

    Ok(None)
}

pub fn hexdump(data: &[u8]) {
    for (i, chunk) in data.chunks(16).enumerate() {
        print!("{:08x}: ", i * 16);
        for byte in chunk {
            print!("{:02x} ", byte);
        }
        for _ in 0..(16 - chunk.len()) {
            print!("   ");
        }
        print!("|");
        for byte in chunk {
            if *byte > 0x28 && *byte < 0x7F {
                print!("{}", *byte as char);
            } else {
                print!(".");
            }
        }
        println!("|");
    }
    println!("");
}

pub fn hexdump_u16(data: &[u16], start: usize) {
    for (i, chunk) in data.chunks(16).enumerate() {
        print!("{:08x}: ", (i * 16) + start);
        for byte in chunk {
            print!("{:04x} ", byte);
        }

        // print ascii representation
        print!("|");
        for word in chunk {
            let b1 = (word >> 8) as u8;
            let b2 = (word & 0xFF) as u8;

            for b in [b1, b2] {
                if b > 0x28 && b < 0x7F {
                    print!("{}", b as char);
                } else {
                    print!(".");
                }
            }
        }
        println!("|");
    }
}
