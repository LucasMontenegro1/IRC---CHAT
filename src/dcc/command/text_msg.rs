use std::io::Write;

use crate::{
    dcc::dcc_connection::{DccConnection, Stream},
    error::{error_command::ErrorCommand, error_msg::ErrorMsg, error_server::ErrorServer},
    parser::dcc_message::DccMessage,
};

/// Represents a text message for DCC communication.
#[derive(Clone, Debug)]
pub struct DccTextMsg {
    /// The IP address associated with the text message.
    ip: String,

    /// The content of the text message.
    message: String,
}

impl DccTextMsg {
    /// Creates a new `DccTextMsg` instance from a `DccMessage`.
    ///
    /// # Arguments
    ///
    /// * `msg` - The DCC message from which to extract information.
    ///
    /// # Returns
    ///
    /// A Result containing the created `DccTextMsg` or an `ErrorMsg` in case of failure.
    pub fn new(msg: DccMessage) -> Result<Self, ErrorMsg> {
        let ip = msg.prefix();
        let message = msg.parameters();

        if let (Some(ip), Some(message)) = (ip, message) {
            Ok(Self {
                ip: ip.trim_start_matches(':').to_string(),
                message: message.join(" "),
            })
        } else {
            Err(ErrorMsg::InvalidMsg(ErrorCommand::MissingParametersDcc(
                super::DccCommand::Send,
            )))
        }
    }

    /// Sends a response using the provided `DccConnection`.
    ///
    /// # Arguments
    ///
    /// * `dcc_connection` - The DCC connection used to send the response.
    ///
    /// # Returns
    ///
    /// A Result indicating success or an `ErrorServer` in case of failure.
    pub fn response<T>(&self, mut dcc_connection: DccConnection<T>) -> Result<(), ErrorServer>
    where
        T: Stream,
    {
        dcc_connection.write_all(self.message.as_bytes())?;
        Ok(())
    }

    /// Gets the IP associated with the text message.
    ///
    /// # Returns
    ///
    /// The IP address as a String.
    pub fn get_ip(self) -> String {
        self.ip.clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::dcc::dcc_connection::DirectMessage;
    use std::{
        io::{Cursor, Read},
        sync::mpsc,
    };

    use super::*;

    fn create_dcc_connection() -> DccConnection<MockTcpStream> {
        let (tx_dcc, _rx_dcc): (mpsc::Sender<DirectMessage>, mpsc::Receiver<DirectMessage>) =
            mpsc::channel();
        let user = "test_user".to_string();
        let stream = MockTcpStream::new(&[]);
        DccConnection::new(user.clone(), stream, "1".to_string(), tx_dcc)
    }

    #[derive(Clone)]
    struct MockTcpStream {
        inner: Cursor<Vec<u8>>,
    }

    impl MockTcpStream {
        fn new(data: &[u8]) -> Self {
            MockTcpStream {
                inner: Cursor::new(data.to_vec()),
            }
        }
    }

    impl Stream for MockTcpStream {
        fn try_clone(&self) -> std::io::Result<Self> {
            Ok(self.clone())
        }
    }

    impl Write for MockTcpStream {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.inner.write(buf)
        }

        fn flush(&mut self) -> std::io::Result<()> {
            self.inner.flush()
        }

        fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
            self.inner.write_all(buf)
        }
    }

    impl Read for MockTcpStream {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            self.inner.read(buf)
        }
    }

    #[test]
    fn test_new_dcc_connection() {
        let dcc_connection = create_dcc_connection();
        assert_eq!(dcc_connection.clone().get_user(), "test_user".to_string());
        assert!(dcc_connection.clone().get_stream().is_ok());
    }

    #[test]
    fn test_clone_dcc_connection() {
        let dcc_connection = create_dcc_connection();
        let cloned_dcc_connection = dcc_connection.clone();

        assert_eq!(
            dcc_connection.get_user(),
            cloned_dcc_connection.clone().get_user()
        );
        assert!(cloned_dcc_connection.clone().get_stream().is_ok());
    }

    #[test]
    fn test_receive_msgs() {
        let (tx_dcc, rx_dcc): (mpsc::Sender<DirectMessage>, mpsc::Receiver<DirectMessage>) =
            mpsc::channel();
        let user = "test_user".to_string();
        let stream = MockTcpStream::new(b"Test DCC Message");
        let mut dcc_connection = DccConnection::new(user.clone(), stream, "1".to_string(), tx_dcc);

        dcc_connection.receive_msgs().unwrap();

        let received_msg = rx_dcc.recv().unwrap();
        assert_eq!(received_msg.get_user(), user);
        assert_eq!(received_msg.get_msg(), "Test DCC Message");
    }

    #[test]
    fn user_gets_correctly() {
        let user = "test_user".to_string();
        let dcc_connection = create_dcc_connection();
        assert_eq!(dcc_connection.get_user(), user);
    }
}
