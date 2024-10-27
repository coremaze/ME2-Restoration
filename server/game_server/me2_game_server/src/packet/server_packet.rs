use crate::{
    connection::Connection,
    proplist::{PropValue, Proplist},
};

pub fn send_chat(connection: &mut Connection, username: &str, message: &str) {
    let message = message.replace('"', "\" & QUOTE & \"");
    let packet = format!("L~[\"{username}: {message}\"]\r");
    connection.send(&packet).ok();
}

pub fn send_avatar(
    connection: &mut Connection,
    avatar_id: i64,
    display_name: &str,
    customization: &str,
) {
    let mut avatar_proplist = Proplist::new();
    avatar_proplist.add_element("dn", PropValue::String(display_name.to_string()));
    avatar_proplist.add_element("cm", PropValue::String(customization.to_string()));

    let packet = format!("A~{} {}\r", avatar_id, avatar_proplist);

    connection.send(&packet).ok();
}

pub fn send_auth_response(connection: &mut Connection, valid: bool) {
    let resp = if valid { "VALID" } else { "INVALID" };
    connection.send(resp).ok();
}

pub fn send_alive(connection: &mut Connection) {
    connection.send("ALIVE").ok();
}
