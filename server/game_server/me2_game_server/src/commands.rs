use std::str::SplitWhitespace;

use crate::{
    connection::ConnectionID,
    packet::server_packet::{
        send_admin_command, send_chat, send_chat_alert, send_server_message, AdminCommand,
    },
    server::Server,
};

// Handles commands from the client via chat, returns true if the command was handled
pub fn handle_command(server: &mut Server, connection_id: ConnectionID, command: &str) -> bool {
    if !command.starts_with(".") {
        return false;
    }

    let Some(command) = command.get(1..) else {
        return false;
    };

    let args = command.split_whitespace();

    match command {
        "help" => help(server, connection_id, args),
        "list" => list(server, connection_id, args),
        "whereami" => whereami(server, connection_id, args),
        "killme" => killme(server, connection_id, args),
        _ => false,
    }
}

fn help(server: &mut Server, connection_id: ConnectionID, args: SplitWhitespace) -> bool {
    const COMMANDS: [(&str, &str); 4] = [
        ("help", "Shows help message"),
        ("list", "Lists all players in the game"),
        ("whereami", "Shows your current location"),
        ("killme", "Disconnects you from the game"),
    ];

    let commands = COMMANDS
        .iter()
        .map(|(command, description)| format!("  .{command} - {description}"))
        .collect::<Vec<String>>()
        .join("\n");

    send_server_message(
        server.connections.get_connection_mut(connection_id),
        &commands,
    );

    true
}

fn list(server: &mut Server, connection_id: ConnectionID, args: SplitWhitespace) -> bool {
    let connections = server.connections.iter().collect::<Vec<_>>();
    let players = connections
        .iter()
        .filter_map(|(_, connection)| connection.player.as_ref())
        .collect::<Vec<_>>();

    let player_names_and_coordinates = players
        .iter()
        .map(|player| {
            format!(
                "{}: {} {} {}",
                player.display_name, player.location.0, player.location.1, player.location.2
            )
        })
        .collect::<Vec<String>>()
        .join("\n");
    let message = format!("Players:\n{player_names_and_coordinates}");

    send_chat_alert(
        server.connections.get_connection_mut(connection_id),
        &message,
    );

    true
}

fn whereami(server: &mut Server, connection_id: ConnectionID, args: SplitWhitespace) -> bool {
    let connection = server.connections.get_connection(connection_id);
    let Some(player) = &connection.player else {
        return false;
    };

    let message = format!(
        "Your coordinates:\nx = {} y = {} z = {}",
        player.location.0, player.location.1, player.location.2
    );

    send_chat_alert(
        server.connections.get_connection_mut(connection_id),
        &message,
    );

    true
}

fn killme(server: &mut Server, connection_id: ConnectionID, args: SplitWhitespace) -> bool {
    send_admin_command(
        server.connections.get_connection_mut(connection_id),
        AdminCommand::Kill,
    );
    true
}
