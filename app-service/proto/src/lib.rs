pub mod generated;

pub use generated::auth::*;

// Custom implementations for the generated types

impl std::fmt::Display for SignupRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SignupRequest {{ email: {}, requires_2fa: {} }}",
            self.email, self.requires_2fa
        )
    }
}

impl std::fmt::Display for SignupResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SignupResponse {{ message: {} }}", self.message)
    }
}

impl std::fmt::Display for VerifyTokenRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "VerifyTokenRequest {{ token: {} }}", self.token)
    }
}

impl std::fmt::Display for VerifyTokenResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "VerifyTokenResponse {{ is_valid: {} }}", self.is_valid)
    }
}

// Any future implementations or helper functions can go below
