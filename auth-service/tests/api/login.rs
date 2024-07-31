// use tonic::{Request, Status};

// use crate::helpers::TestApp;

// #[tokio::test]
// async fn login_returns_ok() {
//     let mut app = TestApp::new().await;

//     // Create a login request
//     let login_request = LoginRequest {
//         email: "test@example.com".to_string(),
//         password: "password123".to_string(),
//     };

//     // Send the login request
//     let response = app.client.login(Request::new(login_request)).await;

//     // Check if the response is Ok
//     assert!(response.is_ok(), "Login failed: {:?}", response.err());

//     // If you want to check the contents of the response:
//     if let Ok(response) = response {
//         let login_response = response.into_inner();
//         // Add assertions based on your LoginResponse structure
//         // For example:
//         // assert!(!login_response.token.is_empty(), "Token should not be empty");
//     }
// }
