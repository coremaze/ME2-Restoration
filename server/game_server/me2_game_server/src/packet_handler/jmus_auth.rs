use crate::{
    avatar::get_avatar_data, connection::ConnectionID, packet::JmusAuth, player::Player,
    server::Server,
};

pub fn handle_jmus_auth(server: &mut Server, connection_id: ConnectionID, data: &JmusAuth) {
    let session_id = &data.session_id;

    let mut valid = true;
    for (_, connection) in server.connections.iter() {
        let Some(player) = &connection.player else {
            continue;
        };
        if player.session_id == *session_id {
            println!("Session ID already in use: {session_id}");
            valid = false;
            break;
        }
    }

    let resp = if valid { "VALID" } else { "INVALID" };
    {
        let connection = server.connections.get_connection_mut(connection_id);

        if valid {
            connection.player = Some(Player::new(session_id, connection_id.to_num() as u32));
        }

        connection.send(resp).ok();
    }

    // Send avatar data to all other players
    if let Some(my_avatar_data) = get_avatar_data(server, connection_id) {
        for (_, connection) in server.connections.iter_mut() {
            if let Some(player) = &connection.player {
                if player.ingame {
                    connection.send(&my_avatar_data).ok();
                }
            }
        }
    }
}
