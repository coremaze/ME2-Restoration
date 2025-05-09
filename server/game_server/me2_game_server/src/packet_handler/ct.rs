use crate::{
    commands::handle_command, connection::ConnectionID, packet::client_packet::Ct,
    packet::server_packet::send_chat, server::Server,
};

pub fn handle_ct(server: &mut Server, connection_id: ConnectionID, data: &Ct) {
    // println!("Chat message: {:#?}", data);

    // The client checks this in two different ways
    if data.target == "GET" || (data.chat.is_empty() && data.target.is_empty()) {
        let connection = server.connections.get_connection_mut(connection_id);
        let Some(player) = &mut connection.player else {
            return;
        };
        let chats = player.unsent_chats.clone();
        player.unsent_chats.clear();

        for (username, chat) in chats {
            // println!("Sending chat from {username}: {chat}");
            send_chat(connection, &username, &chat);
        }
    } else if data.target.is_empty() {
        if handle_command(server, connection_id, &data.chat) {
            return;
        }
        let username = {
            let connection = server.connections.get_connection(connection_id);
            let Some(player) = &connection.player else {
                return;
            };
            player.display_name.clone()
        };
        for (_, other_connection) in server.connections.iter_mut() {
            if let Some(player) = &mut other_connection.player {
                player
                    .unsent_chats
                    .push((username.clone(), data.chat.clone()));
            }
        }
        println!("[Chat] {username}: {}", data.chat);
    }
}
