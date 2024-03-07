use std::str::FromStr;

use crate::command::Command;
use crate::error::error_command::ErrorCommand;
use crate::error::error_msg::ErrorMsg;
use crate::error::error_server::ErrorServer;
use crate::parser::{process_params, process_prefix};

/// Space char
pub const SPACE_CHAR: char = ' ';

///char `:` as u8
pub const COLON_U8: u8 = b':';

///
/// Representation of a message according to the IRC protocol
/// Each IRC message may consist of up to three main parts: the prefix
/// (optional), the command, and the command parameters (of which there
/// may be up to 15).  The prefix, command, and all parameters are
/// separated by one (or more) ASCII space character(s) (0x20).
///
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Message {
    prefix: Option<String>,
    command: Command,
    parameters: Option<Vec<String>>,
}

///
/// Implementation of the FromStr trait for message.
/// This trait is implemented to parse messages according
/// to the irc protocol.
///
///
impl FromStr for Message {
    type Err = ErrorMsg;
    ///
    ///Function that parses a str returning the resulting message
    ///
    /// # arguments
    ///  * `s: &str`: Message in str that will be parsed into a message.
    ///
    /// # Return
    ///  The function Returns a Result, which can be a message or an error if there is one.
    ///
    /// # Example
    ///  ```rust
    ///     # use irc_project::parser::message::Message;
    ///     # use std::str::FromStr;
    ///     let string = String::from("LIST #twilight_zone,#42 toulsun.uolu.fi");
    ///     let result = Message::from_str(&string).unwrap();
    ///  ```
    ///
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (prefix, parameters);
        let mut command = Err(ErrorCommand::UnknownCommand);
        let mut split = s.trim().split(SPACE_CHAR);

        match split.next() {
            Some(value) => {
                if value.is_empty() {
                    return Err(ErrorMsg::EmptyMsg);
                }
                prefix = process_prefix(value);
                match prefix {
                    None => command = Command::from_str(value),
                    Some(_) => {
                        for value in &mut split {
                            if !value.trim().is_empty() {
                                command = Command::from_str(value);
                                break;
                            }
                        }
                    }
                }
            }
            None => return Err(ErrorMsg::EmptyMsg),
        }
        parameters = process_params(split);
        let command = command?;
        let m = Message {
            prefix,
            command,
            parameters,
        };

        Ok(m)
    }
}

///
/// Implementation of the ToString trait for message.
/// This trait is implemented to parse messages according
/// to the irc protocol.
///
impl ToString for Message {
    ///
    ///Function converts a message into a String following
    /// the IRC protocol.
    ///
    ///
    /// # Return
    ///  The function Returns a String, with the message.
    ///
    /// # Example
    ///  ```rust
    ///     # use irc_project::parser::message::Message;
    ///     # use irc_project::command::Command;
    ///     let prefix = Some(String::from(":testnick"));
    ///     let command = Command::User;
    ///     let parameters = Some(vec![
    ///         String::from("guest"),
    ///         String::from("tolmoon"),
    ///         String::from("tolsun"),
    ///         String::from(":Ronnie Reagan"),]);
    ///     let message = Message::new(prefix, command, parameters);
    ///     let string = String::from(":testnick USER guest tolmoon tolsun :Ronnie Reagan");
    ///     let resultado = message.to_string();
    ///     assert_eq!(string, resultado);
    ///  ```
    ///
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

impl Message {
    ///
    /// representation of a message
    /// according to the IRC protocol.
    ///
    /// # Arguments
    /// * `prefix:Option<String>`: IRC prefix
    /// * `command:String` :The IRC command as a string
    /// * `parameters : Option<Vec<String>>`: Parameters of the IRC command
    ///    as a Option Vector of Strings
    ///
    ///
    pub fn new(prefix: Option<String>, command: Command, parameters: Option<Vec<String>>) -> Self {
        Message {
            prefix,
            command,
            parameters,
        }
    }

    pub fn prefix(&self) -> Option<String> {
        if let Some(nickname) = &self.prefix {
            let mut n = nickname.to_owned();
            n.remove(0);
            return Some(n);
        }
        self.prefix.clone()
    }

