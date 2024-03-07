///
/// enum that contains the different types of errors.
///
///
#[derive(Debug, PartialEq, Eq)]
pub enum ErrorRply {
    EmptyRply,
    InvalidRply,
    NoCode,
}

#[cfg(test)]
mod test {
    use super::ErrorRply;

    #[test]
    fn debugs_correctly() {
        let error = ErrorRply::EmptyRply;
        assert_eq!(format!("{error:?}"), "EmptyRply")
    }

    #[test]
    fn partial_equation() {
        let error_1 = ErrorRply::EmptyRply;
        let error_2 = ErrorRply::EmptyRply;

        let result = error_1.eq(&error_2);
        assert!(result)
    }

    #[test]
    fn partial_equation_ne() {
        let error_1 = ErrorRply::InvalidRply;
        let error_2 = ErrorRply::EmptyRply;

        let result = error_1.ne(&error_2);
        assert!(result)
    }
}
