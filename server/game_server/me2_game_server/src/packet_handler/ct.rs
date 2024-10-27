use crate::packet::server_packet::send_chat;
use crate::{connection::ConnectionID, packet::client_packet::Ct, server::Server};

pub fn handle_ct(server: &mut Server, connection_id: ConnectionID, data: &Ct) {
    println!("Chat message: {:#?}", data);

    if data.target.is_empty() {
        let username = {
            let connection = server.connections.get_connection(connection_id);
            let Some(player) = &connection.player else {
                return;
            };
            player.display_name.clone()
        };
        for (_, other_connection) in server.connections.iter_mut() {
            send_chat(other_connection, &username, &data.chat);
        }
    }
}
