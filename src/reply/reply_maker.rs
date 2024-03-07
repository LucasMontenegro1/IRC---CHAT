use super::Reply;

///
/// function that converts a reply into a printable format
/// that matches the one given by the irc rfc protocol
///
///
pub fn make_reply_format(reply: Reply) -> String {
    let mut result = String::new();
    let prefix = match reply.prefix.clone() {
        Some(c) => c,
        None => "".to_string(),
    };
    result.push_str(prefix.as_str());
    let parameters = match reply.parameters.clone() {
        Some(c) => c,
        None => vec!["".to_string()],
    };

    match reply.code {
        super::code::Code::ErrNeedmoreparams => {
            result.push(' ');
            result.push_str(parameters.concat().as_str());
            result.push_str(":Not enough parameters");
        }
        super::code::Code::ErrAlreadyregistred => {
            result.push_str(":You may not register");
        }
        super::code::Code::ErrNonicknamegiven => {
            result.push_str(":No nickname given");
        }
        super::code::Code::ErrErroneusnickname => {
            result.push(' ');
            result.push_str(parameters.concat().as_str());
            result.push_str(":Erroneus nickname");
        }
        super::code::Code::ErrNicknameinuse => {
            result.push(' ');
            result.push_str(parameters.concat().as_str());
            result.push_str(":Nickname is already in use");
        }
        super::code::Code::ErrNickcollision => {
            result.push(' ');
            result.push_str(parameters.concat().as_str());
            result.push_str(":Nickname collision KILL");
        }
        super::code::Code::ErrPasswdmismatch => {
            result.push_str(":Password incorrect");
        }
        super::code::Code::ErrNooperhost => {
            result.push_str(":No O-lines for your host");
        }
        super::code::Code::ErrNoprivileges => {
            result.push_str(":Permission Denied- You're not an IRC operator");
        }
        super::code::Code::ErrNosuchserver => {
            result.push(' ');
            result.push_str(parameters.concat().as_str());
            result.push_str(":No such server");
        }
        super::code::Code::RplYoureoper => {
            result.push_str(":You are now an IRC operator");
        }
        super::code::Code::ErrNoSuchNickname => {
            result.push(' ');
            result.push_str(parameters.concat().as_str());
            result.push_str(": no such nick/channel");
        }
        super::code::Code::ErrNorecipient => {
            result.push(' ');
            result.push_str(": no recipient given ");
            result.push_str(parameters.concat().as_str());
        }
        super::code::Code::ErrNotOnChannel => {
            result.push(' ');
            result.push_str(parameters.concat().as_str());
            result.push_str(": you are not on that channel ");
        }
        super::code::Code::ErrUseronchannel => {
            result.push(' ');
            result.push_str(parameters.concat().as_str());
            result.push_str(": is already on channel ");
        }
        super::code::Code::ErrChanOPrivsNeeded => {
            result.push(' ');
            result.push_str(parameters.concat().as_str());
            result.push_str(": You're not channel operator ");
        }
        super::code::Code::ErrNoSuchChannel => {
            result.push(' ');
            result.push_str(parameters.concat().as_str());
            result.push_str(": No such channel ");
        }
        super::code::Code::RplNamRply => {
            result.push(' ');
            result.push_str(parameters.concat().as_str());
        }
        super::code::Code::RplyEndOfNames => {
            result.push(' ');
            result.push_str(parameters.concat().as_str());
            result.push_str(": End of /NAMES list");
        }
        super::code::Code::RplyListStart => {
            result.push_str("Channel :Users  Name");
        }
        super::code::Code::RplyListEnd => {
            result.push_str(":End of /LIST");
        }
        super::code::Code::RplyList => {
            result.push(' ');
            result.push_str(parameters.concat().as_str());
        }
        super::code::Code::RplyWhoIsChannel => {
            result.push(':');
            result.push_str(parameters.concat().as_str());
        }
        super::code::Code::RplyWhoIsUser => {
            result.push_str(&parameters.join(" "));
        }
        super::code::Code::RplyEndWhois => {
            result.push_str(":End of /WHOIS list");
        }
        super::code::Code::RplyWho => {
            result.push_str(&parameters.join(" "));
        }
        super::code::Code::RplyEndWho => {
            result.push_str(":End of /WHO list");
        }
        super::code::Code::RplyInviting => {
            result.push_str(&parameters.join(" "));
        }
        super::code::Code::RplyBannedFromChan => {
            result.push_str(&parameters[0]);
            result.push_str(":Cannot join channel (+b)");
        }
        super::code::Code::ErrInviteOnlyChan => {
            result.push_str(parameters.concat().as_str());
            result.push_str(":Cannot join channel (+i)");
        }
        super::code::Code::ErrChannelIsFull => {
            result.push_str(parameters.concat().as_str());
            result.push_str(":Cannot join channel (+l)");
        }
        super::code::Code::RplyBanList => {
            result.push(' ');
            result.push_str(parameters.concat().as_str());
        }
        super::code::Code::RplyEndBanList => {
            result.push_str(parameters.concat().as_str());
            result.push_str(" :End of channel ban list");
        }
        super::code::Code::RplyCantSendChannel => {
            result.push_str(parameters.concat().as_str());
            result.push_str(" :Cannot send to channel");
        }
        super::code::Code::RplyBadChannelKey => {
            result.push_str(parameters.concat().as_str());
            result.push_str(" :Cannot join channel (+k)");
        }
        super::code::Code::RplyAway => {
            result.push_str(&parameters.join(" "));
        }
        super::code::Code::RplyNowAway => {
            result.push_str(" :You have been marked as being away");
        }
        super::code::Code::RplyUnaway => {
            result.push_str(" :You are no longer marked as being away");
        }
        super::code::Code::RplyTopic => {
            result.push_str(&parameters.join(" "));
        }
        super::code::Code::RplyNoTopic => {
            result.push_str(parameters.concat().as_str());
            result.push_str(" :No topic is set");
        }
        super::code::Code::RplyModes => {
            result.push_str(parameters.concat().as_str());
        }
        super::code::Code::ErrUsersDontMatch => {
            result.push_str(":Cant change mode for other users");
        }
        super::code::Code::ErrNoTextToSend => {
            result.push_str(":No text to send");
        }
        super::code::Code::RplyNone => {}
    };
    result
}

