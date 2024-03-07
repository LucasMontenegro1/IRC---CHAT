use std::io::Write;

use crate::{
    dcc::{dcc_connection::Stream, dcc_handler::DccHandler},
    error::{error_command::ErrorCommand, error_msg::ErrorMsg, error_server::ErrorServer},
    parser::dcc_message::DccMessage,
};

/// Represents a DCC RESUME command, providing information about resuming file transfers.
pub struct DccResume {
    /// The username associated with the DCC RESUME command.
    pub user: String,
    /// The IP address associated with the DCC RESUME command.
    pub ip: String,
    /// The filename being transferred in the DCC RESUME command.
    pub filename: String,
    /// The port number associated with the DCC RESUME command.
    pub port: String,
    /// The optional offset parameter indicating the position to resume from.
    pub offset: Option<String>,
}

impl DccResume {
    /// Creates a new instance of `DccResume` from a DccMessage.
    ///
    /// # Arguments
    ///
    /// * `dcc_message` - The DccMessage containing the parameters for DCC RESUME.
    ///
    /// # Returns
    ///
    /// A `Result` containing the parsed `DccResume` or an `ErrorMsg` if parsing fails.
    pub fn new(dcc_message: DccMessage) -> Result<DccResume, ErrorMsg> {
        let filename = dcc_message.get_param_from_msg(0);
        let port = dcc_message.get_param_from_msg(2);
        let ip = dcc_message.get_param_from_msg(1);
        let user = dcc_message.target_user();
        let offset = dcc_message.get_param_from_msg(3);

        if let (Some(ip), Some(port), Some(filename), Some(user)) = (ip, port, filename, user) {
            Ok(Self {
                user,
                ip,
                port,
                filename,
                offset,
            })
        } else {
            Err(ErrorMsg::InvalidMsg(ErrorCommand::MissingParametersDcc(
                super::DccCommand::Resume,
            )))
        }
    }

    /// Sends a response for the DCC RESUME command.
    ///
    /// # Arguments
    ///
    /// * `handler` - The DccHandler responsible for handling the DCC connection.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure in sending the response.
    pub fn response<T>(&self, handler: &mut DccHandler<T>) -> Result<(), ErrorServer>
    where
        T: Stream,
    {
        if let Some(download) = handler.search_downloads(format!("{}:{}", self.ip, self.port)) {
            let position = download.total_bytes_read();
            handler.write_all(
                format!(
                    "privmsg {} dcc resume {} {} {} {}",
                    self.user, self.filename, self.ip, self.port, position
                )
                .as_bytes(),
            )?;
        } else {
            println!("No se encontro la descarga");
        }
        Ok(())
    }

    /// Handles the reception of the DCC RESUME command, resuming the upload if found.
    ///
    /// # Arguments
    ///
    /// * `handler` - The DccHandler responsible for handling the DCC connection.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure in handling the reception.
    pub fn reception<T>(&self, handler: &mut DccHandler<T>) -> Result<(), ErrorServer>
    where
        T: Stream,
    {
        if let Some(upload) = handler.search_upload(format!("{}:{}", self.ip, self.port)) {
            upload.set_resume()?;
        } else {
            return Err(ErrorServer::BadQuery);
        }
        Ok(())
    }

    /// Gets the port associated with the DCC RESUME command.
    pub fn port(&self) -> String {
        self.port.clone()
    }

    /// Gets the IP associated with the DCC RESUME command.
    pub fn ip(&self) -> String {
        self.ip.clone()
    }

    /// Gets the optional offset associated with the DCC RESUME command.
    pub fn offset(&self) -> Option<String> {
        self.offset.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_dcc_resume_new() {
        // Mock DccMessage with required parameters
        let dcc_message =
            DccMessage::from_str("privmsg user DCC RESUME filename 127.0.0.1 12345").unwrap();

        // Test DccResume creation
        let dcc_resume_result = DccResume::new(dcc_message);
        assert!(dcc_resume_result.is_ok());

        // Extract DccResume from the result
        let dcc_resume = dcc_resume_result.unwrap();

        // Check individual properties
        assert_eq!(dcc_resume.user, "user");
        assert_eq!(dcc_resume.filename, "filename");
        assert_eq!(dcc_resume.ip, "127.0.0.1");
        assert_eq!(dcc_resume.port, "12345");
    }

    #[test]
    fn test_dcc_resume_new_invalid() {
        // Mock DccMessage with required parameters
        let dcc_message =
            DccMessage::from_str("privmsg user DCC RESUME filename 127.0.0.1").unwrap();

        // Test DccResume creation
        let dcc_resume_result = DccResume::new(dcc_message);
        assert!(dcc_resume_result.is_err());
    }
}
