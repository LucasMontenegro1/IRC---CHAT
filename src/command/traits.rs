use crate::error::error_server::ErrorServer;
use crate::parser::message::Message;
use crate::user::builder::UserBuilder;
use std::net::TcpStream;

///
/// trait RegistrationCommand
///
pub trait RegistrationCommand {
    fn register_user(&self, builder: &UserBuilder) -> UserBuilder;
    //fn new_registration_command(msg: Message) -> Result<Box<dyn RegistrationCommand>, Reply>;
}

///
/// trait response
///
pub trait Response {
    fn response(msg: &Message);
}

pub trait Runnable {
    fn run(&mut self, socket: &mut TcpStream) -> Result<(), ErrorServer>;
}
