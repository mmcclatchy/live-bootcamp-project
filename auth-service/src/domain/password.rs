use std::fmt;

const ERROR_MESSAGE: &str = "Invalid Password: Must be at least 8 characters long, contain at least one uppercase character and one number";

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Password(String);

impl Password {
    pub fn parse(password: String) -> Result<Self, String> {
        if password.len() < 8
            || !password.chars().any(|c| c.is_uppercase())
            || !password.chars().any(|c| c.is_numeric())
        {
            return Err(ERROR_MESSAGE.to_string());
        }
        Ok(Password(password))
    }
}

impl AsRef<str> for Password {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Password {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_password() {
        let password = "P@ssw0rd".to_string();
        let result = Password::parse(password.clone());
        assert!(result.is_ok());
        let password_struct = result.unwrap();
        assert_eq!(password_struct.as_ref(), password);
    }

    #[test]
    fn test_password_too_short() {
        let password = "P@ss".to_string();
        let result = Password::parse(password);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ERROR_MESSAGE);
    }

    #[test]
    fn test_password_no_uppercase() {
        let password = "p@ssw0rd".to_string();
        let result = Password::parse(password);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ERROR_MESSAGE);
    }

    // #[test]
    // fn test_password_no_symbol() {
    //     let password = "Password123".to_string();
    //     let result = Password::parse(password);
    //     assert!(result.is_err());
    //     assert_eq!(result.unwrap_err(), ERROR_MESSAGE);
    // }

    #[test]
    fn test_password_all_requirements_failed() {
        let password = "password".to_string();
        let result = Password::parse(password);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ERROR_MESSAGE);
    }

    #[test]
    fn test_password_display() {
        let password = "P@ssw0rd".to_string();
        let password_struct = Password::parse(password.clone()).unwrap();
        assert_eq!(format!("{}", password_struct), password);
    }

    #[test]
    fn test_password_as_ref() {
        let password = "P@ssw0rd".to_string();
        let password_struct = Password::parse(password.clone()).unwrap();
        assert_eq!(password_struct.as_ref(), password);
    }
}
