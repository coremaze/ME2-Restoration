use crate::proplist::PropValue;

#[derive(Debug)]
pub struct Player {
    pub session_id: String,

    pub location: (f32, f32, f32),
    pub rotation: (f32, f32, f32),
}

impl Player {
    pub fn new(session_id: String) -> Player {
        Player {
            session_id,
            location: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
        }
    }
}
