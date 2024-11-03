use crate::{
    connection::ConnectionID,
    packet::client_packet::CSPacket,
    proplist::{PropValue, Proplist},
    server::Server,
};

pub fn handle_us(server: &mut Server, connection_id: ConnectionID, _packet: &CSPacket) {
    println!("US packet received");

    let connection = server.connections.get_connection_mut(connection_id);

    let Some(player) = &connection.player else {
        return;
    };

    let dn = &player.session_id;
    let cm = &player.customization;
    let uid = connection_id.to_string();

    let mut props = Proplist::new();
    props.add_element("uid", PropValue::String(uid)); // User ID
    props.add_element("dn", PropValue::String(dn.clone())); // Display Name
    props.add_element("iV", PropValue::List(vec![]));
    props.add_element("lg", PropValue::String("en".to_string())); // Language
    props.add_element("mm", PropValue::Integer(0)); // Mute
    props.add_element("cm", PropValue::String(cm.to_string())); // Avatar Customization
    props.add_element("uc", PropValue::List(vec![]));
    props.add_element("uh", PropValue::List(vec![]));
    props.add_element("bl", PropValue::List(vec![]));
    props.add_element("bu", PropValue::List(vec![])); // Buddy list
    props.add_element("cb", PropValue::Void); // Chat block password (VOID to allow chat without password)
    props.add_element(
        "pa",
        PropValue::List(vec![
            // pkMAXATTRIB is 1000
            PropValue::Float(1000.0), // Jump power
            PropValue::Float(510.0),  // Run power
            PropValue::Float(510.0),  // Trade?
            PropValue::Float(510.0),  // Something about vehicles?
        ]),
    ); // Player power(?) attributes
    props.add_element("pp", PropValue::Integer(0));
    props.add_element("ppnew", PropValue::Integer(0));
    props.add_element("gt", PropValue::Integer(0));
    props.add_element("ga", PropValue::Integer(0));

    // plHighScores
    let mut highscore_proplist = Proplist::new();
    highscore_proplist.add_element("L01A03Z02", PropValue::String("1 1 1".to_string()));
    highscore_proplist.add_element("L01A02Z02", PropValue::String("2 2 2".to_string()));
    highscore_proplist.add_element("L01A05Z02", PropValue::String("3 3 3".to_string()));
    highscore_proplist.add_element("L01A05Z03", PropValue::String("4 4 4".to_string()));
    highscore_proplist.add_element("L02A02Z03", PropValue::String("5 5 5".to_string()));
    highscore_proplist.add_element("L02A02Z02", PropValue::String("6 6 6".to_string()));
    highscore_proplist.add_element("L02A04Z02", PropValue::String("7 7 7".to_string()));
    highscore_proplist.add_element("L02A04Z03", PropValue::String("8 8 8".to_string()));
    props.add_element("hs", PropValue::Proplist(highscore_proplist));
    let resp = format!("U {}", props);
    println!("US response: {resp}");

    connection.send(&resp);
}
