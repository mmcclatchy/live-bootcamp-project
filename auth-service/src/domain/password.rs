use color_eyre::eyre::{eyre, Result};

use secrecy::{ExposeSecret, Secret};

const ERROR_MESSAGE: &str =
    "Invalid Password: Must be at least 8 characters long, contain at least one uppercase character and one number";

#[derive(Debug, Clone)]
pub struct Password(Secret<String>);

impl Password {
    pub async fn parse(s: Secret<String>) -> Result<Self> {
        if s.expose_secret().len() < 8
            || !s.expose_secret().chars().any(|c| c.is_uppercase())
            || !s.expose_secret().chars().any(|c| c.is_numeric())
        {
            return Err(eyre!(ERROR_MESSAGE.to_string()));
        }
        Ok(Self(s))
    }
}

impl AsRef<Secret<String>> for Password {
    fn as_ref(&self) -> &Secret<String> {
        &self.0
    }
}

impl PartialEq for Password {
    fn eq(&self, other: &Self) -> bool {
        self.0.expose_secret() == other.0.expose_secret()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_valid_password() {
        let password = Secret::new("P@ssw0rd".to_string());
        let result = Password::parse(password.clone()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_password_too_short() {
        let password = Secret::new("P@ss".to_string());
        let result = Password::parse(password).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_password_no_uppercase() {
        let password = Secret::new("p@ssw0rd".to_string());
        let result = Password::parse(password).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_password_all_requirements_failed() {
        let password = Secret::new("password".to_string());
        let result = Password::parse(password).await;
        assert!(result.is_err());
    }
}
