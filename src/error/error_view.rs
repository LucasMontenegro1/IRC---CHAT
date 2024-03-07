use std::sync::mpsc::SendError;

#[derive(Debug)]
pub enum ErrorView {
    ChannelError,
    CantAddWidget,
}

impl From<SendError<String>> for ErrorView {
    fn from(_error: SendError<String>) -> Self {
        ErrorView::ChannelError
    }
}

#[cfg(test)]
mod test {
    use std::sync::mpsc;

    use super::ErrorView;

    #[test]
    fn debugs_correctly() {
        let error = ErrorView::ChannelError;
        assert_eq!(format!("{error:?}"), "ChannelError")
    }

    #[test]
    fn from_error_send() {
        let (tx, _rx) = mpsc::channel::<String>();
        std::mem::drop(_rx);
        let result = tx.send("message".to_string());

        if let Err(c) = result {
            let err = ErrorView::from(c);
            assert_eq!(format!("{err:?}"), "ChannelError")
        }
    }
}
