use axum::{
    extract::Path,
    Json,
};
use serde::{Deserialize, Serialize};
use stonehm::{api_router, api_handler};
use stonehm_macros::{StoneSchema, api_error};
use stonehm::StoneSchema; // Need trait in scope for api_handler macro

#[derive(Serialize, StoneSchema)]
struct HelloResponse {
    message: String,
}

#[derive(Deserialize, StoneSchema)]
struct GreetRequest {
    name: String,
}

#[derive(Serialize, StoneSchema)]
struct GreetResponse {
    greeting: String,
}

#[derive(Deserialize)]
struct UserId {
    id: u32,
}

#[derive(Serialize, StoneSchema)]
struct UserResponse {
    id: u32,
    name: String,
    email: String,
}

#[derive(Serialize, StoneSchema)]
struct ErrorResponse {
    error: String,
    code: u32,
}

#[api_error]
enum ApiError {
    /// 404: User not found
    UserNotFound { id: u32 },
    
    /// 400: Invalid input provided  
    InvalidInput { message: String },
    
    /// 500: Internal server error
    DatabaseError,
}

/// Returns a simple hello world message
/// 
/// This endpoint doesn't require any parameters and always returns
/// the same friendly greeting message.
/// 
/// # Responses
/// - 200: Successfully returned hello message
#[api_handler]
async fn hello() -> Json<HelloResponse> {
    Json(HelloResponse {
        message: "Hello, World!".to_string(),
    })
}

/// Creates a personalized greeting
///
/// Takes a name in the request body and returns a personalized
/// greeting message.
///
/// # Request Body
/// Content-Type: application/json
/// The request body should contain a JSON object with a name field.
/// Example: {"name": "John"}
///
/// # Responses
/// - 200: Successfully created personalized greeting
/// - 400: Invalid request body or missing name field
#[api_handler]
async fn greet(Json(payload): Json<GreetRequest>) -> Json<GreetResponse> {
    Json(GreetResponse {
        greeting: format!("Hello, {}!", payload.name),
    })
}

/// Get user information by ID
///
/// Retrieves user information for the specified user ID. 
/// Returns mock user data including name and email address.
///
/// # Parameters
/// - id (path): The unique identifier of the user to retrieve
///
/// # Responses
/// - 200: Successfully retrieved user information
/// - 404: User not found for the given ID
/// - 400: Invalid user ID format
#[api_handler]
async fn get_user(Path(UserId { id }): Path<UserId>) -> Json<UserResponse> {
    Json(UserResponse {
        id,
        name: format!("User {id}"),
        email: format!("user{id}@example.com"),
    })
}

/// Delete a user account
///
/// Permanently deletes a user account and all associated data.
/// This action cannot be undone.
///
/// # Parameters
/// - id (path): The unique identifier of the user to delete
///
/// # Responses
/// - 204: User successfully deleted
/// - 404:
///   description: User not found
///   content:
///     application/json:
///       schema: ErrorResponse
///   examples:
///     - name: user_not_found
///       summary: User ID does not exist
///       value: {"error": "User not found", "code": 404}
/// - 403:
///   description: Insufficient permissions to delete user
///   content:
///     application/json:
///       schema: ErrorResponse
#[api_handler]
async fn delete_user(Path(UserId { id }): Path<UserId>) {
    // Implementation would go here
    println!("Deleting user {id}")
}

/// Create a new user with automatic error handling
///
/// Creates a new user account. This demonstrates automatic error response
/// generation based on the Result return type.
///
/// # Request Body
/// Content-Type: application/json
/// User information with name and email fields.
#[api_handler]
async fn create_user_with_errors(Json(payload): Json<GreetRequest>) -> Result<Json<UserResponse>, ApiError> {
    // Simulate some basic validation
    if payload.name.is_empty() {
        return Err(ApiError::InvalidInput { 
            message: "Name cannot be empty".to_string() 
        });
    }
    
    // Simulate a database lookup
    if payload.name == "existing_user" {
        return Err(ApiError::UserNotFound { id: 123 });
    }
    
    // Simulate a database error
    if payload.name == "db_error" {
        return Err(ApiError::DatabaseError);
    }
    
    let name = payload.name;
    Ok(Json(UserResponse {
        id: 42,
        name: name.clone(),
        email: format!("{}@example.com", name.to_lowercase()),
    }))
}


#[tokio::main]
async fn main() {
    // Test: Print the OpenAPI spec to see if schemas are included
    if std::env::args().any(|arg| arg == "--test-schema") {
        let router = api_router!("Hello World API", "1.0.0")
            .get("/", hello)
            .post("/greet", greet) 
            .get("/users/:id", get_user)
            .delete("/users/:id", delete_user)
            .post("/users", create_user_with_errors);
            
        let spec = router.openapi_spec();
        println!("{}", serde_json::to_string_pretty(&spec).unwrap());
        return;
    }
    
    // Create router with routes
    let router = api_router!("Hello World API", "1.0.0")
        .get("/", hello)
        .post("/greet", greet) 
        .get("/users/:id", get_user)
        .delete("/users/:id", delete_user)
        .post("/users", create_user_with_errors);
    
    // Add OpenAPI routes based on flag
    let (app, openapi_urls) = if std::env::args().any(|arg| arg == "--default") {
        println!("Using default OpenAPI routes...");
        (
            router.with_openapi_routes().into_router(),
            vec![
                "  - http://127.0.0.1:3000/openapi.json",  
                "  - http://127.0.0.1:3000/openapi.yaml",
            ]
        )
    } else {
        println!("Using custom OpenAPI routes with /api/docs prefix...");
        (
            router.with_openapi_routes_prefix("/api/docs").into_router(),
            vec![
                "  - http://127.0.0.1:3000/api/docs.json",  
                "  - http://127.0.0.1:3000/api/docs.yaml",
            ]
        )
    };

    run_server(app, openapi_urls).await;
}

async fn run_server(app: axum::Router, openapi_urls: Vec<&str>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    
    println!("Server running on http://127.0.0.1:3000");
    println!("OpenAPI spec available at:");
    for url in openapi_urls {
        println!("{url}");
    }
    println!();
    println!("Available endpoints:");
    println!("  - GET /");
    println!("  - POST /greet");
    println!("  - GET /users/:id");
    println!("  - DELETE /users/:id");
    println!("  - POST /users");
    println!();
    println!("Usage:");
    println!("  cargo run                 # Uses custom prefix /api/docs");
    println!("  cargo run -- --default    # Uses default prefix /openapi");
    
    axum::serve(listener, app).await.unwrap();
}
