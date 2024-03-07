use std::str::FromStr;

use super::{process_params, process_prefix};
use crate::error::error_command::ErrorCommand;
use crate::{dcc::command::DccCommand, error::error_msg::ErrorMsg};

/// Space char
pub const SPACE_CHAR: char = ' ';

///char `:` as u8
pub const COLON_U8: u8 = b':';

/// Represents a message in the DCC protocol.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct DccMessage {
    /// The optional prefix of the DCC message.
    pub prefix: Option<String>,

    /// The DCC command associated with the message.
    pub command: DccCommand,

    /// Optional parameters of the DCC message.
    pub parameters: Option<Vec<String>>,

    /// The optional target user of the DCC message.
    pub target_user: Option<String>,
}

impl FromStr for DccMessage {
    type Err = ErrorMsg;

    /// Parses a string into a DccMessage.
    ///
    /// # Arguments
    ///
    /// * `s` - The string to parse into a DccMessage.
    ///
    /// # Errors
    ///
    /// Returns a `Result` containing the parsed `DccMessage` or an `ErrorMsg` if parsing fails.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut start = 0;
        let (prefix, parameters);
        let target_user;

        let trimmed = s.trim_end();

        if trimmed.ends_with("//END") {
            return DccMessage::parse_crlf_terminated_message(s);
        }
        let split: Vec<&str> = s.split_whitespace().collect();

        if split.len() < 2 {
            return Err(ErrorMsg::EmptyMsg);
        }

        prefix = process_prefix(split[0]);
        if prefix.is_none() {
            start = 1
        }
        if split[1 - start].eq_ignore_ascii_case("PRIVMSG") {
            target_user = Some(split[2 - start].trim_start_matches(':').to_string());
            start = 3;
            if prefix.is_none() {
                start -= 1
            }
        } else {
            start = 0;
            if prefix.is_some() {
                start += 1
            }

            target_user = None;
        }

        let command_str = match (split.get(start), split.get(start + 1)) {
            (Some(cmd), Some(arg)) => format!("{} {}", cmd, arg),
            _ => return Err(ErrorMsg::InvalidMsg(ErrorCommand::UnknownCommand)),
        };

        let command = DccCommand::from_str(command_str.as_str())?;
        parameters = process_params(split[start + 2..].join(" ").trim().split(SPACE_CHAR));

        Ok(DccMessage {
            prefix,
            command,
            parameters,
            target_user,
        })
    }
}

impl DccMessage {
    /// Creates a new DccMessage instance.
    ///
    /// # Arguments
    ///
    /// * `prefix` - An optional prefix for the DCC message.
    /// * `command` - The DCC command associated with the message.
    /// * `parameters` - Optional parameters of the DCC message.
    /// * `target_user` - An optional target user for the DCC message.
    ///
    /// # Returns
    ///
    /// A new `DccMessage` instance.
    pub fn new(
        prefix: Option<String>,
        command: DccCommand,
        parameters: Option<Vec<String>>,
        target_user: Option<String>,
    ) -> Self {
        DccMessage {
            prefix,
            command,
            parameters,
            target_user,
        }
    }

    /// Gets the optional prefix of the DCC message, excluding the leading ':' character if present.
    ///
    /// # Returns
    ///
    /// An `Option<String>` representing the prefix without the ':' character.
    pub fn prefix(&self) -> Option<String> {
        if let Some(nickname) = &self.prefix {
            let mut n = nickname.to_owned();
            n.remove(0);
            return Some(n);
        }
        self.prefix.clone()
    }

    /// Adds a prefix to the DCC message.
    ///
    /// # Arguments
    ///
    /// * `prefix` - A `String` representing the prefix to be added.
    pub fn add_prefix(&mut self, prefix: String) {
        self.prefix = Some(prefix);
    }

    /// Gets the optional target user of the DCC message.
    ///
    /// # Returns
    ///
    /// An `Option<String>` representing the target user.
    pub fn target_user(&self) -> Option<String> {
        self.target_user.clone()
    }

