use crate::error::error_user::ErrorUser;
use crate::user::User;

///
/// struct that implements
/// the user builder, this entity
/// is responsible for building the
/// users that are registered on the server
///
///
#[derive(Debug, PartialEq, Eq)]
pub struct UserBuilder {
    nickname: String,
    username: String,
    hostname: String,
    servername: String,
    realname: String,
    password: String,
}

///
/// implements the trait default for
/// the UserBuilder
///
impl std::default::Default for UserBuilder {
    fn default() -> Self {
        UserBuilder::new()
    }
}

impl UserBuilder {
    ///
    /// function that builds a
    /// new user returning
    /// the respective user
    ///
    pub fn build(&self) -> Result<User, ErrorUser> {
        if self.nickname.is_empty()
            || self.username.is_empty()
            || self.hostname.is_empty()
            || self.servername.is_empty()
            || self.realname.is_empty()
        {
            return Err(ErrorUser::BuildError);
        }
        Ok(User::new(
            &self.nickname,
            &self.username,
            &self.hostname,
            &self.servername,
            &self.realname,
            &self.password,
        ))
    }
    ///
    /// function that creates a
    /// new user builder.
    ///
    pub fn new() -> Self {
        UserBuilder {
            nickname: String::new(),
            username: String::new(),
            hostname: String::new(),
            servername: String::new(),
            realname: String::new(),
            password: String::new(),
        }
    }

    //  Since the password MUST be entered before the User attributes,
    // setting the passsword "resets" the UserBuilder attributes
    pub fn password(pwd: &str) -> Self {
        UserBuilder {
            nickname: String::new(),
            username: String::new(),
            hostname: String::new(),
            servername: String::new(),
            realname: String::new(),
            password: pwd.to_string(),
        }
    }

    pub fn load(&mut self, user: &UserBuilder) {
        if !user.nickname.is_empty() {
            self.nickname(&user.nickname);
        }
        if !user.username.is_empty() {
            self.username(&user.username);
        }
        if !user.hostname.is_empty() {
            self.hostname(&user.hostname);
        }
        if !user.servername.is_empty() {
            self.servername(&user.servername);
        }
        if !user.realname.is_empty() {
            self.realname(&user.realname);
        }
        if !user.password.is_empty() {
            self.password = user.password.to_string();
        }
    }

    pub fn load_from_user(&mut self, user: &User) {
        if let Some(str) = user.nickname() {
            self.nickname(str);
        }
        if let Some(str) = user.username() {
            self.username(str);
        }
        if let Some(str) = user.hostname() {
            self.hostname(str);
        }
        if let Some(str) = user.servername() {
            self.servername(str);
        }
        if let Some(str) = user.realname() {
            self.realname(str);
        }
        // No se que tan necesario va a ser que el usuario cargue la contraseña
    }

    pub fn nickname(&mut self, nickname: &str) {
        self.nickname = String::from(nickname);
    }

    pub fn username(&mut self, username: &str) {
        self.username = String::from(username);
    }

    pub fn hostname(&mut self, hostname: &str) {
        self.hostname = String::from(hostname);
    }

    pub fn servername(&mut self, servername: &str) {
        self.servername = String::from(servername);
    }

