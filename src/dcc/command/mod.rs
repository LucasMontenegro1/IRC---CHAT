/// Module containing various DCC command implementations.
pub mod accept;
pub mod chat;
pub mod close;
pub mod pause;
pub mod resume;
pub mod send;
pub mod text_msg;

use std::{fmt, str::FromStr};

use crate::error::error_command::ErrorCommand;

/// Represents different DCC commands.
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum DccCommand {
    /// DCC CHAT command.
    Chat,

    /// DCC SEND command.
    Send,

    /// DCC RESUME command.
    Resume,

    /// DCC ACCEPT command.
    Accept,

    /// DCC CLOSE command.
    Close,

    /// DCC PAUSE command.
    Pause,

    /// DCC MSG command.
    MSG,
}

impl FromStr for DccCommand {
    type Err = ErrorCommand;

    /// Converts a string to a DccCommand.
    ///
    /// # Arguments
    ///
    /// * `input` - The string to convert.
    ///
    /// # Returns
    ///
    /// A Result containing the parsed DccCommand or an ErrorCommand if parsing fails.
    fn from_str(input: &str) -> Result<DccCommand, Self::Err> {
        match input.to_uppercase().as_str() {
            "DCC CHAT" => Ok(DccCommand::Chat),
            "DCC SEND" => Ok(DccCommand::Send),
            "DCC RESUME" => Ok(DccCommand::Resume),
            "DCC PAUSE" => Ok(DccCommand::Pause),
            "DCC ACCEPT" => Ok(DccCommand::Accept),
            "DCC CLOSE" => Ok(DccCommand::Close),
            "DCC MSG" => Ok(DccCommand::MSG),
            _ => Err(ErrorCommand::UnknownCommand),
        }
    }
}

impl fmt::Display for DccCommand {
    /// Formats the DccCommand for display.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter.
    ///
    /// # Returns
    ///
    /// A fmt::Result indicating success or failure in formatting.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DCC {}", format!("{:?}", self).to_uppercase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_dcc_command_from_str_valid() {
        // Test case with valid DCC commands
        assert_eq!(DccCommand::from_str("DCC CHAT"), Ok(DccCommand::Chat));
        assert_eq!(DccCommand::from_str("DCC SEND"), Ok(DccCommand::Send));
        assert_eq!(DccCommand::from_str("DCC RESUME"), Ok(DccCommand::Resume));
        assert_eq!(DccCommand::from_str("DCC PAUSE"), Ok(DccCommand::Pause));
        assert_eq!(DccCommand::from_str("DCC ACCEPT"), Ok(DccCommand::Accept));
        assert_eq!(DccCommand::from_str("DCC CLOSE"), Ok(DccCommand::Close));
        assert_eq!(DccCommand::from_str("DCC MSG"), Ok(DccCommand::MSG));
    }

    #[test]
    fn test_dcc_command_from_str_invalid() {
        // Test case with invalid DCC command
        assert_eq!(
            DccCommand::from_str("INVALID COMMAND"),
            Err(ErrorCommand::UnknownCommand)
        );
    }

    #[test]
    fn test_dcc_command_display() {
        // Test case for displaying DCC commands
        assert_eq!(format!("{}", DccCommand::Chat), "DCC CHAT");
        assert_eq!(format!("{}", DccCommand::Send), "DCC SEND");
        assert_eq!(format!("{}", DccCommand::Resume), "DCC RESUME");
        assert_eq!(format!("{}", DccCommand::Pause), "DCC PAUSE");
        assert_eq!(format!("{}", DccCommand::Accept), "DCC ACCEPT");
        assert_eq!(format!("{}", DccCommand::Close), "DCC CLOSE");
        assert_eq!(format!("{}", DccCommand::MSG), "DCC MSG");
    }
}