    /// Gets the DCC command associated with the message.
    ///
    /// # Returns
    ///
    /// A `DccCommand` enum representing the DCC command.
    pub fn command(&self) -> DccCommand {
        self.command.clone()
    }

    /// Gets the optional parameters of the DCC message.
    ///
    /// # Returns
    ///
    /// An `Option<Vec<String>>` representing the parameters.
    pub fn parameters(&self) -> Option<Vec<String>> {
        self.parameters.to_owned()
    }

    /// Gets a parameter from the DCC message at the specified index.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the parameter to retrieve.
    ///
    /// # Returns
    ///
    /// An `Option<String>` representing the parameter at the specified index.
    pub fn get_param_from_msg(&self, index: usize) -> Option<String> {
        if let Some(c) = self.parameters.clone() {
            if let Some(ch) = c.get(index) {
                return Some(ch.clone());
            }
        };
        None
    }

    /// Parses a CRLF-terminated DCC message without a target user and returns a DccMessage.
    ///
    /// # Arguments
    ///
    /// * `s` - A string slice representing the DCC message to be parsed.
    ///
    /// # Returns
    ///
    /// A `Result` containing the parsed `DccMessage` or an `ErrorMsg` if parsing fails.
    fn parse_crlf_terminated_message(s: &str) -> Result<Self, ErrorMsg> {
        let trimmed = s.trim_end_matches("//END");

        // Verificar si hay un prefix (cadena que comienza con ':')
        let prefix = if let Some(prefix_part) = trimmed.split_whitespace().next() {
            if prefix_part.starts_with(':') {
                process_prefix(prefix_part)
            } else {
                return Err(ErrorMsg::InvalidMsg(ErrorCommand::MissingParametersDcc(
                    DccCommand::MSG,
                )));
            }
        } else {
            return Err(ErrorMsg::InvalidMsg(ErrorCommand::MissingParametersDcc(
                DccCommand::MSG,
            )));
        };

        let command = DccCommand::MSG;
        let parameters = process_params(
            trimmed
                .split_whitespace()
                .skip(1)
                .collect::<Vec<&str>>()
                .join(" ")
                .split(SPACE_CHAR),
        );
        let target_user = None; // No hay usuario de destino en este caso

        Ok(DccMessage {
            prefix,
            command,
            parameters,
            target_user,
        })
    }

    pub fn is_command(&self, command: DccCommand) -> bool {
        self.command.eq(&command)
    }
}

