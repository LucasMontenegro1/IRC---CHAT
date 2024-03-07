///
/// enum that contains the different kind
/// of user errors.
///
#[derive(Debug)]
pub enum ErrorUser {
    BuildError,
}

#[cfg(test)]
mod test {
    use super::ErrorUser;

    #[test]
    fn debugs_correctly() {
        let error = ErrorUser::BuildError;
        assert_eq!(format!("{error:?}"), "BuildError")
    }
}
