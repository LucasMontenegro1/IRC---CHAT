pub mod reply_maker;
use std::{
    num::ParseIntError,
    str::{FromStr, Split},
};

use self::code::Code;
use crate::error::error_reply::ErrorRply;
pub mod code;

const COLON_U8: u8 = b':';

#[derive(Debug, PartialEq, Eq)]
pub struct Reply {
    prefix: Option<String>,
    code: Code,
    parameters: Option<Vec<String>>,
}

impl Reply {
    ///
    /// creates a new reply, not recommended
    /// function since it's a generic one
    ///
    pub fn new(
        prefix: Option<String>,
        code: i32,
        parameters: Option<Vec<String>>,
    ) -> Result<Reply, ErrorRply> {
        match Code::try_from(code) {
            Ok(code) => Ok(Reply {
                prefix,
                code,
                parameters,
            }),
            Err(_) => Err(ErrorRply::InvalidRply),
        }
    }

    pub fn code(&self) -> Code {
        self.code
    }

    pub fn err_already_registered(prefix: Option<String>) -> Self {
        Self {
            prefix,
            code: Code::ErrAlreadyregistred,
            parameters: None,
        }
    }

    ///
    /// creates an ERR_NEEDMOREPARAMS
    ///
    pub fn err_need_more_params(prefix: Option<String>, parameters: Vec<String>) -> Self {
        Self {
            prefix,
            code: Code::ErrNeedmoreparams,
            parameters: Some(parameters),
        }
    }

    ///
    /// creates an ERR_NONICKNAMEGIVEN
    ///
    pub fn err_no_nickname_given(prefix: Option<String>) -> Self {
        Self {
            prefix,
            code: Code::ErrNonicknamegiven,
            parameters: None,
        }
    }

    ///
    ///  creates an ERR_ERRONEUSNICKNAME
    ///
    pub fn err_erroneus_nickname(prefix: Option<String>, parameters: Vec<String>) -> Self {
        Self {
            prefix,
            code: Code::ErrErroneusnickname,
            parameters: Some(parameters),
        }
    }

    ///
    /// creates an ERR_NICKINUSE
    ///
    pub fn err_nickname_in_use(prefix: Option<String>, parameters: Vec<String>) -> Self {
        Self {
            prefix,
            code: Code::ErrNicknameinuse,
            parameters: Some(parameters),
        }
    }

    ///
    /// creates an ERR_NICKCOLLISION
    ///
    pub fn err_nick_collision(prefix: Option<String>, parameters: Vec<String>) -> Self {
        Self {
            prefix,
            code: Code::ErrNickcollision,
            parameters: Some(parameters),
        }
    }

    ///
    /// creates an ERR_PASSWDMISSMATCH
    ///
    pub fn err_password_missmatch(prefix: Option<String>) -> Self {
        Self {
            prefix,
            code: Code::ErrPasswdmismatch,
            parameters: None,
        }
    }

    ///
    /// creates an ERR_NOOPERHOST
    ///
    pub fn err_no_oper_host(prefix: Option<String>) -> Self {
        Self {
            prefix,
            code: Code::ErrNooperhost,
            parameters: None,
        }
    }

    ///
    /// creates an ERR_NOPRIVILEGES
    ///
    pub fn err_no_privileges(prefix: Option<String>) -> Self {
        Self {
            prefix,
            code: Code::ErrNoprivileges,
            parameters: None,
        }
    }

    ///
    /// creates an ERR_NOSUCHSERVER
    ///
    pub fn err_no_such_server(prefix: Option<String>, parameters: Vec<String>) -> Self {
        Self {
            prefix,
            code: Code::ErrNosuchserver,
            parameters: Some(parameters),
        }
    }

    ///
    /// creates an ERR_NOSUCHNICKNAME
    ///
    pub fn err_no_such_nickname(prefix: Option<String>, parameters: Vec<String>) -> Self {
        Self {
            prefix,
            code: Code::ErrNoSuchNickname,
            parameters: Some(parameters),
        }
    }

    ///
    /// creates a ERR_NORECIPIENT
    ///
    pub fn err_no_recipient(prefix: Option<String>, parameters: Vec<String>) -> Self {
        Self {
            prefix,
            code: Code::ErrNorecipient,
            parameters: Some(parameters),
        }
    }

