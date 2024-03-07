use std::convert::TryFrom;

///
/// Enum that contains all the codes
/// for the existant replys in the
/// IRC protocol
///
///
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Code {
    //Solo agrego de los que tenemos que implementar
    //ERRORS
    /// "<command> :Not enough parameters"
    ErrNeedmoreparams = 461,

    ///  ":You may not reregister"
    ErrAlreadyregistred = 462,

    /// ":No nickname given"
    ErrNonicknamegiven = 431,

    /// "<nick> :Erroneus nickname"
    ErrErroneusnickname = 432,

    ///"<nick> :Nickname is already in use"
    ErrNicknameinuse = 433,

    /// "<nick> :Nickname collision KILL"
    ErrNickcollision = 436,

    /// ":Password incorrect"
    ErrPasswdmismatch = 464,

    /// ":No O-lines for your host"
    ErrNooperhost = 491,

    /// ' ":Permission Denied- You're not an IRC operator"
    ErrNoprivileges = 481,

    /// "<server name> :No such server"
    ErrNosuchserver = 402,

    /// '<nickname> : no such nick/channel
    ErrNoSuchNickname = 401,

    /// 'no recipient given (<command>)
    ErrNorecipient = 411,

    ///
    /// "<channel> :You're not on that channel"
    ///
    ErrNotOnChannel = 442,

    ///
    /// "<user> <channel> :is already on channel"
    ///
    ErrUseronchannel = 443,

    ///
    /// "<channel> :You're not channel operator"
    ///
    ErrChanOPrivsNeeded = 482,

    ///
    /// "<channel name> :No such channel"
    ///
    ErrNoSuchChannel = 403,

    ///
    /// ":No text to send"
    ///
    ErrNoTextToSend = 412,

    //REPLIES
    /// ":You are now an IRC operator"
    RplYoureoper = 381,

    /// "<channel> :[[@|+]<nick> [[@|+]<nick> [...]]]"
    RplNamRply = 353,

    ///"<channel> :End of /NAMES list"
    RplyEndOfNames = 366,

    /// "Channel :Users  Name"
    RplyListStart = 321,

    /// "Channel :Users  Name"
    RplyListEnd = 323,

    /// "<channel> <# visible> :<topic>"
    RplyList = 322,

    /// "<<nick> :{[@|+]<channel><space>}"
    RplyWhoIsChannel = 319,

    /// "<nick> <user> <host> * :<real name>"
    RplyWhoIsUser = 311,

    /// "<nick> :End of /WHOIS list"
    RplyEndWhois = 318,

    ///"<channel> <user> <host> <server> <nick> \
    /// <H|G>[*][@|+] :<hopcount> <real name>"
    RplyWho = 352,

    ///"<name> :End of /WHO list"
    RplyEndWho = 315,

    /// "<channel> <nick>"
    RplyInviting = 341,

    /// <channel>:Cannot join channel (+b)
    RplyBannedFromChan = 474,

    RplyBanList = 367,

    RplyEndBanList = 368,

    RplyCantSendChannel = 404,

    /// <channel> :Cannot join channel (+k)
    RplyBadChannelKey = 475,

    /// "<nick> :<away message>"
    RplyAway = 301,

    /// ":You are no longer marked as being away"
    RplyUnaway = 305,

    /// ":You have benn marked as being away"
    RplyNowAway = 306,

    /// "<channel> :<topic> "
    RplyTopic = 332,

    /// "<channel> :No topic is set"
    RplyNoTopic = 331,

    /// "<channel> :Cannot join channle (+l)"
    ErrChannelIsFull = 471,

    /// "<channel> :Cannot join channel (+i)"
    ErrInviteOnlyChan = 473,

    /// for user and channel modes, also code 221
    RplyModes = 324,

    /// ":Cant change mode for other users"
    ErrUsersDontMatch = 502,

    /// Dummy Reply
    RplyNone = 300,
}

impl TryFrom<i32> for Code {
    type Error = ();

