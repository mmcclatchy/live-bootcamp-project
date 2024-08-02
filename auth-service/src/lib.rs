use std::net::SocketAddr;
use std::sync::Arc;

use log::info;
use tonic::{Request, Response, Status};

use auth_proto::{
    auth_service_server::{AuthService, AuthServiceServer},
    SignupRequest, SignupResponse, VerifyTokenRequest, VerifyTokenResponse,
};
use domain::{
    data_stores::{UserStore, UserStoreError},
    email::Email,
    error::AuthAPIError,
    password::Password,
    user::User,
};
use services::app_state::AppState;

pub mod domain;
pub mod services;

pub struct AuthServiceImpl<T: UserStore> {
    app_state: Arc<AppState<T>>,
}

impl<T: UserStore> AuthServiceImpl<T> {
    pub fn new(app_state: Arc<AppState<T>>) -> Self {
        Self { app_state }
    }
}

#[tonic::async_trait]
impl<T: UserStore + Send + Sync + 'static> AuthService for AuthServiceImpl<T> {
    async fn signup(
        &self,
        request: Request<SignupRequest>,
    ) -> Result<Response<SignupResponse>, Status> {
        info!("Received signup request");

        let req = request.into_inner();
        let email = Email::parse(req.email).map_err(AuthAPIError::InvalidEmail)?;
        let password = Password::parse(req.password).map_err(AuthAPIError::InvalidPassword)?;

        let user = User::new(email, password, req.requires_2fa);

        let mut user_store = self.app_state.user_store.write().await;
        user_store.add_user(user).await.map_err(|e| match e {
            UserStoreError::UserAlreadyExists => AuthAPIError::UserAlreadyExists,
            _ => AuthAPIError::UnexpectedError,
        })?;

        Ok(Response::new(SignupResponse {
            message: "User created successfully".to_string(),
        }))
    }

    async fn verify_token(
        &self,
        request: Request<VerifyTokenRequest>,
    ) -> Result<Response<VerifyTokenResponse>, Status> {
        info!("Received verify_token request");

        let _req = request.into_inner();
        // TODO: Implement token verification logic
        Ok(Response::new(VerifyTokenResponse { is_valid: true }))
    }
}

pub struct Application {
    address: SocketAddr,
}

impl Application {
    pub fn new(address: SocketAddr) -> Self {
        Self { address }
    }

    pub async fn run<T: UserStore + Send + Sync + 'static>(
        self,
        app_state: Arc<AppState<T>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("gRPC server listening on {}", self.address);

        let auth_service = AuthServiceImpl::new(app_state);
        let grpc_service = AuthServiceServer::new(auth_service);

        tonic::transport::Server::builder()
            .add_service(grpc_service)
            .serve(self.address)
            .await
            .map_err(|e| e.into())
    }
}