    ///
    /// creates an ERR_NOTONCHANNEL
    ///
    pub fn err_not_on_channel(prefix: Option<String>, parameters: Vec<String>) -> Self {
        Self {
            prefix,
            code: Code::ErrNotOnChannel,
            parameters: Some(parameters),
        }
    }

    ///
    /// creates an ERR_USERONCHANNEL
    ///
    pub fn err_user_on_channel(prefix: Option<String>, parameters: Vec<String>) -> Self {
        Self {
            prefix,
            code: Code::ErrUseronchannel,
            parameters: Some(parameters),
        }
    }
    ///
    /// creates an ERR_USERONCHANNEL
    ///
    pub fn err_chan_o_privs_needed(prefix: Option<String>, parameters: Vec<String>) -> Self {
        Self {
            prefix,
            code: Code::ErrChanOPrivsNeeded,
            parameters: Some(parameters),
        }
    }

    ///
    /// creates an ERR_NOSUCHCHANNEL
    ///
    pub fn err_no_such_channel(prefix: Option<String>, parameters: Vec<String>) -> Self {
        Self {
            prefix,
            code: Code::ErrNoSuchChannel,
            parameters: Some(parameters),
        }
    }

    ///
    /// creates a RPL_NOTEXTTOSEND
    ///
    pub fn err_no_text_to_send(prefix: Option<String>) -> Self {
        Self {
            prefix,
            code: Code::ErrNoTextToSend,
            parameters: None,
        }
    }

    ///
    /// creates a RPL_YOUAREOPER
    ///
    pub fn rpl_you_are_oper(prefix: Option<String>) -> Self {
        Self {
            prefix,
            code: Code::RplYoureoper,
            parameters: None,
        }
    }

    ///
    /// creates a RPL_NONAMRPLY
    ///
    pub fn rpl_nam_rply(prefix: Option<String>, parameters: Vec<String>) -> Self {
        Self {
            prefix,
            code: Code::RplNamRply,
            parameters: Some(parameters),
        }
    }

    ///
    /// creates a RPL_NONAMRPLY
    ///
    pub fn rpl_end_of_names(prefix: Option<String>, parameters: Vec<String>) -> Self {
        Self {
            prefix,
            code: Code::RplyEndOfNames,
            parameters: Some(parameters),
        }
    }

    ///
    /// creates a RPL_LISTSTART
    ///
    pub fn rply_list_start(prefix: Option<String>) -> Self {
        Self {
            prefix,
            code: Code::RplyListStart,
            parameters: None,
        }
    }

    ///
    /// creates a RPL_LIST
    ///
    pub fn rply_list(prefix: Option<String>, parameters: Vec<String>) -> Self {
        Self {
            prefix,
            code: Code::RplyList,
            parameters: Some(parameters),
        }
    }

    ///
    /// creates a RPL_LISTSTEND
    ///
    pub fn rply_list_end(prefix: Option<String>) -> Self {
        Self {
            prefix,
            code: Code::RplyListEnd,
            parameters: None,
        }
    }

    pub fn rpl_whoischannel(params: Vec<String>) -> Self {
        Self {
            prefix: None,
            code: Code::RplyWhoIsChannel,
            parameters: Some(params),
        }
    }

    pub fn rpl_whoisuser(params: Vec<String>) -> Self {
        Self {
            prefix: None,
            code: Code::RplyWhoIsUser,
            parameters: Some(params),
        }
    }

    pub fn rpl_endwhois() -> Self {
        Self {
            prefix: None,
            code: Code::RplyEndWhois,
            parameters: None,
        }
    }

    pub fn rpl_endwho() -> Self {
        Self {
            prefix: None,
            code: Code::RplyEndWho,
            parameters: None,
        }
    }

    pub fn rpl_who(params: Vec<String>) -> Self {
        Self {
            prefix: None,
            code: Code::RplyWho,
            parameters: Some(params),
        }
    }

    pub fn rpl_inviting(channel: String, nickname: String) -> Self {
        Self {
            prefix: None,
            code: Code::RplyInviting,
            parameters: Some(vec![channel, nickname]),
        }
    }

    pub fn err_banned_from_ch(params: Vec<String>) -> Self {
        Self {
            prefix: None,
            code: Code::RplyBannedFromChan,
            parameters: Some(params),
        }
    }