/// Converts the DccMessage to its string representation.
///
/// # Returns
///
/// A `String` representing the DccMessage in the IRC message format.
impl ToString for DccMessage {
    fn to_string(&self) -> String {
        let mut message = vec![];
        let prefix = self.prefix.to_owned();
        if let Some(value) = prefix {
            message.push(value);
        }
        message.push(self.command.to_string());
        let parameters = self.parameters.to_owned();
        if let Some(value) = parameters {
            message.push(value.join(" "));
        }
        message.join(" ")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn dcc_message_chat_with_prefix() {
        let prefix: Option<String> = Some(String::from(":fer"));
        let command = DccCommand::Chat;
        let parameters = Some(vec![
            String::from("chat"),
            String::from("localhost"),
            String::from("8081"),
        ]);
        let target_user = Some(String::from("lucas"));
        let message = DccMessage::new(
            prefix.clone(),
            command,
            parameters.clone(),
            target_user.clone(),
        );
        let string = String::from(":fer PRIVMSG lucas DCC CHAT chat localhost 8081");
        let result = DccMessage::from_str(&string).unwrap();
        assert_eq!(result, message);
    }

    #[test]
    fn dcc_message_chat_without_prefix() {
        let prefix: Option<String> = None;
        let command = DccCommand::Chat;
        let parameters = Some(vec![
            String::from("chat"),
            String::from("localhost"),
            String::from("8081"),
        ]);
        let target_user = Some(String::from("lucas"));
        let message = DccMessage::new(
            prefix.clone(),
            command,
            parameters.clone(),
            target_user.clone(),
        );
        let string = String::from("PRIVMSG lucas DCC CHAT chat localhost 8081");
        let result = DccMessage::from_str(&string).unwrap();
        assert_eq!(result, message);
    }

    #[test]
    fn dcc_message_send_without_prefix() {
        let prefix: Option<String> = None;
        let command = DccCommand::Send;
        let parameters = Some(vec![
            String::from("file.txt"),
            String::from("localhost"),
            String::from("8081"),
            String::from("200"),
        ]);
        let target_user = Some(String::from("lucas"));
        let message = DccMessage::new(
            prefix.clone(),
            command,
            parameters.clone(),
            target_user.clone(),
        );
        let string = String::from("PRIVMSG lucas DCC SEND file.txt localhost 8081 200");
        let result = DccMessage::from_str(&string).unwrap();
        assert_eq!(result, message);
    }

    #[test]
    fn dcc_send_with_prefix() {
        let prefix: Option<String> = Some(":localhost:9090".to_string());
        let command = DccCommand::Send;
        let parameters = Some(vec![
            String::from("file.txt"),
            String::from("localhost"),
            String::from("8081"),
            String::from("200"),
        ]);
        let target_user = Some(String::from("lucas"));
        let message = DccMessage::new(
            prefix.clone(),
            command,
            parameters.clone(),
            target_user.clone(),
        );
        let string =
            String::from(":localhost:9090 PRIVMSG lucas DCC SEND file.txt localhost 8081 200");
        let result = DccMessage::from_str(&string).unwrap();
        assert_eq!(result, message);
    }

    #[test]
    fn dcc_message_send_with_prefix() {
        let prefix: Option<String> = Some(String::from(":fer"));
        let command = DccCommand::Send;
        let parameters = Some(vec![
            String::from("file.txt"),
            String::from("localhost"),
            String::from("8081"),
            String::from("200"),
        ]);
        let message = DccMessage::new(prefix.clone(), command, parameters.clone(), None);
        let string = String::from(":fer DCC SEND file.txt localhost 8081 200");
        let result = DccMessage::from_str(&string).unwrap();
        assert_eq!(result, message);
    }

    #[test]
    fn dcc_accept_message_from_str() {
        let prefix: Option<String> = None;
        let command = DccCommand::Accept;
        let parameters = Some(vec![
            String::from("file.txt"),
            String::from("localhost"),
            String::from("8081"),
            String::from("10"),
        ]);
        let message = DccMessage::new(prefix, command, parameters, None);
        let string = String::from("DCC ACCEPT file.txt localhost 8081 10");
        let result = DccMessage::from_str(&string).unwrap();
        assert_eq!(result, message);
    }

    #[test]
    fn dcc_accept_message_from_str_prefix() {
        let prefix: Option<String> = Some(String::from(":fer"));
        let command = DccCommand::Accept;
        let parameters = Some(vec![
            String::from("file.txt"),
            String::from("localhost"),
            String::from("8081"),
            String::from("10"),
        ]);
        let message = DccMessage::new(prefix, command, parameters, None);
        let string = String::from(":fer DCC ACCEPT file.txt localhost 8081 10");
        let result = DccMessage::from_str(&string).unwrap();
        assert_eq!(result, message);
    }

    #[test]
    fn dcc_message_crlf_terminated_from_str() {
        let string = ":localhost:9090 mensaje //END";
        let result = DccMessage::from_str(string).unwrap();

        let expected_message = DccMessage {
            prefix: Some(":localhost:9090".to_string()),
            command: DccCommand::MSG,
            parameters: Some(vec!["mensaje".to_string()]),
            target_user: None,
        };

        assert_eq!(result, expected_message);
    }

    #[test]
    fn parse_empty_msg() {
        let string = String::from("  ");
        let result = DccMessage::from_str(&string);
        assert!(result.is_err());
    }
}