    ///
    /// Converts a i32 into a Code
    ///
    fn try_from(v: i32) -> Result<Self, Self::Error> {
        match v {
            x if x == Code::ErrAlreadyregistred as i32 => Ok(Code::ErrAlreadyregistred),
            x if x == Code::ErrNeedmoreparams as i32 => Ok(Code::ErrNeedmoreparams),
            x if x == Code::ErrNonicknamegiven as i32 => Ok(Code::ErrNonicknamegiven),
            x if x == Code::ErrErroneusnickname as i32 => Ok(Code::ErrErroneusnickname),
            x if x == Code::ErrNicknameinuse as i32 => Ok(Code::ErrNicknameinuse),
            x if x == Code::ErrPasswdmismatch as i32 => Ok(Code::ErrPasswdmismatch),
            x if x == Code::ErrNickcollision as i32 => Ok(Code::ErrNickcollision),
            x if x == Code::ErrNooperhost as i32 => Ok(Code::ErrNooperhost),
            x if x == Code::ErrNoprivileges as i32 => Ok(Code::ErrNoprivileges),
            x if x == Code::ErrNosuchserver as i32 => Ok(Code::ErrNosuchserver),
            x if x == Code::ErrNoSuchNickname as i32 => Ok(Code::ErrNoSuchNickname),
            x if x == Code::ErrNorecipient as i32 => Ok(Code::ErrNorecipient),
            x if x == Code::ErrNotOnChannel as i32 => Ok(Code::ErrNotOnChannel),
            x if x == Code::ErrUseronchannel as i32 => Ok(Code::ErrUseronchannel),
            x if x == Code::ErrChanOPrivsNeeded as i32 => Ok(Code::ErrChanOPrivsNeeded),
            x if x == Code::ErrNoSuchChannel as i32 => Ok(Code::ErrNoSuchChannel),
            x if x == Code::ErrChannelIsFull as i32 => Ok(Code::ErrChannelIsFull),
            x if x == Code::ErrInviteOnlyChan as i32 => Ok(Code::ErrInviteOnlyChan),
            x if x == Code::RplyNone as i32 => Ok(Code::RplyNone),
            x if x == Code::RplNamRply as i32 => Ok(Code::RplNamRply),
            x if x == Code::RplyEndOfNames as i32 => Ok(Code::RplyEndOfNames),
            x if x == Code::RplyListStart as i32 => Ok(Code::RplyListStart),
            x if x == Code::RplyListEnd as i32 => Ok(Code::RplyListEnd),
            x if x == Code::RplyList as i32 => Ok(Code::RplyList),
            x if x == Code::RplYoureoper as i32 => Ok(Code::RplYoureoper),
            x if x == Code::RplyWhoIsChannel as i32 => Ok(Code::RplyWhoIsChannel),
            x if x == Code::RplyWhoIsUser as i32 => Ok(Code::RplyWhoIsUser),
            x if x == Code::RplyEndWhois as i32 => Ok(Code::RplyEndWhois),
            x if x == Code::RplyWho as i32 => Ok(Code::RplyWho),
            x if x == Code::RplyEndWho as i32 => Ok(Code::RplyEndWho),
            x if x == Code::RplyBanList as i32 => Ok(Code::RplyBanList),
            x if x == Code::RplyEndBanList as i32 => Ok(Code::RplyEndBanList),
            x if x == Code::RplyCantSendChannel as i32 => Ok(Code::RplyCantSendChannel),
            x if x == Code::RplyBadChannelKey as i32 => Ok(Code::RplyBadChannelKey),
            x if x == Code::RplyUnaway as i32 => Ok(Code::RplyUnaway),
            x if x == Code::RplyNowAway as i32 => Ok(Code::RplyNowAway),
            x if x == Code::RplyAway as i32 => Ok(Code::RplyAway),
            x if x == Code::RplyTopic as i32 => Ok(Code::RplyTopic),
            x if x == Code::RplyNoTopic as i32 => Ok(Code::RplyNoTopic),
            x if x == Code::RplyInviting as i32 => Ok(Code::RplyInviting),
            x if x == Code::RplyBannedFromChan as i32 => Ok(Code::RplyBannedFromChan),
            x if x == Code::RplyModes as i32 => Ok(Code::RplyModes),
            x if x == Code::ErrUsersDontMatch as i32 => Ok(Code::ErrUsersDontMatch),
            x if x == Code::ErrNoTextToSend as i32 => Ok(Code::ErrNoTextToSend),
            _ => Err(()),
        }
    }
}

///
/// converts the given code into a
/// string
///
impl ToString for Code {
    fn to_string(&self) -> String {
        let x = *self as i32;
        x.to_string()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn code_can_convert_to_string() {
        let x = Code::try_from(461).unwrap();
        assert_eq!(x.to_string(), "461");
    }

    #[test]
    fn err_need_more_params() {
        let x = Code::try_from(461).unwrap();
        assert_eq!(x, Code::ErrNeedmoreparams)
    }

    #[test]
    fn err_already_registered() {
        let x = Code::try_from(462).unwrap();
        assert_eq!(x, Code::ErrAlreadyregistred)
    }

    #[test]
    fn err_no_nickname_given() {
        let x = Code::try_from(431).unwrap();
        assert_eq!(x, Code::ErrNonicknamegiven)
    }
    #[test]
    fn err_erroneus_nickname() {
        let x = Code::try_from(432).unwrap();
        assert_eq!(x, Code::ErrErroneusnickname)
    }
    #[test]
    fn err_nickname_in_use() {
        let x = Code::try_from(433).unwrap();
        assert_eq!(x, Code::ErrNicknameinuse)
    }
    #[test]
    fn err_password_missmatch() {
        let x = Code::try_from(464).unwrap();
        assert_eq!(x, Code::ErrPasswdmismatch)
    }
    #[test]
    fn err_nick_collision() {
        let x = Code::try_from(436).unwrap();
        assert_eq!(x, Code::ErrNickcollision)
    }
    #[test]
    fn err_no_oper_host() {
        let x = Code::try_from(491).unwrap();
        assert_eq!(x, Code::ErrNooperhost)
    }
    #[test]
    fn err_no_privileges() {
        let x = Code::try_from(481).unwrap();
        assert_eq!(x, Code::ErrNoprivileges)
    }
    #[test]
    fn rpl_you_are_oper() {
        let x = Code::try_from(381).unwrap();
        assert_eq!(x, Code::RplYoureoper)
    }
    #[test]
    fn err_no_such_server() {
        let x = Code::try_from(402).unwrap();
        assert_eq!(x, Code::ErrNosuchserver)
    }

    #[test]
    fn code_is_valid() {
        let x = Code::try_from(461);
        assert!(x.is_ok())
    }

    #[test]
    fn code_is_invalid() {
        let x = Code::try_from(999);
        assert!(x.is_err())
    }

    #[test]
    fn code_is_valid_as_number() {
        assert_eq!(Code::ErrAlreadyregistred as i32, 462);
    }

    #[test]
    fn clone_valid() {
        let x = Code::ErrAlreadyregistred;
        let y = x;
        assert_eq!(x, y);
    }

    #[test]
    fn debug_valid() {
        let x = Code::ErrAlreadyregistred;
        assert_eq!(format!("{:?}", x), "ErrAlreadyregistred");
    }
}
