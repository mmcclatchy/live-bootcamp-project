use crate::domain::{email::Email, password::Password};

#[derive(Clone, Debug, PartialEq)]
pub struct User {
    pub email: Email,
    pub password: Password,
    pub requires_2fa: bool,
}

impl User {
    #[allow(dead_code)]
    pub fn new(email: Email, password: Password, requires_2fa: bool) -> Self {
        User {
            email,
            password,
            requires_2fa,
        }
    }

    pub fn update_password(&mut self, password: Password) {
        self.password = password;
    }
}
