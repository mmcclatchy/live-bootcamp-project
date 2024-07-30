use std::borrow::Cow;
use std::fmt;
use validator::ValidateEmail;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Email(String);

impl Email {
    pub fn parse(email: String) -> Result<Self, String> {
        let email_instance = Self(email);
        if email_instance.validate_email() {
            Ok(email_instance)
        } else {
            Err("Invalid email address".to_string())
        }
    }
}

impl ValidateEmail for Email {
    fn as_email_string(&self) -> Option<Cow<'_, str>> {
        Some(Cow::Borrowed(&self.0))
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Email {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::{faker::internet::en::SafeEmail, Fake, Faker};
    use quickcheck::TestResult;
    use quickcheck_macros::quickcheck;

    #[test]
    fn test_valid_email_with_fake() {
        let email: String = SafeEmail().fake();
        let result = Email::parse(email.clone());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string(), email);
    }

    #[test]
    fn test_invalid_email_with_fake() {
        let invalid_email: String = Faker.fake();
        let result = Email::parse(invalid_email);
        assert!(result.is_err());
    }

    #[quickcheck]
    fn quickcheck_email_validation(email: String) -> TestResult {
        let email_instance = Email(email.clone());
        TestResult::from_bool(email_instance.validate_email() == Email::parse(email).is_ok())
    }

    #[test]
    fn test_as_ref() {
        let email_string = "test@email.com".to_string();
        let email = Email::parse(email_string.clone()).unwrap();
        assert_eq!(email.as_ref(), email_string);
    }

    #[test]
    fn test_display_implementation() {
        let email_string = "test@email.com".to_string();
        let email = Email::parse(email_string.clone()).unwrap();
        assert_eq!(format!("{}", email), email_string);
    }
}
