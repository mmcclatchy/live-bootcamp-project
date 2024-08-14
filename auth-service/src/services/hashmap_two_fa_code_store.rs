use std::collections::HashMap;

use crate::domain::{
    data_stores::{LoginAttemptId, TwoFACode, TwoFACodeStore, TwoFACodeStoreError},
    email::Email,
};

#[derive(Default)]
pub struct HashMapTwoFACodeStore {
    codes: HashMap<Email, (LoginAttemptId, TwoFACode)>,
}

impl HashMapTwoFACodeStore {
    pub fn new() -> Self {
        Self {
            codes: HashMap::new(),
        }
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

    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError> {
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
