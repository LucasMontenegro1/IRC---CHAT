use crate::{command::Command, dcc::command::DccCommand};
use std::fmt;

///
/// enum that implements the different errors
/// that the commands can present. In turn, it implements
/// the trait Display
///
///
#[derive(Debug, PartialEq, Eq)]
pub enum ErrorCommand {
    UnknownCommand,
    MissingParameters(Command),
    MissingParametersDcc(DccCommand),
}

impl fmt::Display for ErrorCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorCommand::UnknownCommand => write!(f, "Unknown Command")?,
            ErrorCommand::MissingParameters(cmd) => write!(f, "Missing parameter/s. {cmd:}")?,
            ErrorCommand::MissingParametersDcc(cmd) => write!(f, "Missing parameter/s. {cmd:}")?,
        }
        writeln!(f)
    }
}

#[cfg(test)]
mod test {
    use super::ErrorCommand;

    #[test]
    fn debugs_correctly() {
        let error = ErrorCommand::UnknownCommand;
        assert_eq!(format!("{error:?}"), "UnknownCommand")
    }

    #[test]
    fn displays_correctl_for_unknown() {
        let error = ErrorCommand::UnknownCommand;
        assert_eq!(format!("{error}"), "Unknown Command\n")
    }

    #[test]
    fn displays_correctly_for_missing_parameters() {
        let error = ErrorCommand::MissingParameters(crate::command::Command::Away);
        assert_eq!(format!("{error}"), "Missing parameter/s. AWAY\n")
    }

    #[test]
    fn partial_equation() {
        let error_1 = ErrorCommand::MissingParameters(crate::command::Command::Away);
        let error_2 = ErrorCommand::MissingParameters(crate::command::Command::Away);

        let result = error_1.eq(&error_2);
        assert!(result)
    }

    #[test]
    fn partial_equation_ne() {
        let error_1 = ErrorCommand::MissingParameters(crate::command::Command::Away);
        let error_2 = ErrorCommand::UnknownCommand;

        let result = error_1.ne(&error_2);
        assert!(result)
    }
}
