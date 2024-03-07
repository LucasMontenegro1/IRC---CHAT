fn main() -> Result<(), ()> {
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::main;

    #[test]
    fn main_ok() {
        assert!(main().is_ok());
    }
}
