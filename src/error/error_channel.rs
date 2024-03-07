///
/// enum that implements the different errors
/// that the client can present. In turn, it implements
/// the trait From<SendError<String>> and the tratit
/// From<std::io::Error>
///
///
#[derive(Debug, PartialEq, Eq)]
pub enum ErrorChannel {
    BannedClient,
    FullChannel,
    ClientNotInvited,
    BadKey,
}

#[cfg(test)]
mod test {
    use super::ErrorChannel;

    #[test]
    fn debugs_correctly() {
        let error = ErrorChannel::BannedClient;
        assert_eq!(format!("{error:?}"), "BannedClient")
    }
}