    pub fn rpl_ban_list(params: Vec<String>) -> Self {
        Self {
            prefix: None,
            code: Code::RplyBanList,
            parameters: Some(params),
        }
    }

    pub fn rpl_end_ban_list(params: Vec<String>) -> Self {
        Self {
            prefix: None,
            code: Code::RplyEndBanList,
            parameters: Some(params),
        }
    }

    pub fn err_cant_send_to_channel(params: Vec<String>) -> Self {
        Self {
            prefix: None,
            code: Code::RplyCantSendChannel,
            parameters: Some(params),
        }
    }

    pub fn err_bad_chan_key(params: Vec<String>) -> Self {
        Self {
            prefix: None,
            code: Code::RplyBadChannelKey,
            parameters: Some(params),
        }
    }

    pub fn rpl_unaway() -> Self {
        Self {
            prefix: None,
            code: Code::RplyUnaway,
            parameters: None,
        }
    }

    pub fn rpl_nowaway() -> Self {
        Self {
            prefix: None,
            code: Code::RplyNowAway,
            parameters: None,
        }
    }

    pub fn rpl_away(nickname: String, away_msg: String) -> Self {
        Self {
            prefix: None,
            code: Code::RplyAway,
            parameters: Some(vec![nickname, away_msg]),
        }
    }

    pub fn rpl_topic(channel: String, topic: String) -> Self {
        Self {
            prefix: None,
            code: Code::RplyTopic,
            parameters: Some(vec![channel, topic]),
        }
    }

    pub fn rpl_no_topic(channel: String) -> Self {
        Self {
            prefix: None,
            code: Code::RplyNoTopic,
            parameters: Some(vec![channel]),
        }
    }
    pub fn err_channel_is_full(channel: String) -> Self {
        Self {
            prefix: None,
            code: Code::ErrChannelIsFull,
            parameters: Some(vec![channel]),
        }
    }
    pub fn err_invite_only_chan(channel: String) -> Self {
        Self {
            prefix: None,
            code: Code::ErrInviteOnlyChan,
            parameters: Some(vec![channel]),
        }
    }
    pub fn rply_modes(params: Vec<String>) -> Self {
        Self {
            prefix: None,
            code: Code::RplyModes,
            parameters: Some(params),
        }
    }
    pub fn err_users_dont_match() -> Self {
        Self {
            prefix: None,
            code: Code::ErrUsersDontMatch,
            parameters: None,
        }
    }
    pub fn rpl_none() -> Self {
        Self {
            prefix: None,
            code: Code::RplyNone,
            parameters: None,
        }
    }
}

impl ToString for Reply {
    fn to_string(&self) -> String {
        let mut reply = vec![];
        let prefix = self.prefix.to_owned();
        match prefix {
            None => {}
            Some(value) => reply.push(value),
        }
        reply.push(self.code.to_string());
        let parameters = self.parameters.to_owned();
        match parameters {
            None => {}
            Some(value) => reply.push(value.join(" ")),
        }
        reply.join(" ")
    }
}

impl FromStr for Reply {
    type Err = ErrorRply;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (prefix, parameters);
        let mut code_string = String::new();
        let mut split = s.trim().split(' ');

        match split.next() {
            Some(value) => {
                if value.is_empty() {
                    return Err(ErrorRply::EmptyRply);
                }
                prefix = process_prefix(value);
                match prefix {
                    None => code_string = value.to_string(),
                    Some(_) => {
                        for value in &mut split {
                            if !value.trim().is_empty() {
                                code_string = value.to_string().parse().unwrap();
                                break;
                            }
                        }
                        if code_string.is_empty() {
                            return Err(ErrorRply::NoCode);
                        }
                    }
                }
            }
            None => return Err(ErrorRply::EmptyRply),
        }
        let code: Result<i32, ParseIntError> = code_string.parse();

        let cod = match code {
            Ok(c) => c,
            Err(_) => return Err(ErrorRply::NoCode),
        };
        parameters = process_params(split);

        match Reply::new(prefix, cod, parameters) {
            Ok(reply) => Ok(reply),
            Err(_) => Err(ErrorRply::InvalidRply),
        }
    }
}

