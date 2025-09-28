use axum::{
    extract::Path,
    Json,
};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use keystone::{api_router, api_handler};

#[derive(Serialize, JsonSchema)]
struct HelloResponse {
    message: String,
}

#[derive(Deserialize, JsonSchema)]
struct GreetRequest {
    name: String,
}

#[derive(Serialize, JsonSchema)]
struct GreetResponse {
    greeting: String,
}

#[derive(Deserialize)]
struct UserId {
    id: u32,
}

#[derive(Serialize, JsonSchema)]
struct UserResponse {
    id: u32,
    name: String,
    email: String,
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
        name: format!("User {}", id),
        email: format!("user{}@example.com", id),
    })
}

#[tokio::main]
async fn main() {
    // Example 1: Default OpenAPI routes (uses /openapi prefix)
    if std::env::args().any(|arg| arg == "--default") {
        println!("Using default OpenAPI routes...");
        
        let app = api_router!("Hello World API", "1.0.0")
            .get("/", hello)
            .post("/greet", greet) 
            .get("/users/:id", get_user)
            .with_openapi_routes()  // Default: /openapi.json and /openapi.yaml
            .into_router();

        run_server(app, vec![
            "  - http://127.0.0.1:3000/openapi.json",  
            "  - http://127.0.0.1:3000/openapi.yaml",
        ]).await;
        return;
    }

    // Example 2: Custom OpenAPI routes prefix
    println!("Using custom OpenAPI routes with /api/docs prefix...");
    
    let app = api_router!("Hello World API", "1.0.0")
        .get("/", hello)
        .post("/greet", greet) 
        .get("/users/:id", get_user)
        .with_openapi_routes_prefix("/api/docs")  // Custom: /api/docs.json and /api/docs.yaml
        .into_router();

    run_server(app, vec![
        "  - http://127.0.0.1:3000/api/docs.json",  
        "  - http://127.0.0.1:3000/api/docs.yaml",
    ]).await;
}

async fn run_server(app: axum::Router, openapi_urls: Vec<&str>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    
    println!("Server running on http://127.0.0.1:3000");
    println!("OpenAPI spec available at:");
    for url in openapi_urls {
        println!("{}", url);
    }
    println!("");
    println!("Available endpoints:");
    println!("  - GET /");
    println!("  - POST /greet");
    println!("  - GET /users/:id");
    println!("");
    println!("Usage:");
    println!("  cargo run                 # Uses custom prefix /api/docs");
    println!("  cargo run -- --default    # Uses default prefix /openapi");
    
    axum::serve(listener, app).await.unwrap();
}