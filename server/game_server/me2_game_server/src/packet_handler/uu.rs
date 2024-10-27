use crate::packet::server_packet::send_avatar;
use crate::{
    connection::ConnectionID,
    proplist::{PropValue, Proplist},
    server::Server,
};

pub fn handle_uu(server: &mut Server, connection_id: ConnectionID, settings: &str) {
    println!("UU User Update packet received: {settings}");

    let Ok(props) = Proplist::parse(settings) else {
        println!("Failed to parse settings");
        return;
    };

    let connection = server.connections.get_connection_mut(connection_id);
    let Some(player) = &mut connection.player else {
        println!("Player not found");
        return;
    };

    if let Some(customization) = props.get_string("cm") {
        println!("Customization: {customization}");
        player.customization = customization.to_string();
    }

    player.ingame = true;

    // Send all other player's avatar data to the new player
    let mut av_id_name_custs: Vec<(u32, String, String)> = Vec::new();
    for (&other_connection_id, other_connection) in server.connections.iter() {
        if other_connection_id == connection_id {
            continue;
        }
        if let Some(player) = &other_connection.player {
            av_id_name_custs.push((
                player.avatar_id,
                player.display_name.clone(),
                player.customization.clone(),
            ));
        }
    }

    let connection = server.connections.get_connection_mut(connection_id);
    for (avatar_id, display_name, customization) in av_id_name_custs {
        send_avatar(connection, avatar_id.into(), &display_name, &customization);
    }
}
