use crate::command::traits::RegistrationCommand;
use crate::command::Command;
use crate::parser::message::Message;
use crate::reply::Reply;
use crate::user::builder::UserBuilder;

pub struct PassCommand {
    password: String,
}

///
/// implementation of the RegisterCommand trait
///
impl RegistrationCommand for PassCommand {
    fn register_user(&self, _builder: &UserBuilder) -> UserBuilder {
        UserBuilder::password(&self.password.to_string())
    }
}

impl PassCommand {
    pub fn new_registration_command(msg: Message) -> Result<Box<dyn RegistrationCommand>, Reply> {
        Ok(Box::new(PassCommand {
            password: Self::get_password_from_msg(msg)?,
        }))
    }

    fn get_password_from_msg(msg: Message) -> Result<String, Reply> {
        match msg.parameters() {
            Some(pwd) => Ok(pwd[0].clone()),
            None => Err(Reply::err_need_more_params(
                None,
                vec![Command::Pass.to_string()],
            )),
        }
    }
}
