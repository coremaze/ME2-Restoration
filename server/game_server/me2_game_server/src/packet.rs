#[derive(Debug)]
pub enum CSPacket {
    JmusCheck,
    JmusAuth(JmusAuth),
    Us,         // Get user settings
    Uu(String), // Update user settings
    Mu(Mu),     // Movement update
    Ct(Ct),     // Chat message
    ImAlive,    // Keep alive
    Gp(String), // Get avatar data
}

#[derive(Debug)]
pub struct JmusAuth {
    pub session_id: String,
}

#[derive(Debug)]
pub struct Mu {
    pub movement: String,
    pub cell_list: String,
}

#[derive(Debug)]
pub struct Ct {
    pub chat: String,
    pub target: String,
}

struct PacketIterator<'a, T>
where
    T: FnMut(&u8) -> bool,
{
    split: std::slice::Split<'a, u8, T>,
    bytes_taken: usize,
}

impl<'a, T> PacketIterator<'a, T>
where
    T: FnMut(&u8) -> bool,
{
    fn new(buffer: &'a [u8], predicate: T) -> Self {
        Self {
            split: buffer.split(predicate),
            bytes_taken: 0,
        }
    }

    fn bytes_taken(&self) -> usize {
        self.bytes_taken
    }
}

impl<'a, T> Iterator for PacketIterator<'a, T>
where
    T: FnMut(&u8) -> bool,
{
    type Item = &'a [u8];
    fn next(&mut self) -> Option<Self::Item> {
        self.split.next().map(|x| {
            self.bytes_taken += x.len() + 1; // +1 for the \r
            x
        })
    }
}

pub fn take_packet(buffer: &mut Vec<u8>) -> Option<CSPacket> {
    // Iterate over segments ending in \r
    let mut segments = if buffer.contains(&b'\r') {
        PacketIterator::new(buffer, |&x| x == b'\r')
    } else {
        return None;
    };

    let packet_type = std::str::from_utf8(segments.next()?).ok()?;

    let result = match packet_type {
        "JMUS_CHECK" => Some(CSPacket::JmusCheck),
        "JMUS_AUTH" => {
            let session_id = std::str::from_utf8(segments.next()?).ok()?;
            Some(CSPacket::JmusAuth(JmusAuth {
                session_id: session_id.to_string(),
            }))
        }
        "US" => Some(CSPacket::Us),
        "UU" => {
            let settings = std::str::from_utf8(segments.next()?).ok()?;

            Some(CSPacket::Uu(settings.to_string()))
        }
        "MU" => {
            let movement = std::str::from_utf8(segments.next()?).ok()?;

            let cell_list = std::str::from_utf8(segments.next()?).ok()?;
            Some(CSPacket::Mu(Mu {
                movement: movement.to_string(),
                cell_list: cell_list.to_string(),
            }))
        }
        "CT" => {
            let chat = std::str::from_utf8(segments.next()?).ok()?;
            let target = std::str::from_utf8(segments.next()?).ok()?;

            Some(CSPacket::Ct(Ct {
                chat: chat.to_string(),
                target: target.to_string(),
            }))
        }
        "IM_ALIVE" => Some(CSPacket::ImAlive),
        "GP" => {
            let avatar_id = std::str::from_utf8(segments.next()?).ok()?;
            Some(CSPacket::Gp(avatar_id.to_string()))
        }
        _ => {
            println!("Unknown packet type: {packet_type:?}");
            None
        }
    };

    if let Some(packet) = &result {
        buffer.drain(..segments.bytes_taken());
    }
    result
}
