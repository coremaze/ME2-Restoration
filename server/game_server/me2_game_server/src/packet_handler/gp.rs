use crate::packet::server_packet::send_avatar;
use crate::{connection::ConnectionID, server::Server};

pub fn handle_gp(server: &mut Server, connection_id: ConnectionID, avatar_id: &str) {
    println!("Handling GP packet for avatar_id: {avatar_id}");
    let Ok(avatar_id_num) = avatar_id.parse::<u32>() else {
        return;
    };

    let mut avatar_id_name_cust: Option<(u32, String, String)> = None;
    for (_, connection) in server.connections.iter() {
        let Some(other_player) = &connection.player else {
            continue;
        };

        if avatar_id_num == other_player.avatar_id {
            avatar_id_name_cust = Some((
                other_player.avatar_id,
                other_player.display_name.clone(),
                other_player.customization.clone(),
            ));
        }
    }

    if let Some((avatar_id, display_name, customization)) = avatar_id_name_cust {
        let connection = server.connections.get_connection_mut(connection_id);
        send_avatar(connection, avatar_id.into(), &display_name, &customization);
    }
}
