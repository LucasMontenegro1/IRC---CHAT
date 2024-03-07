use std::io::Write;

use crate::{
    dcc::{dcc_connection::Stream, dcc_handler::DccHandler},
    error::{error_command::ErrorCommand, error_msg::ErrorMsg, error_server::ErrorServer},
    parser::dcc_message::DccMessage,
};

/// Represents a DCC Accept command.
pub struct DccAccept {
    /// The target user for the DCC Accept command.
    user: String,
    /// The filename associated with the DCC Accept command.
    filename: String,
    /// The IP address associated with the DCC Accept command.
    ip: String,
    /// The port number associated with the DCC Accept command.
    port: String,
    /// The position parameter associated with the DCC Accept command.
    position: u64,
}

impl DccAccept {
    /// Creates a new `DccAccept` instance from a DccMessage.
    ///
    /// # Arguments
    ///
    /// * `dcc_message` - The DccMessage containing information for creating DccAccept.
    ///
    /// # Returns
    ///
    /// * A Result containing the created DccAccept on success, or an ErrorMsg on failure.
    pub fn new(dcc_message: DccMessage) -> Result<DccAccept, ErrorMsg> {
        let user = dcc_message.target_user();
        let filename = dcc_message.get_param_from_msg(0);
        let ip = dcc_message.get_param_from_msg(1);
        let port = dcc_message.get_param_from_msg(2);
        let position = dcc_message.get_param_from_msg(3);

        if let (Some(ip), Some(port), Some(position), Some(filename), Some(user)) =
            (ip, port, position, filename, user)
        {
            if let Ok(position) = position.parse::<u64>() {
                Ok(Self {
                    ip,
                    port,
                    filename,
                    position,
                    user,
                })
            } else {
                Err(ErrorMsg::InvalidMsg(ErrorCommand::MissingParametersDcc(
                    super::DccCommand::Accept,
                )))
            }
        } else {
            Err(ErrorMsg::InvalidMsg(ErrorCommand::MissingParametersDcc(
                super::DccCommand::Accept,
            )))
        }
    }

    /// Sends a response to the DCC handler based on the information in the DccAccept command.
    ///
    /// # Arguments
    ///
    /// * `handler` - The DCC handler to send the response to.
    ///
    /// # Returns
    ///
    /// * A Result indicating success or an ErrorServer on failure.
    pub fn response<T>(&self, handler: &mut DccHandler<T>) -> Result<(), ErrorServer>
    where
        T: Stream,
    {
        handler.write_all(
            format!(
                "privmsg {} dcc accept {} {} {} {}",
                self.user, self.filename, self.ip, self.port, self.position
            )
            .as_bytes(),
        )?;
        Ok(())
    }

    /// Gets the position parameter associated with the DCC Accept command.
    pub fn position(&self) -> u64 {
        self.position
    }

    /// Gets the port number associated with the DCC Accept command.
    pub fn port(&self) -> String {
        self.port.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::str::FromStr;
    use std::sync::mpsc;

    #[derive(Clone)]
    struct MockTcpStream {
        inner: std::io::Cursor<Vec<u8>>,
    }

    impl MockTcpStream {
        fn new(data: &[u8]) -> Self {
            MockTcpStream {
                inner: std::io::Cursor::new(data.to_vec()),
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
    fn test_dcc_accept_new() {
        // Mock DccMessage with required parameters
        let dcc_message =
            DccMessage::from_str("privmsg user DCC ACCEPT filename 127.0.0.1 12345 0").unwrap();

        // Test DccAccept creation
        let dcc_accept_result = DccAccept::new(dcc_message);
        assert!(dcc_accept_result.is_ok());

        // Extract DccAccept from the result
        let dcc_accept = dcc_accept_result.unwrap();

        // Check individual properties
        assert_eq!(dcc_accept.user, "user");
        assert_eq!(dcc_accept.filename, "filename");
        assert_eq!(dcc_accept.ip, "127.0.0.1");
        assert_eq!(dcc_accept.port, "12345");
        assert_eq!(dcc_accept.position, 0);
    }

    #[test]
    fn test_dcc_accept_new_invalid() {
        // Mock DccMessage with required parameters
        let dcc_message =
            DccMessage::from_str("privmsg user DCC ACCEPT filename 127.0.0.1").unwrap();

        // Test DccAccept creation
        let dcc_accept_result = DccAccept::new(dcc_message);
        assert!(dcc_accept_result.is_err());
    }

    #[test]
    fn test_dcc_accept_response() {
        // Create a mock DccHandler with a MockStream
        // Crear un canal (channel) para enviar mensajes
        let (tx_dcc, _) = mpsc::channel();

        // Crear un nuevo DccHandler con el sender del canal
        let mut mock_handler = DccHandler::new(MockTcpStream::new(&[]), tx_dcc);

        // Create a sample DccAccept
        let dcc_accept = DccAccept {
            user: "test_user".to_string(),
            filename: "test_file".to_string(),
            ip: "127.0.0.1".to_string(),
            port: "12345".to_string(),
            position: 0,
        };

        // Test response function
        let response_result = dcc_accept.response(&mut mock_handler);
        assert!(response_result.is_ok());
    }
}
