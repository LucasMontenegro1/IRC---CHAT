use std::io::Write;

use crate::{
    dcc::{dcc_connection::Stream, dcc_handler::DccHandler},
    error::{error_command::ErrorCommand, error_msg::ErrorMsg, error_server::ErrorServer},
    parser::dcc_message::DccMessage,
};

/// Represents a DCC PAUSE command, providing information about pausing file transfers.
pub struct DccPause {
    /// The username associated with the DCC PAUSE command.
    pub user: String,
    /// The filename being transferred in the DCC PAUSE command.
    pub filename: String,
    /// The port number associated with the DCC PAUSE command.
    pub port: String,
    /// The IP address associated with the DCC PAUSE command.
    pub ip: String,
}

impl DccPause {
    /// Creates a new instance of DccPause from a DccMessage.
    ///
    /// # Arguments
    ///
    /// * `dcc_message` - The DccMessage containing the parameters for DccPause.
    ///
    /// # Returns
    ///
    /// A Result containing the parsed DccPause or an ErrorMsg if parsing fails.
    pub fn new(dcc_message: DccMessage) -> Result<DccPause, ErrorMsg> {
        let filename = dcc_message.get_param_from_msg(0);
        let port = dcc_message.get_param_from_msg(2);
        let ip = dcc_message.get_param_from_msg(1);
        let user = dcc_message.target_user();

        if let (Some(port), Some(ip), Some(filename), Some(user)) = (port, ip, filename, user) {
            Ok(Self {
                port,
                user,
                filename,
                ip,
            })
        } else {
            Err(ErrorMsg::InvalidMsg(ErrorCommand::MissingParametersDcc(
                super::DccCommand::Resume,
            )))
        }
    }

    /// Sends a response for the DCC PAUSE command.
    ///
    /// # Arguments
    ///
    /// * `handler` - The DccHandler responsible for handling the DCC connection.
    ///
    /// # Returns
    ///
    /// A Result indicating success or failure in sending the response.
    pub fn response<T>(&self, handler: &mut DccHandler<T>) -> Result<(), ErrorServer>
    where
        T: Stream,
    {
        handler.write_all(
            format!(
                "privmsg {} dcc pause {} {} {}",
                self.user, self.filename, self.ip, self.port
            )
            .as_bytes(),
        )?;
        Ok(())
    }

    /// Gets the IP associated with the DCC PAUSE command.
    pub fn ip(&self) -> String {
        self.ip.clone()
    }

    /// Gets the port associated with the DCC PAUSE command.
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
    fn test_dcc_pause_new() {
        // Mock DccMessage with required parameters
        let dcc_message =
            DccMessage::from_str("privmsg user DCC PAUSE filename 127.0.0.1 12345 0").unwrap();

        // Test DccPause creation
        let dcc_pause_result = DccPause::new(dcc_message);
        assert!(dcc_pause_result.is_ok());

        // Extract DccPause from the result
        let dcc_pause = dcc_pause_result.unwrap();

        // Check individual properties
        assert_eq!(dcc_pause.user, "user");
        assert_eq!(dcc_pause.filename, "filename");
        assert_eq!(dcc_pause.ip, "127.0.0.1");
        assert_eq!(dcc_pause.port, "12345");
    }

    #[test]
    fn test_dcc_pause_new_invalid() {
        // Mock DccMessage with required parameters
        let dcc_message =
            DccMessage::from_str("privmsg user DCC PAUSE filename 127.0.0.1").unwrap();

        // Test DccPause creation
        let dcc_pause_result = DccPause::new(dcc_message);
        assert!(dcc_pause_result.is_err());
    }

    #[test]
    fn test_dcc_pause_response() {
        // Create a mock DccHandler with a MockStream
        // Crear un canal (channel) para enviar mensajes
        let (tx_dcc, _) = mpsc::channel();

        // Crear un nuevo DccHandler con el sender del canal
        let mut mock_handler = DccHandler::new(MockTcpStream::new(&[]), tx_dcc);

        // Create a sample DccPause
        let dcc_pause = DccPause {
            user: "test_user".to_string(),
            filename: "test_file".to_string(),
            ip: "127.0.0.1".to_string(),
            port: "12345".to_string(),
        };

        // Test response function
        let response_result = dcc_pause.response(&mut mock_handler);
        assert!(response_result.is_ok());
    }
}
