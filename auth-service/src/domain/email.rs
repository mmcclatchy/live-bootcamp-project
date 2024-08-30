use std::{borrow::Cow, hash::Hash};

use secrecy::{ExposeSecret, Secret};
use serde::{ser::SerializeStruct, Serialize};
use validator::ValidateEmail;

use macros::SecretString;

#[derive(Clone, Debug, SecretString)]
pub struct Email(Secret<String>);

impl Email {
    pub fn parse(email: Secret<String>) -> Result<Self, String> {
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
        Some(Cow::Borrowed(self.0.expose_secret()))
    }
}

impl Hash for Email {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.expose_secret().hash(state)
    }
}

impl Eq for Email {}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::{faker::internet::en::SafeEmail, Fake, Faker};
    use quickcheck::TestResult;
    use quickcheck_macros::quickcheck;

    fn string_to_email_result(email: String) -> Result<Email, String> {
        Email::parse(Secret::new(email))
    }

    #[test]
    fn test_valid_email_with_fake() {
        let email: String = SafeEmail().fake();
        let result = string_to_email_result(email.clone());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().expose_secret_string(), email);
    }

    #[test]
    fn test_invalid_email_with_fake() {
        let invalid_email: String = Faker.fake();
        let result = string_to_email_result(invalid_email);
        assert!(result.is_err());
    }

    #[quickcheck]
    fn quickcheck_email_validation(email: String) -> TestResult {
        let email_result = string_to_email_result(email.clone());
        match email_result {
            Err(err_msg) => TestResult::from_bool(err_msg == "Invalid email address"),
            Ok(email_instance) => {
                TestResult::from_bool(email_instance.validate_email() == Email::parse(Secret::new(email)).is_ok())
            }
        }
    }

    #[test]
    fn test_as_ref() {
        let email_string = "test@email.com".to_string();
        let email = string_to_email_result(email_string.clone()).unwrap();
        assert_eq!(email.expose_secret_string(), email_string);
    }
}
