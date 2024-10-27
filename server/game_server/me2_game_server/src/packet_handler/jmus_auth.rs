use crate::packet::server_packet::send_auth_response;
use crate::packet::server_packet::send_avatar;
use crate::{
    connection::ConnectionID, packet::client_packet::JmusAuth,
    player::Player, server::Server,
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

    let connection = server.connections.get_connection_mut(connection_id);
    if valid {
        connection.player = Some(Player::new(session_id, connection_id.to_num() as u32));
    }
    send_auth_response(connection, valid);

    // Send avatar data to all other players
    if let Some(player) = &server.connections.get_connection(connection_id).player {
        let (avatar_id, display_name, customization) = (
            player.avatar_id,
            player.display_name.clone(),
            player.customization.clone(),
        );

        for (&cid, connection) in server.connections.iter_mut() {
            if cid != connection_id {
                send_avatar(connection, avatar_id.into(), &display_name, &customization);
            }
        }
    };
}
