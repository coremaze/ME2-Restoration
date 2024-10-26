use crate::{
    avatar::get_avatar_data,
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

    if let Some(PropValue::String(customization)) = props.get_element("cm") {
        println!("Customization: {customization}");
        player.customization = customization.clone();
    }

    player.ingame = true;

    // Send all other player's avatar data to the new player
    let mut avatar_datas: Vec<String> = Vec::new();
    for (&other_connection_id, other_connection) in server.connections.iter() {
        if other_connection_id == connection_id {
            continue;
        }
        if let Some(avatar_data) = get_avatar_data(server, other_connection_id) {
            avatar_datas.push(avatar_data);
        }
    }

    let connection = server.connections.get_connection_mut(connection_id);
    for avatar_data in avatar_datas {
        connection.send(&avatar_data).ok();
    }
}
