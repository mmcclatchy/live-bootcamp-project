use std::collections::HashMap;

use crate::domain::{
    data_stores::{LoginAttemptId, TwoFACode, TwoFACodeStore, TwoFACodeStoreError},
    email::Email,
};

#[derive(Clone, Default, Debug)]
pub struct HashMapTwoFACodeStore {
    codes: HashMap<Email, (LoginAttemptId, TwoFACode)>,
}

impl HashMapTwoFACodeStore {
    pub fn new() -> Self {
        Self { codes: HashMap::new() }
    }
}

#[async_trait::async_trait]
impl TwoFACodeStore for HashMapTwoFACodeStore {
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError> {
        self.codes
            .insert(email.clone(), (login_attempt_id.clone(), code.clone()));
        Ok(())
    }

    async fn get_code(&self, email: &Email) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError> {
        match self.codes.get(email) {
            None => Err(TwoFACodeStoreError::LoginAttemptIdNotFound),
            Some(code_ref) => Ok((*code_ref).clone()),
        }
    }

    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError> {
        match self.codes.remove(email) {
            None => Err(TwoFACodeStoreError::LoginAttemptIdNotFound),
            Some(_) => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use secrecy::Secret;

    use super::*;
    use crate::domain::email::Email;

    fn str_to_valid_email(email: &str) -> Email {
        Email::parse(Secret::new(email.to_string())).unwrap()
    }

    #[tokio::test]
    async fn test_add_code() {
        let mut store = HashMapTwoFACodeStore::new();
        let email = str_to_valid_email("test@example.com");
        let login_attempt_id = LoginAttemptId::default();
        let code = TwoFACode::default();

        let result = store
            .add_code(email.clone(), login_attempt_id.clone(), code.clone())
            .await;

        assert!(result.is_ok());
        assert_eq!(store.codes.len(), 1);
        assert_eq!(store.codes.get(&email), Some(&(login_attempt_id, code)));
    }

    #[tokio::test]
    async fn test_get_code_existing() {
        let mut store = HashMapTwoFACodeStore::new();
        let email = str_to_valid_email("test@example.com");
        let login_attempt_id = LoginAttemptId::default();
        let code = TwoFACode::default();

        store
            .codes
            .insert(email.clone(), (login_attempt_id.clone(), code.clone()));

        let result = store.get_code(&email).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), (login_attempt_id, code));
    }

    #[tokio::test]
    async fn test_get_code_non_existing() {
        let store = HashMapTwoFACodeStore::new();
        let email = str_to_valid_email("test@example.com");

        let result = store.get_code(&email).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_remove_code_existing() {
        let mut store = HashMapTwoFACodeStore::new();
        let email = str_to_valid_email("test@example.com");
        let login_attempt_id = LoginAttemptId::default();
        let code = TwoFACode::default();

        store.codes.insert(email.clone(), (login_attempt_id, code));

        let result = store.remove_code(&email).await;

        assert!(result.is_ok());
        assert_eq!(store.codes.len(), 0);
        assert!(store.codes.get(&email).is_none());
    }

    #[tokio::test]
    async fn test_remove_code_non_existing() {
        let mut store = HashMapTwoFACodeStore::new();
        let email = str_to_valid_email("test@example.com");

        let result = store.remove_code(&email).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_add_code_overwrites_existing() {
        let mut store = HashMapTwoFACodeStore::new();
        let email = str_to_valid_email("test@example.com");
        let login_attempt_id1 = LoginAttemptId::default();
        let code1 = TwoFACode::default();
        let login_attempt_id2 = LoginAttemptId::default();
        let code2 = TwoFACode::default();

        store.add_code(email.clone(), login_attempt_id1, code1).await.unwrap();
        store
            .add_code(email.clone(), login_attempt_id2.clone(), code2.clone())
            .await
            .unwrap();

        assert_eq!(store.codes.len(), 1);
        assert_eq!(store.codes.get(&email), Some(&(login_attempt_id2, code2)));
    }
}
