#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum UserFlag {
    S, // - marks a user for receipt of server notices;
    W, // - user receives wallops;
    O, // - operator flag.
    I, // - marks a users as invisible;
}

impl std::fmt::Display for UserFlag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

#[cfg(test)]
mod test {

    use std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
    };

    use super::UserFlag;

    #[test]
    fn display_for_channel_flag() {
        let flag = UserFlag::W;
        assert_eq!(format!("{flag}"), "w");
    }

    #[test]
    fn hash_for_flags() {
        let flag = UserFlag::W;
        let mut hasher = DefaultHasher::new();
        flag.hash(&mut hasher);
        println!("Hash is {:x}!", hasher.finish());
    }

    #[test]
    fn clone_works() {
        let flag = UserFlag::W;
        let clone = flag.clone();
        assert_eq!(format!("{flag}"), format!("{clone}"));
    }
}
