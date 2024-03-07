#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum ChannelFlag {
    O, // give/take channel operator privileges;
    P, // private channel ChannelFlag;
    S, // secret channel ChannelFlag;
    I, // invite-only channel ChannelFlag;
    T, // topic settable by channel operator only ChannelFlag;
    N, // no messages to channel from clients on the outside;
    M, // moderated channel;
    L, // set the user limit to channel;
    B, // set a ban mask to keep users out;
    V, // give/take the ability to speak on a moderated channel;
    K, // set a channel key (password).
}

impl std::fmt::Display for ChannelFlag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

#[cfg(test)]
mod test {

    use std::{collections::hash_map::DefaultHasher, hash::Hash};

    use super::ChannelFlag;

    #[test]
    fn display_for_channel_flag() {
        let flag = ChannelFlag::B;
        assert_eq!(format!("{flag}"), "b");
    }

    #[test]
    fn hash_for_flags() {
        let flag = ChannelFlag::B;
        let mut hasher = DefaultHasher::new();
        flag.hash(&mut hasher);
        // println!("Hash is {:x}!", hasher.finish());
    }
}
