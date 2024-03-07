use std::sync::mpsc::SendError;
///
/// enum that implements the different errors
/// that the client can present. In turn, it implements
/// the trait From<SendError<String>> and the tratit
/// From<std::io::Error>
///
///
#[derive(Debug)]
pub enum ErrorClient {
    TcpStreamError(std::io::ErrorKind),
    LockedResource,
    ChannelError,
    UnacceptedClient,
    ServerClosed,
    ClientQuit,
}

impl From<SendError<String>> for ErrorClient {
    fn from(_error: SendError<String>) -> Self {
        ErrorClient::ChannelError
    }
}

impl From<std::io::Error> for ErrorClient {
    fn from(error: std::io::Error) -> Self {
        ErrorClient::TcpStreamError(error.kind())
    }
}

#[cfg(test)]
mod test {
    use super::ErrorClient;
    use std::{
        io::{Error, ErrorKind},
        sync::mpsc,
    };

    #[test]
    fn debugs_correctly() {
        let error = ErrorClient::ChannelError;
        assert_eq!(format!("{error:?}"), "ChannelError")
    }

    #[test]
    fn from_std_error_to_error_client() {
        let error = Error::new(ErrorKind::Other, "oh no!");
        let error_client = ErrorClient::from(error);
        assert_eq!(format!("{error_client:?}"), "TcpStreamError(Other)");
    }

    #[test]
    fn from_error_send() {
        let (tx, _rx) = mpsc::channel::<String>();
        std::mem::drop(_rx);
        let result = tx.send("message".to_string());
        if let Err(c) = result {
            let err = ErrorClient::from(c);
            assert_eq!(format!("{err:?}"), "ChannelError")
        }
    }
}
