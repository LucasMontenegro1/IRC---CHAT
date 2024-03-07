use crate::error::error_command::ErrorCommand;
use std::fmt;

use super::error_server::ErrorServer;

///
/// enum that implements the different errors
/// that the message can present. In turn, it implements
/// the trait From<ErrorCommand> and the tratit
/// Display
///
///
#[derive(Debug, PartialEq, Eq)]
pub enum ErrorMsg {
    EmptyMsg,
    InvalidMsg(ErrorCommand),
    ServerError(ErrorServer),
}

impl From<ErrorCommand> for ErrorMsg {
    fn from(error: ErrorCommand) -> Self {
        ErrorMsg::InvalidMsg(error)
    }
}

impl From<ErrorServer> for ErrorMsg {
    fn from(error: ErrorServer) -> Self {
        ErrorMsg::ServerError(error)
    }
}

impl fmt::Display for ErrorMsg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorMsg::EmptyMsg => write!(f, "Empty message")?,
            ErrorMsg::InvalidMsg(e) => write!(f, "Erroneous message: {e:}")?,
            ErrorMsg::ServerError(cmd) => write!(f, "Server Error {cmd:}")?,
        }
        writeln!(f)
    }
}

#[cfg(test)]
mod test {
    use super::{ErrorCommand, ErrorMsg};

    #[test]
    fn debugs_correctly() {
        let error = ErrorMsg::EmptyMsg;
        assert_eq!(format!("{error:?}"), "EmptyMsg")
    }

    #[test]
    fn displays_correctly_for_empty() {
        let error = ErrorMsg::EmptyMsg;
        assert_eq!(format!("{error}"), "Empty message\n")
    }

    #[test]
    fn displays_correctl_for_invalid() {
        let error = ErrorMsg::InvalidMsg(ErrorCommand::UnknownCommand);
        assert_eq!(format!("{error}"), "Erroneous message: Unknown Command\n\n")
    }

    #[test]
    fn partial_equation() {
        let error_1 = ErrorMsg::EmptyMsg;
        let error_2 = ErrorMsg::EmptyMsg;

        let result = error_1.eq(&error_2);
        assert!(result)
    }

    #[test]
    fn partial_equation_ne() {
        let error_1 = ErrorMsg::InvalidMsg(ErrorCommand::UnknownCommand);
        let error_2 = ErrorMsg::EmptyMsg;

        let result = error_1.ne(&error_2);
        assert!(result)
    }
}