fn process_prefix(s: &str) -> Option<String> {
    if s.as_bytes()[0] == COLON_U8 {
        return Some(s.to_string());
    }
    None
}

fn process_params(mut split: Split<char>) -> Option<Vec<String>> {
    let mut params = vec![];
    for s in &mut split {
        if !s.trim().is_empty() {
            if s.as_bytes()[0] == COLON_U8 {
                //En caso de existir un parametro con ":"
                let mut str = s.to_owned() + " ";
                str.push_str(&split.collect::<Vec<&str>>().join(" "));
                params.push(str);
                return Some(params);
            } else {
                params.push(s.to_string())
            }
        }
    }

    if params.is_empty() {
        return None;
    }

    Some(params)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn reply_exists_for_invalid_nickname() {
        let parameters = Some(vec![String::from("nickname")]);
        let reply = Reply::new(None, 431, parameters);

        assert!(reply.is_ok())
    }

    #[test]
    fn code_to_string() {
        let reply = Reply::err_nickname_in_use(None, vec!["NICK".to_string()]);
        let string = reply.to_string();
        assert_eq!(string, "433 NICK")
    }

    #[test]
    fn code_is_invalid() {
        assert!(Reply::new(None, 2, None).is_err())
    }

    #[test]
    fn code_store_valid() {
        let parameters = Some(vec![String::from("nickname")]);
        let reply = Reply::new(None, 431, parameters).unwrap();
        assert_eq!(431, reply.code as i32);
    }

    #[test]
    fn reply_as_string() {
        let reply = Reply::err_already_registered(None);
        let result = reply.to_string();
        let expected = "462".to_string();
        assert_eq!(result, expected)
    }

    #[test]
    fn from_string_correctly() {
        let reply = Reply::from_str("462").unwrap();
        let expected = Reply::err_already_registered(None);
        assert_eq!(expected.code, reply.code)
    }

    #[test]
    fn from_string_wrong() {
        let reply = Reply::from_str("");
        assert_eq!(reply, Err(ErrorRply::EmptyRply));
    }

    #[test]
    fn no_recipient() {
        let reply = Reply::err_no_recipient(None, vec!["SERVER".to_string()]);
        let code = 411;
        assert_eq!(code, reply.code as i32)
    }

    #[test]
    fn err_not_on_channel() {
        let reply = Reply::err_not_on_channel(None, vec![" ".to_string()]);
        let code = 442;
        assert_eq!(code, reply.code as i32)
    }

    #[test]
    fn err_user_on_channel() {
        let reply = Reply::err_user_on_channel(None, vec![" ".to_string()]);
        let code = 443;
        assert_eq!(code, reply.code as i32)
    }

    #[test]
    fn err_chan_o_privs_needed() {
        let reply = Reply::err_chan_o_privs_needed(None, vec![" ".to_string()]);
        let code = 482;
        assert_eq!(code, reply.code as i32)
    }

    #[test]
    fn err_no_such_channel() {
        let reply = Reply::err_no_such_channel(None, vec![" ".to_string()]);
        let code = 403;
        assert_eq!(code, reply.code as i32)
    }

    #[test]
    fn err_no_text_to_send() {
        let reply = Reply::err_no_text_to_send(None);
        let code = 412;
        assert_eq!(code, reply.code as i32)
    }

    #[test]
    fn rpl_nam_rply() {
        let reply = Reply::rpl_nam_rply(None, vec![" ".to_string()]);
        let code = 353;
        assert_eq!(code, reply.code as i32)
    }

    #[test]
    fn rpl_end_of_names() {
        let reply = Reply::rpl_end_of_names(None, vec![" ".to_string()]);
        let code = 366;
        assert_eq!(code, reply.code as i32)
    }

    #[test]
    fn rply_list_start() {
        let reply = Reply::rply_list_start(None);
        let code = 321;
        assert_eq!(code, reply.code as i32)
    }

    #[test]
    fn rply_list() {
        let reply = Reply::rply_list(None, vec![" ".to_string()]);
        let code = 322;
        assert_eq!(code, reply.code as i32)
    }

    #[test]
    fn rply_list_end() {
        let reply = Reply::rply_list_end(None);
        let code = 323;
        assert_eq!(code, reply.code as i32)
    }

    #[test]
    fn rpl_whoischannel() {
        let reply = Reply::rpl_whoischannel(vec![" ".to_string()]);
        let code = 319;
        assert_eq!(code, reply.code as i32)
    }

    #[test]
    fn rpl_endwhois() {
        let reply = Reply::rpl_endwhois();
        let code = 318;
        assert_eq!(code, reply.code as i32)
    }

    #[test]
    fn rpl_inviting() {
        let reply = Reply::rpl_inviting(" ".to_string(), " ".to_string());
        let code = 341;
        assert_eq!(code, reply.code as i32)
    }

    #[test]
    fn err_banned_from_ch() {
        let reply = Reply::err_banned_from_ch(vec![" ".to_string()]);
        let code = 474;
        assert_eq!(code, reply.code as i32)
    }

    #[test]
    fn rpl_ban_list() {
        let reply = Reply::rpl_ban_list(vec![" ".to_string()]);
        let code = 367;
        assert_eq!(code, reply.code as i32)
    }

    #[test]
    fn rpl_end_ban_list() {
        let reply = Reply::rpl_end_ban_list(vec![" ".to_string()]);
        let code = 368;
        assert_eq!(code, reply.code as i32)
    }

    #[test]
    fn err_cant_send_to_channel() {
        let reply = Reply::err_cant_send_to_channel(vec![" ".to_string()]);
        let code = 404;
        assert_eq!(code, reply.code as i32)
    }

    #[test]
    fn err_bad_chan_key() {
        let reply = Reply::err_bad_chan_key(vec![" ".to_string()]);
        let code = 475;
        assert_eq!(code, reply.code as i32)
    }

    #[test]
    fn rpl_nowaway() {
        let reply = Reply::rpl_nowaway();
        let code = 306;
        assert_eq!(code, reply.code as i32)
    }

    #[test]
    fn rpl_away() {
        let reply = Reply::rpl_away(" ".to_string(), " ".to_string());
        let code = 301;
        assert_eq!(code, reply.code as i32)
    }

    #[test]
    fn rpl_topic() {
        let reply = Reply::rpl_topic(" ".to_string(), " ".to_string());
        let code = 332;
        assert_eq!(code, reply.code as i32)
    }

    #[test]
    fn rpl_no_topic() {
        let reply = Reply::rpl_no_topic(" ".to_string());
        let code = 331;
        assert_eq!(code, reply.code as i32)
    }

    #[test]
    fn err_channel_is_full() {
        let reply = Reply::err_channel_is_full(" ".to_string());
        let code = 471;
        assert_eq!(code, reply.code as i32)
    }

    #[test]
    fn err_invite_only_chan() {
        let reply = Reply::err_invite_only_chan(" ".to_string());
        let code = 473;
        assert_eq!(code, reply.code as i32)
    }

    #[test]
    fn rply_modes() {
        let reply = Reply::rply_modes(vec![" ".to_string()]);
        let code = 324;
        assert_eq!(code, reply.code as i32)
    }

    #[test]
    fn err_users_dont_match() {
        let reply = Reply::err_users_dont_match();
        let code = 502;
        assert_eq!(code, reply.code as i32)
    }

    #[test]
    fn rpl_none() {
        let reply = Reply::rpl_none();
        let code = 300;
        assert_eq!(code, reply.code as i32)
    }

    #[test]
    fn rpl_whoisuser() {
        let reply = Reply::rpl_whoisuser(vec![" ".to_string()]);
        let code = 311;
        assert_eq!(code, reply.code as i32)
    }

    #[test]
    fn rpl_unaway() {
        let reply = Reply::rpl_unaway();
        let code = 305;
        assert_eq!(code, reply.code as i32)
    }

    #[test]
    fn code_test() {
        let reply = Reply::rpl_unaway();
        let code = 305;
        assert_eq!(code, reply.code() as i32)
    }

    #[test]
    fn debugs_correctly() {
        let reply = Reply::rpl_unaway();
        assert_eq!(
            format!("{reply:?}"),
            "Reply { prefix: None, code: RplyUnaway, parameters: None }"
        )
    }

    #[test]
    fn partial_equation() {
        let rep1 = Reply::rpl_unaway();
        let rep2 = Reply::rpl_unaway();

        let result = rep1.eq(&rep2);
        assert!(result)
    }
}