#[cfg(test)]
mod test {
    use crate::reply::Reply;

    use super::make_reply_format;

    #[test]
    fn reply_already_registered_with_no_server() {
        let reply = Reply::err_already_registered(None);
        let result = make_reply_format(reply);
        let expected = ":You may not register".to_string();
        assert_eq!(result, expected);
    }
    #[test]
    fn reply_with_server() {
        let reply = Reply::err_already_registered(Some("server".to_string()));
        let result = make_reply_format(reply);
        let expected = "server:You may not register".to_string();
        assert_eq!(result, expected);
    }

    #[test]
    fn no_nickname_given() {
        let reply = Reply::err_no_nickname_given(None);
        let result = make_reply_format(reply);
        let expected = ":No nickname given".to_string();
        assert_eq!(result, expected);
    }

    #[test]
    fn no_enough_parameters() {
        let reply = Reply::err_need_more_params(None, vec!["NICK".to_string()]);
        let result = make_reply_format(reply);
        let expected = " NICK:Not enough parameters".to_string();
        assert_eq!(result, expected);
    }

    #[test]
    fn no_enough_parameters_with_server() {
        let reply =
            Reply::err_need_more_params(Some("server".to_string()), vec!["NICK".to_string()]);
        let result = make_reply_format(reply);
        let expected = "server NICK:Not enough parameters".to_string();
        assert_eq!(result, expected);
    }

    #[test]
    fn err_erroneus_nickname() {
        let reply = Reply::err_erroneus_nickname(None, vec!["NICK".to_string()]);
        let result = make_reply_format(reply);
        let expected = " NICK:Erroneus nickname".to_string();
        assert_eq!(result, expected);
    }

    #[test]
    fn err_nickname_in_use() {
        let reply = Reply::err_nickname_in_use(None, vec!["NICK".to_string()]);
        let result = make_reply_format(reply);
        let expected = " NICK:Nickname is already in use".to_string();
        assert_eq!(result, expected);
    }

    #[test]
    fn err_nick_collision() {
        let reply = Reply::err_nick_collision(None, vec!["NICK".to_string()]);
        let result = make_reply_format(reply);
        let expected = " NICK:Nickname collision KILL".to_string();
        assert_eq!(result, expected);
    }
    #[test]
    fn err_password_missmatch() {
        let reply = Reply::err_password_missmatch(None);
        let result = make_reply_format(reply);
        let expected = ":Password incorrect".to_string();
        assert_eq!(result, expected);
    }

    #[test]
    fn err_no_oper_host() {
        let reply = Reply::err_no_oper_host(None);
        let result = make_reply_format(reply);
        let expected = ":No O-lines for your host".to_string();
        assert_eq!(result, expected);
    }

    #[test]
    fn err_no_privileges() {
        let reply = Reply::err_no_privileges(None);
        let result = make_reply_format(reply);
        let expected = ":Permission Denied- You're not an IRC operator".to_string();
        assert_eq!(result, expected);
    }

    #[test]
    fn err_no_such_server() {
        let reply = Reply::err_no_such_server(None, vec!["SERVERNAME".to_string()]);
        let result = make_reply_format(reply);
        let expected = " SERVERNAME:No such server".to_string();
        assert_eq!(result, expected);
    }

    #[test]
    fn err_no_such_nickname() {
        let reply = Reply::err_no_such_nickname(None, vec!["Nick".to_string()]);
        let result = make_reply_format(reply);
        let expected = " Nick: no such nick/channel".to_string();
        assert_eq!(result, expected);
    }

    #[test]
    fn rpl_you_are_oper() {
        let reply = Reply::rpl_you_are_oper(None);
        let result = make_reply_format(reply);
        let expected = ":You are now an IRC operator".to_string();
        assert_eq!(result, expected);
    }
}
