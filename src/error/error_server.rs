use crate::error::error_msg::ErrorMsg;
use crate::error::error_user::ErrorUser;
use std::sync::mpsc::{RecvError, SendError};

///
/// enum that implements the different errors
/// that the server can present. In turn, it implements
/// the trait From<SendError<String>> and the tratit
/// From<std::io::Error>
///
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ErrorServer {
    TcpStreamError(std::io::ErrorKind),
    TcpFail,
    LockedResource,
    PoisonedThread,
    ChannelError,
    UnacceptedClient,
    ServerClosed,
    UnreachableClient,
    UnknownCommand,
    UnexpectedCommand,
    BadQuery,
    DCCError,
}

impl From<std::io::Error> for ErrorServer {
    fn from(error: std::io::Error) -> Self {
        ErrorServer::TcpStreamError(error.kind())
    }
}

impl From<ErrorUser> for ErrorServer {
    fn from(_error: ErrorUser) -> Self {
        ErrorServer::UnacceptedClient
    }
}

impl From<ErrorMsg> for ErrorServer {
    fn from(_error: ErrorMsg) -> Self {
        ErrorServer::UnknownCommand
    }
}

impl<T> From<SendError<T>> for ErrorServer {
    fn from(_error: SendError<T>) -> Self {
        ErrorServer::ChannelError
    }
}
impl From<RecvError> for ErrorServer {
    fn from(_error: RecvError) -> Self {
        ErrorServer::ChannelError
    }
}

impl std::fmt::Display for ErrorServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_uppercase())
    }
}

#[cfg(test)]
mod test {
    use std::{
        io::{Error, ErrorKind},
        sync::mpsc,
    };

    use crate::error::{error_msg::ErrorMsg, error_user::ErrorUser};

    use super::ErrorServer;

    #[test]
    fn debugs_correctly() {
        let error = ErrorServer::UnknownCommand;
        assert_eq!(format!("{error:?}"), "UnknownCommand")
    }

    #[test]
    fn displays_correctly_for_unknown() {
        let error = ErrorServer::UnknownCommand;
        assert_eq!(format!("{error}"), "UNKNOWNCOMMAND")
    }

    #[test]
    fn displays_correctly_for_unaccepted_client() {
        let error = ErrorServer::UnacceptedClient;
        assert_eq!(format!("{error}"), "UNACCEPTEDCLIENT")
    }

    #[test]
    fn from_std_error() {
        let error = Error::new(ErrorKind::Other, "oh no!");
        let error_server = ErrorServer::from(error);
        assert_eq!(format!("{error_server:?}"), "TcpStreamError(Other)");
    }

    #[test]
    fn from_error_user() {
        let error = ErrorUser::BuildError;
        let error_server = ErrorServer::from(error);
        assert_eq!(format!("{error_server:?}"), "UnacceptedClient");
    }

    #[test]
    fn from_error_msg() {
        let error = ErrorMsg::EmptyMsg;
        let error_server = ErrorServer::from(error);
        assert_eq!(format!("{error_server:?}"), "UnknownCommand");
    }

    #[test]
    fn from_error_send() {
        let (tx, _rx) = mpsc::channel::<String>();
        std::mem::drop(_rx);
        let result = tx.send("message".to_string());
        if let Err(c) = result {
            let err = ErrorServer::from(c);
            assert_eq!(format!("{err:?}"), "ChannelError")
        }
    }
}
