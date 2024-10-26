#[derive(Debug)]
pub struct Player {
    pub session_id: String,
    pub avatar_id: u32,
    pub display_name: String,

    pub ingame: bool,

    pub customization: String,
    pub location: (f32, f32, f32),
    pub rotation: (f32, f32, f32),
}

impl Player {
    pub fn new(session_id: &str, avatar_id: u32) -> Player {
        let session_id = session_id.to_string();
        let display_name = session_id.clone();
        Player {
            session_id,
            avatar_id,
            display_name,
            ingame: false,
            customization: String::from("241111112111111"),
            location: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
        }
    }
}
