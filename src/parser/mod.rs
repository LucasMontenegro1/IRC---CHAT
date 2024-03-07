pub mod dcc_message;
use std::str::Split;
pub mod message;
///char `:` as u8
pub const COLON_U8: u8 = b':';

///
/// Function that processes the message prefix
///
/// # Arguments
///  * `s: &str`: str with the message
/// # Return
///  in case there is a prefix the function returns it as a string, in case
/// there is no prefix the funtion returns a None.
///
///
fn process_prefix(s: &str) -> Option<String> {
    if s.as_bytes()[0] == COLON_U8 {
        return Some(s.to_string());
    }
    None
}
///
/// add a prefix with the user to a message
///
pub fn add_prefix(user: Option<&str>, msg: &str) -> String {
    if msg.trim().as_bytes()[0] != COLON_U8 || msg.is_empty() {
        let mut message_to_send = String::from(COLON_U8 as char);
        let user = user.unwrap_or("None");
        message_to_send.push_str(user);
        message_to_send.push(' ');
        message_to_send.push_str(msg);
        println!("{}", message_to_send);
        message_to_send.trim().to_string()
    } else {
        msg.trim().to_string()
    }
}

pub fn convert_into_command_prefix(user: &str) -> String {
    let mut message_to_send = String::from(COLON_U8 as char);
    message_to_send.push_str(user);
    message_to_send.to_string()
}

pub fn delete_prefix(msg: &str) -> String {
    let mut msg: Vec<&str> = msg.split_whitespace().collect();
    msg.remove(0);
    msg.join(" ")
}

///
///  Function that processes the parameters of the
///  message given by the IRC protocol.
///
///  # Arguments
/// * `split: Split<char>`: split of chars with the parameters
///     of the message
///
///  # Return
///  Function returns a Option<Vec<String>>, in case there are params
/// in the message, the function returns them as a vector of Strings.
/// On the other hand, if the parameter section is empty it returns none.
///
///
fn process_params(mut split: Split<char>) -> Option<Vec<String>> {
    let mut params = vec![];
    for s in &mut split {
        if !s.trim().is_empty() {
            if s.as_bytes()[0] == COLON_U8 {
                //En caso de existir un parametro con ":"
                let mut str = s.to_owned() + " ";
                str.push_str(&split.collect::<Vec<&str>>().join(" "));
                params.push(str);
                return Some(params);
            } else {
                params.push(s.to_string())
            }
        }
    }

    if params.is_empty() {
        return None;
    }
    Some(params)
}
