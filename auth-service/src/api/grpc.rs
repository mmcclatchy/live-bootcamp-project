use std::sync::Arc;
use std::{error::Error, net::SocketAddr};

use log::info;
use secrecy::Secret;
use tonic::{Request, Response, Status};

use crate::domain::user::NewUser;
use crate::domain::{
    data_stores::{UserStore, UserStoreError},
    email::Email,
    error::AuthAPIError,
    password::Password,
};
use crate::services::app_state::{AppServices, AppState};
use auth_proto::{
    auth_service_server::{AuthService, AuthServiceServer},
    SignupRequest, SignupResponse, VerifyTokenRequest, VerifyTokenResponse,
};

pub struct GRPCAuthService<S: AppServices + 'static> {
    pub app_state: Arc<AppState<S>>,
}

impl<S: AppServices + 'static> GRPCAuthService<S> {
    pub fn new(app_state: Arc<AppState<S>>) -> Self {
        Self { app_state }
    }
}

#[tonic::async_trait]
impl<S: AppServices + 'static> AuthService for GRPCAuthService<S> {
    async fn signup(&self, request: Request<SignupRequest>) -> Result<Response<SignupResponse>, Status> {
        info!("[gRPC][signup] Received request:  {:?}", request);

        let req = request.into_inner();
        let email = Email::parse(req.email).map_err(AuthAPIError::InvalidEmail)?;
        let password = Password::parse(Secret::new(req.password))
            .await
            .map_err(AuthAPIError::InvalidPassword)?;

        let user = NewUser::new(email, password, req.requires_2fa);

        let mut user_store = self.app_state.user_store.write().await;
        user_store.add_user(user).await.map_err(|e| match e {
            UserStoreError::UserAlreadyExists => AuthAPIError::UserAlreadyExists,
            _ => AuthAPIError::UnexpectedError(e.into()),
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

pub struct GRPCApp<S: AppServices + 'static> {
    pub address: SocketAddr,
    server: AuthServiceServer<GRPCAuthService<S>>,
}

impl<S: AppServices + 'static> GRPCApp<S> {
    pub async fn new(app_state: Arc<AppState<S>>, address: String) -> Result<Self, Box<dyn Error>> {
        let listener = tokio::net::TcpListener::bind(&address).await?;
        let address = listener.local_addr()?.to_string();
        #[allow(clippy::expect_fun_call)]
        let address = address
            .parse()
            .expect(&format!("[GRPCApp][new] Failed to parse address: {address}"));
        let auth_service = GRPCAuthService::new(app_state);
        let server = AuthServiceServer::new(auth_service);

        Ok(Self { address, server })
    }

    pub async fn run(self) -> Result<(), tonic::transport::Error> {
        info!("gRPC server listening on {}", self.address);

        tonic::transport::Server::builder()
            .add_service(self.server)
            .serve(self.address)
            .await
    }
}
