pub mod api;
pub mod domain;
pub mod routes;
pub mod services;
pub mod utils;

pub use api::grpc::GRPCApp;
pub use api::rest::RESTApp;