    pub fn realname(&mut self, realname: &str) {
        self.realname = String::from(realname);
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn new_user_builder() {
        let expected = UserBuilder {
            nickname: String::new(),
            hostname: String::new(),
            username: String::new(),
            servername: String::new(),
            realname: String::new(),
            password: String::new(),
        };
        let result = UserBuilder::new();
        assert_eq!(result, expected);
    }

    #[test]
    fn new_user_builder_with_password() {
        let expected = UserBuilder {
            nickname: String::new(),
            hostname: String::new(),
            username: String::new(),
            servername: String::new(),
            realname: String::new(),
            password: String::from("secretpasswordhere"),
        };
        let result = UserBuilder::password("secretpasswordhere");
        assert_eq!(result, expected);
    }

    #[test]
    fn load_nickname() {
        let expected = UserBuilder {
            nickname: String::from("Wiz"),
            hostname: String::new(),
            username: String::new(),
            servername: String::new(),
            realname: String::new(),
            password: String::new(),
        };
        let mut user = UserBuilder::new();
        user.nickname("Wiz");

        let mut result = UserBuilder::new();
        result.load(&user);

        assert_eq!(result, expected);
    }

    #[test]
    fn load_username() {
        let expected = UserBuilder {
            nickname: String::new(),
            username: String::from("guest"),
            hostname: String::new(),
            servername: String::new(),
            realname: String::new(),
            password: String::new(),
        };
        let mut user = UserBuilder::new();
        user.username("guest");

        let mut result = UserBuilder::new();
        result.load(&user);

        assert_eq!(result, expected);
    }

    #[test]
    fn load_hostname() {
        let expected = UserBuilder {
            nickname: String::new(),
            username: String::new(),
            hostname: String::from("tolmoon"),
            servername: String::new(),
            realname: String::new(),
            password: String::new(),
        };
        let mut user = UserBuilder::new();
        user.hostname("tolmoon");

        let mut result = UserBuilder::new();
        result.load(&user);

        assert_eq!(result, expected);
    }

    #[test]
    fn load_servername() {
        let expected = UserBuilder {
            nickname: String::new(),
            username: String::new(),
            hostname: String::new(),
            servername: String::from("tolsun"),
            realname: String::new(),
            password: String::new(),
        };
        let mut user = UserBuilder::new();
        user.servername("tolsun");

        let mut result = UserBuilder::new();
        result.load(&user);

        assert_eq!(result, expected);
    }

    #[test]
    fn load_realname() {
        let expected = UserBuilder {
            nickname: String::new(),
            username: String::new(),
            hostname: String::new(),
            servername: String::new(),
            realname: String::from("Ronnie Reagan"),
            password: String::new(),
        };
        let mut user = UserBuilder::new();
        user.realname("Ronnie Reagan");

        let mut result = UserBuilder::new();
        result.load(&user);

        assert_eq!(result, expected);
    }

    #[test]
    fn load_from_builder() {
        let expected = UserBuilder {
            nickname: String::from("Wiz"),
            username: String::from("guest"),
            hostname: String::from("tolmoon"),
            servername: String::from("tolsun"),
            realname: String::from("Ronnie Reagan"),
            password: String::from(""),
        };
        let mut user = UserBuilder::new();
        user.username("guest");
        user.hostname("tolmoon");
        user.servername("tolsun");
        user.realname("Ronnie Reagan");

        let mut result = UserBuilder::new();
        result.nickname("Wiz");
        result.load(&user);

        assert_eq!(result, expected);
    }
    #[test]
    fn load_with_password() {
        let expected = UserBuilder {
            nickname: String::from("Wiz"),
            username: String::from("guest"),
            hostname: String::from("tolmoon"),
            servername: String::from("tolsun"),
            realname: String::from("Ronnie Reagan"),
            password: String::from("secretpasswordhere"),
        };
        let mut user = UserBuilder::password("secretpasswordhere");
        user.username("guest");
        user.hostname("tolmoon");
        user.servername("tolsun");
        user.realname("Ronnie Reagan");

        let mut result = UserBuilder::new();
        result.nickname("Wiz");
        result.load(&user);

        assert_eq!(result, expected);
    }

    #[test]
    fn load_user() {
        let expected = UserBuilder {
            nickname: String::from("Wiz"),
            username: String::from("guest"),
            hostname: String::from("tolmoon"),
            servername: String::from("tolsun"),
            realname: String::from("Ronnie Reagan"),
            password: String::new(),
        };
        let user = User::new("Wiz", "guest", "tolmoon", "tolsun", "Ronnie Reagan", "");

        let mut result = UserBuilder::new();
        result.load_from_user(&user);

        assert_eq!(result, expected);
    }

    #[test]
    fn load_user_with_password() {
        let not_expected = UserBuilder {
            nickname: String::from("Wiz"),
            username: String::from("guest"),
            hostname: String::from("tolmoon"),
            servername: String::from("tolsun"),
            realname: String::from("Ronnie Reagan"),
            password: String::from("secretpasswordhere"),
        };
        let user = User::new(
            "Wiz",
            "guest",
            "tolmoon",
            "tolsun",
            "Ronnie Reagan",
            "secretpasswordhere",
        );

        let mut result = UserBuilder::new();
        result.load_from_user(&user);

        assert_ne!(result, not_expected);

        let expected = UserBuilder {
            nickname: String::from("Wiz"),
            username: String::from("guest"),
            hostname: String::from("tolmoon"),
            servername: String::from("tolsun"),
            realname: String::from("Ronnie Reagan"),
            password: String::new(),
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn build_user() {
        let expected = User {
            nickname: String::from("Wiz"),
            username: String::from("guest"),
            hostname: String::from("tolmoon"),
            servername: String::from("tolsun"),
            realname: String::from("Ronnie Reagan"),
            password: None,
            oper: false,
            user_flags: Vec::new(),
        };
        let mut user = UserBuilder::new();
        assert!(user.build().is_err());

        user.nickname("Wiz");
        assert!(user.build().is_err());

        user.username("guest");
        assert!(user.build().is_err());

        user.hostname("tolmoon");
        assert!(user.build().is_err());

        user.servername("tolsun");
        assert!(user.build().is_err());

        user.realname("Ronnie Reagan");
        let result = user.build().unwrap();
        assert_eq!(result, expected);
    }
}
