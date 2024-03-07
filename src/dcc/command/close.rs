use crate::{
    error::{error_command::ErrorCommand, error_msg::ErrorMsg},
    parser::dcc_message::DccMessage,
};

/// Represents a DCC CLOSE message.
pub struct DccClose {
    /// The IP address associated with the DCC CLOSE message.
    ip: String,

    /// The port number associated with the DCC CLOSE message.
    port: String,
}

impl DccClose {
    /// Creates a new `DccClose` instance from a `DccMessage`.
    ///
    /// # Arguments
    ///
    /// * `dcc_message` - The DCC message from which to extract information.
    ///
    /// # Returns
    ///
    /// A Result containing the created `DccClose` or an `ErrorMsg` in case of failure.
    pub fn new(dcc_message: DccMessage) -> Result<DccClose, ErrorMsg> {
        let ip = dcc_message.get_param_from_msg(0);
        let port = dcc_message.get_param_from_msg(1);

        if let (Some(port), Some(ip)) = (port, ip) {
            Ok(Self { port, ip })
        } else {
            Err(ErrorMsg::InvalidMsg(ErrorCommand::MissingParametersDcc(
                super::DccCommand::Resume,
            )))
        }
    }

    /// Gets the IP address associated with the DCC CLOSE message.
    ///
    /// # Returns
    ///
    /// The IP address as a String.
    pub fn ip(&self) -> String {
        self.ip.clone()
    }

    /// Gets the port number associated with the DCC CLOSE message.
    ///
    /// # Returns
    ///
    /// The port number as a String.
    pub fn port(&self) -> String {
        self.port.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_dcc_close_new_valid() {
        // Test case with valid parameters
        let dcc_message = DccMessage::from_str("dcc close 127.0.0.1 8080").unwrap();
        let dcc_close_result = DccClose::new(dcc_message);
        assert!(dcc_close_result.is_ok());

        let dcc_close = dcc_close_result.unwrap();
        assert_eq!(dcc_close.ip(), "127.0.0.1");
        assert_eq!(dcc_close.port(), "8080");
    }

    #[test]
    fn test_dcc_close_new_invalid_missing_parameters() {
        // Test case with missing parameters
        let dcc_message = DccMessage::from_str("dcc close 127.0.0.1").unwrap();
        let dcc_close_result = DccClose::new(dcc_message);
        assert!(dcc_close_result.is_err());
    }

    #[test]
    fn test_dcc_close_ip() {
        // Test getting IP from DccClose
        let dcc_message = DccMessage::from_str("dcc close 192.168.0.1 8080").unwrap();
        let dcc_close = DccClose::new(dcc_message).unwrap();
        assert_eq!(dcc_close.ip(), "192.168.0.1");
    }

    #[test]
    fn test_dcc_close_port() {
        // Test getting port from DccClose
        let dcc_message = DccMessage::from_str("dcc close 127.0.0.1 9090").unwrap();
        let dcc_close = DccClose::new(dcc_message).unwrap();
        assert_eq!(dcc_close.port(), "9090");
    }
}