    pub fn command(&self) -> Command {
        self.command.clone()
    }

    pub fn parameters(&self) -> Option<Vec<String>> {
        self.parameters.to_owned()
    }

    ///
    /// function that parses a message
    /// and returns the index's parameter
    ///
    pub fn get_param_from_msg(&self, index: usize) -> Option<String> {
        if let Some(c) = self.parameters.clone() {
            if let Some(ch) = c.get(index) {
                return Some(ch.clone());
            }
        };
        None
    }

    pub fn is_command(&self, command: Command) -> bool {
        self.command.eq(&command)
    }
}

impl From<ErrorServer> for Message {
    fn from(e: ErrorServer) -> Self {
        let mut msg = String::from(":");
        msg.push_str(e.to_string().as_str());
        Message::new(None, Command::Quit, Some(vec![msg]))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn message_as_string() {
        let prefix = Some(String::from(":testnick"));
        let command = Command::User;
        let parameters = Some(vec![
            String::from("guest"),
            String::from("tolmoon"),
            String::from("tolsun"),
            String::from(":Ronnie Reagan"),
        ]);
        let message = Message::new(prefix, command, parameters);
        let string = String::from(":testnick USER guest tolmoon tolsun :Ronnie Reagan");

        assert_eq!(string, message.to_string());
    }

    #[test]
    fn message_with_empty_prefix_as_string() {
        let command = Command::User;
        let parameters = Some(vec![
            String::from("guest"),
            String::from("tolmoon"),
            String::from("tolsun"),
            String::from(":Ronnie Reagan"),
        ]);
        let message = Message::new(None, command, parameters);
        let string = String::from("USER guest tolmoon tolsun :Ronnie Reagan");

        assert_eq!(string, message.to_string());
    }

    #[test]
    fn message_with_empty_parameters() {
        let string = String::from("NAMES");
        let message = Message::new(None, Command::Names, None);
        assert_eq!(string, message.to_string());
    }

    #[test]
    fn parse_empty_msg() {
        let string = String::from("  ");
        let result = Message::from_str(&string);
        assert!(result.is_err());
    }

    #[test]
    fn parse_invalid_msg_only_prefix() {
        let string = String::from(":Wiz");
        let result = Message::from_str(&string);
        assert!(result.is_err());
    }
    #[test]
    fn parse_invalid_msg_only_params() {
        let string = String::from("Wiz tolsun tolmoon");
        let result = Message::from_str(&string);
        assert!(result.is_err());
    }

    #[test]
    fn parse_msg_with_whitespaces() {
        let string = String::from("  NICK     Wiz    ");
        let result = Message::from_str(&string);
        assert!(result.is_ok());

        let string = String::from("   :Kilroy   NICK     Wiz    ");
        let result = Message::from_str(&string);
        assert!(result.is_ok());
    }

    #[test]
    fn parse_nick_command() {
        let string = String::from("NICK Wiz");
        let result = Message::from_str(&string).unwrap();
        let expected = Message {
            prefix: None,
            command: Command::Nick,
            parameters: Some(vec![String::from("Wiz")]),
        };

        assert_eq!(expected, result);

        let string = String::from(":Wiz NICK Kilroy");
        let result = Message::from_str(&string).unwrap();
        let expected = Message {
            prefix: Some(String::from(":Wiz")),
            command: Command::Nick,
            parameters: Some(vec![String::from("Kilroy")]),
        };

        assert_eq!(expected, result);
    }

    #[test]
    fn parse_user_command() {
        let string = String::from("USER guest tolmoon tolsun :Ronnie Reagan");
        let result = Message::from_str(&string).unwrap();
        let expected = Message {
            prefix: None,
            command: Command::User,
            parameters: Some(vec![
                String::from("guest"),
                String::from("tolmoon"),
                String::from("tolsun"),
                String::from(":Ronnie Reagan"),
            ]),
        };

        assert_eq!(expected, result);

        let string = String::from(":testnick USER guest tolmoon tolsun :Ronnie Reagan");
        let result = Message::from_str(&string).unwrap();
        let expected = Message {
            prefix: Some(String::from(":testnick")),
            command: Command::User,
            parameters: Some(vec![
                String::from("guest"),
                String::from("tolmoon"),
                String::from("tolsun"),
                String::from(":Ronnie Reagan"),
            ]),
        };

        assert_eq!(expected, result);
    }

    #[test]
    fn parse_pass_command() {
        let string = String::from("PASS secretpasswordhere");
        let result = Message::from_str(&string).unwrap();
        let expected = Message {
            prefix: None,
            command: Command::Pass,
            parameters: Some(vec![String::from("secretpasswordhere")]),
        };

        assert_eq!(expected, result);

        let string = String::from(":testnick USER guest tolmoon tolsun :Ronnie Reagan");
        let result = Message::from_str(&string).unwrap();
        let expected = Message {
            prefix: Some(String::from(":testnick")),
            command: Command::User,
            parameters: Some(vec![
                String::from("guest"),
                String::from("tolmoon"),
                String::from("tolsun"),
                String::from(":Ronnie Reagan"),
            ]),
        };

        assert_eq!(expected, result);
    }

    #[test]
    fn parse_oper_command() {
        let string = String::from("OPER foo bar");
        let result = Message::from_str(&string).unwrap();
        let expected = Message {
            prefix: None,
            command: Command::Oper,
            parameters: Some(vec![String::from("foo"), String::from("bar")]),
        };

        assert_eq!(expected, result);
    }
    #[test]
    fn parse_squit_command() {
        let string = String::from("SQUIT toulsun.oulu.fi :Bad link ?");
        let result = Message::from_str(&string).unwrap();
        let expected = Message {
            prefix: None,
            command: Command::Squit,
            parameters: Some(vec![
                String::from("toulsun.oulu.fi"),
                String::from(":Bad link ?"),
            ]),
        };

        assert_eq!(expected, result);

        let string = String::from(":Trillian SQUIT toulsun.oulu.fi :Bad link ?");
        let result = Message::from_str(&string).unwrap();
        let expected = Message {
            prefix: Some(String::from(":Trillian")),
            command: Command::Squit,
            parameters: Some(vec![
                String::from("toulsun.oulu.fi"),
                String::from(":Bad link ?"),
            ]),
        };

        assert_eq!(expected, result);
    }

    #[test]
    fn parse_quit_command() {
        let string = String::from("QUIT :Gone to have lunch");
        let result = Message::from_str(&string).unwrap();
        let expected = Message {
            prefix: None,
            command: Command::Quit,
            parameters: Some(vec![String::from(":Gone to have lunch")]),
        };

        assert_eq!(expected, result);
    }

    #[test]
    fn parse_privmsg_command() {
        let string = String::from(":Angel PRIVMSG Wiz :Hello are you receiving this message ?");
        let result = Message::from_str(&string).unwrap();
        let expected = Message {
            prefix: Some(String::from(":Angel")),
            command: Command::Privmsg,
            parameters: Some(vec![
                String::from("Wiz"),
                String::from(":Hello are you receiving this message ?"),
            ]),
        };

        assert_eq!(expected, result);

        let string = String::from("PRIVMSG $*.fi :Server toulsun.oulu.fi rebooting");
        let result = Message::from_str(&string).unwrap();
        let expected = Message {
            prefix: None,
            command: Command::Privmsg,
            parameters: Some(vec![
                String::from("$*.fi"),
                String::from(":Server toulsun.oulu.fi rebooting"),
            ]),
        };

        assert_eq!(expected, result);
    }

    #[test]
    fn parse_join_command() {
        let string = String::from("JOIN #foo,&bar fubar");
        let result = Message::from_str(&string).unwrap();
        let expected = Message {
            prefix: None,
            command: Command::Join,
            parameters: Some(vec![String::from("#foo,&bar"), String::from("fubar")]),
        };

        assert_eq!(expected, result);

        let string = String::from("JOIN #foo,#bar fubar,foobar");
        let result = Message::from_str(&string).unwrap();
        let expected = Message {
            prefix: None,
            command: Command::Join,
            parameters: Some(vec![
                String::from("#foo,#bar"),
                String::from("fubar,foobar"),
            ]),
        };

        assert_eq!(expected, result);

        let string = String::from(":Wiz JOIN #Twilight_zone");
        let result = Message::from_str(&string).unwrap();
        let expected = Message {
            prefix: Some(String::from(":Wiz")),
            command: Command::Join,
            parameters: Some(vec![String::from("#Twilight_zone")]),
        };

        assert_eq!(expected, result);
    }

    #[test]
    fn parse_part_command() {
        let string = String::from("PART #oz_ops,&group5");
        let result = Message::from_str(&string).unwrap();
        let expected = Message {
            prefix: None,
            command: Command::Part,
            parameters: Some(vec![String::from("#oz_ops,&group5")]),
        };

        assert_eq!(expected, result);
    }

    #[test]
    fn parse_mode_command() {
        let string = String::from("MODE #Finnish +o Kilroy");
        let result = Message::from_str(&string).unwrap();
        let expected = Message {
            prefix: None,
            command: Command::Mode,
            parameters: Some(vec![
                String::from("#Finnish"),
                String::from("+o"),
                String::from("Kilroy"),
            ]),
        };

        assert_eq!(expected, result);

        let string = String::from("MODE &uolu +b *!*@*.edu");
        let result = Message::from_str(&string).unwrap();
        let expected = Message {
            prefix: None,
            command: Command::Mode,
            parameters: Some(vec![
                String::from("&uolu"),
                String::from("+b"),
                String::from("*!*@*.edu"),
            ]),
        };

        assert_eq!(expected, result);

        let string = String::from(":Angel MODE Angel +i");
        let result = Message::from_str(&string).unwrap();
        let expected = Message {
            prefix: Some(String::from(":Angel")),
            command: Command::Mode,
            parameters: Some(vec![String::from("Angel"), String::from("+i")]),
        };

        assert_eq!(expected, result);
    }

    #[test]
    fn parse_topic_command() {
        let string = String::from(":Wiz TOPIC #test :New topic");
        let result = Message::from_str(&string).unwrap();
        let expected = Message {
            prefix: Some(String::from(":Wiz")),
            command: Command::Topic,
            parameters: Some(vec![String::from("#test"), String::from(":New topic")]),
        };

        assert_eq!(expected, result);
    }

    #[test]
    fn parse_names_command() {
        let string = String::from("NAMES");
        let result = Message::from_str(&string).unwrap();
        let expected = Message {
            prefix: None,
            command: Command::Names,
            parameters: None,
        };

        assert_eq!(expected, result);

        let string = String::from("NAMES #twilight_zone,#42");
        let result = Message::from_str(&string).unwrap();
        let expected = Message {
            prefix: None,
            command: Command::Names,
            parameters: Some(vec![String::from("#twilight_zone,#42")]),
        };

        assert_eq!(expected, result);
    }

    #[test]
    fn parse_list_command() {
        let string = String::from("LIST #twilight_zone,#42 toulsun.uolu.fi");
        let result = Message::from_str(&string).unwrap();
        let expected = Message {
            prefix: None,
            command: Command::List,
            parameters: Some(vec![
                String::from("#twilight_zone,#42"),
                String::from("toulsun.uolu.fi"),
            ]),
        };

        assert_eq!(expected, result);
    }

    #[test]
    fn parse_invite_command() {
        let string = String::from(":Angel INVITE Wiz #Dust");
        let result = Message::from_str(&string).unwrap();
        let expected = Message {
            prefix: Some(String::from(":Angel")),
            command: Command::Invite,
            parameters: Some(vec![String::from("Wiz"), String::from("#Dust")]),
        };

        assert_eq!(expected, result);
    }

    #[test]
    fn parse_kick_command() {
        let string = String::from("KICK &Melbourne Matthew");
        let result = Message::from_str(&string).unwrap();
        let expected = Message {
            prefix: None,
            command: Command::Kick,
            parameters: Some(vec![String::from("&Melbourne"), String::from("Matthew")]),
        };

        assert_eq!(expected, result);
    }
    /*

    */
}
