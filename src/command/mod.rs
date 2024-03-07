pub mod away;
pub mod dcc_accept;
pub mod dcc_chat;
pub mod dcc_pause;
pub mod dcc_resume;
pub mod dcc_send;
pub mod invite;
pub mod join;
pub mod kick;
pub mod kill;
pub mod list;
pub mod mode;
pub mod names;
pub mod notice_msg;
pub mod oper_msg;
pub mod part;
pub mod private_msg;
pub mod quit;
pub mod server_msg;
pub mod squit;
pub mod topic;
pub mod user_msg;
pub mod who;
pub mod whois;
use crate::error::error_command::ErrorCommand;
use std::fmt;
use std::str::FromStr;

pub mod nick_command;
pub mod pass_command;
pub mod traits;
pub mod user_command;

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    User,
    Nick,
    Pass,
    Oper,
    Quit,
    Squit,
    Privmsg,
    Notice,
    Join,
    Part,
    Mode,
    Topic,
    Names,
    List,
    Invite,
    Kick,
    Whois,
    Who,
    Away,
    Server,
    Kill,
}

impl FromStr for Command {
    type Err = ErrorCommand;
    fn from_str(input: &str) -> Result<Command, Self::Err> {
        match input.to_uppercase().as_str() {
            "USER" => Ok(Command::User),
            "NICK" => Ok(Command::Nick),
            "PASS" => Ok(Command::Pass),
            "OPER" => Ok(Command::Oper),
            "QUIT" => Ok(Command::Quit),
            "SQUIT" => Ok(Command::Squit),
            "PRIVMSG" => Ok(Command::Privmsg),
            "NOTICE" => Ok(Command::Notice),
            "JOIN" => Ok(Command::Join),
            "PART" => Ok(Command::Part),
            "MODE" => Ok(Command::Mode),
            "TOPIC" => Ok(Command::Topic),
            "NAMES" => Ok(Command::Names),
            "LIST" => Ok(Command::List),
            "INVITE" => Ok(Command::Invite),
            "KICK" => Ok(Command::Kick),
            "WHOIS" => Ok(Command::Whois),
            "WHO" => Ok(Command::Who),
            "AWAY" => Ok(Command::Away),
            "SERVER" => Ok(Command::Server),
            "KILL" => Ok(Command::Kill),
            _ => Err(ErrorCommand::UnknownCommand),
        }
    }
}

// It allow us to print the enum variant by calling command.to_string()
impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_uppercase())
    }
}

impl Clone for Command {
    fn clone(&self) -> Self {
        Command::from_str(self.to_string().as_str()).unwrap()
    }
}

#[cfg(test)]
mod test {
    use crate::command::Command;
    use std::str::FromStr;

    #[test]
    fn clone_command() {
        let command = Command::User;
        let command_copy = command.clone();
        assert_eq!(command_copy, Command::User);
    }

    #[test]
    fn command_dont_exists_is_err() {
        let command = Command::from_str("blabla");
        assert!(command.is_err());
    }

    #[test]
    fn command_from_user() {
        let command = Command::from_str("USER");
        assert_eq!(command, Ok(Command::User));
    }

    #[test]
    fn command_understand_lower_case() {
        let command = Command::from_str("user");
        assert_eq!(command, Ok(Command::User));
    }

    #[test]
    fn command_from_nick() {
        let command = Command::from_str("NICK");
        assert_eq!(command, Ok(Command::Nick));
    }

    #[test]
    fn command_from_pass() {
        let command = Command::from_str("PASS");
        assert_eq!(command, Ok(Command::Pass));
    }

    #[test]
    fn command_from_oper() {
        let command = Command::from_str("OPER");
        assert_eq!(command, Ok(Command::Oper));
    }

    #[test]
    fn command_from_quit() {
        let command = Command::from_str("QUIT");
        assert_eq!(command, Ok(Command::Quit));
    }

    #[test]
    fn command_from_squit() {
        let command = Command::from_str("SQUIT");
        assert_eq!(command, Ok(Command::Squit));
    }

    #[test]
    fn command_from_notice() {
        let command = Command::from_str("NOTICE");
        assert_eq!(command, Ok(Command::Notice));
    }

    #[test]
    fn command_from_join() {
        let command = Command::from_str("JOIN");
        assert_eq!(command, Ok(Command::Join));
    }

    #[test]
    fn command_from_part() {
        let command = Command::from_str("PART");
        assert_eq!(command, Ok(Command::Part));
    }

    #[test]
    fn command_from_mode() {
        let command = Command::from_str("MODE");
        assert_eq!(command, Ok(Command::Mode));
    }

    #[test]
    fn command_from_topic() {
        let command = Command::from_str("TOPIC");
        assert_eq!(command, Ok(Command::Topic));
    }

    #[test]
    fn command_from_names() {
        let command = Command::from_str("NAMES");
        assert_eq!(command, Ok(Command::Names));
    }

    #[test]
    fn command_from_list() {
        let command = Command::from_str("LIST");
        assert_eq!(command, Ok(Command::List));
    }

    #[test]
    fn command_from_invite() {
        let command = Command::from_str("INVITE");
        assert_eq!(command, Ok(Command::Invite));
    }

    #[test]
    fn command_from_kick() {
        let command = Command::from_str("KICK");
        assert_eq!(command, Ok(Command::Kick));
    }

    #[test]
    fn command_from_whois() {
        let command = Command::from_str("WHOIS");
        assert_eq!(command, Ok(Command::Whois));
    }

    #[test]
    fn command_from_who() {
        let command = Command::from_str("WHO");
        assert_eq!(command, Ok(Command::Who));
    }

    #[test]
    fn command_from_away() {
        let command = Command::from_str("AWAY");
        assert_eq!(command, Ok(Command::Away));
    }

    #[test]
    fn command_from_server() {
        let command = Command::from_str("SERVER");
        assert_eq!(command, Ok(Command::Server));
    }
}
