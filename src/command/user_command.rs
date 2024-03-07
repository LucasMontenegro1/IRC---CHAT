use crate::command::traits::RegistrationCommand;
use crate::command::Command;
use crate::parser::message::Message;
use crate::reply::Reply;
use crate::user::builder::UserBuilder;

pub struct UserCommand {
    username: String,
    hostname: String,
    servername: String,
    realname: String,
}

///
/// implementation of the registration
/// trait for the user
///
impl RegistrationCommand for UserCommand {
    fn register_user(&self, builder: &UserBuilder) -> UserBuilder {
        let mut user = UserBuilder::new();
        user.load(builder);
        user.username(&self.username);
        user.hostname(&self.hostname);
        user.servername(&self.servername);
        user.realname(&self.realname);
        user
    }
}

impl UserCommand {
    pub fn new_registration_command(msg: Message) -> Result<Box<dyn RegistrationCommand>, Reply> {
        let username: String;
        let hostname: String;
        let servername = String::new();
        let realname: String;
        if let Some(params) = &msg.parameters() {
            let mut params = params.iter();
            if let Some(str) = params.next() {
                username = str.to_string();
                if let Some(str) = params.next() {
                    hostname = str.to_string();
                    if let Some(_servername) = params.next() {
                        if let Some(str) = params.next() {
                            realname = str.to_string();
                            return Ok(Box::new(UserCommand {
                                username,
                                hostname,
                                servername,
                                realname,
                            }));
                        }
                    }
                }
            }
        }
        Err(Reply::err_need_more_params(
            None,
            vec![Command::User.to_string()],
        ))
    }
}
