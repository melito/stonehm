# Stonehm - Automatic OpenAPI Generation for Axum

Stonehm is a Rust library that automatically generates OpenAPI 3.0 specifications from your Axum web applications by extracting documentation directly from rustdoc comments. It provides a seamless developer experience by using the standard Axum router API while automatically capturing rich API documentation.

## Key Features

- 🚀 **Zero-friction integration** - Uses standard Axum router syntax
- 📝 **Rustdoc extraction** - Automatically extracts API documentation from doc comments
- 🔄 **Automatic updates** - OpenAPI spec updates whenever you change your code
- 📋 **Response documentation** - Document multiple response codes and descriptions
- 🛠️ **Type-safe** - Leverages Rust's type system for accuracy
- ⚡ **Compile-time processing** - No runtime overhead for documentation
- 🔗 **Complete schema generation** - Automatic request/response body schemas

## Quick Start

### Installation

Add Stonehm to your `Cargo.toml`:

```toml
[dependencies]
stonehm = "0.1"
stonehm-macros = "0.1"
axum = "0.7"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

**Note**: Stonehm re-exports `serde` and `serde_json` so you can use `stonehm::serde` instead of adding these as direct dependencies. Schema generation is built-in using our custom `StoneSchema` derive macro from the `stonehm-macros` crate.

### Minimal Setup

For a minimal setup, you only need:

```toml
[dependencies]
stonehm = "0.1"
stonehm-macros = "0.1"
axum = "0.7"  
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

And in your code:

```rust
use axum::Json;
use stonehm::{api_router, api_handler};
use stonehm_macros::StoneSchema;
use serde::Serialize;

#[derive(Serialize, StoneSchema)]
struct Response { message: String }

#[api_handler]
async fn hello() -> Json<Response> {
    Json(Response { message: "Hello!".to_string() })
}

#[tokio::main]
async fn main() {
    let app = api_router!("API", "1.0.0")
        .get("/", hello)
        .with_openapi_routes()
        .into_router();
    // ... server setup
}
```

### Basic Example

```rust
use axum::{Json, extract::Path};
use stonehm::{api_router, api_handler};
use stonehm_macros::StoneSchema;
use serde::Serialize;

#[derive(Serialize, StoneSchema)]
struct HelloResponse {
    message: String,
}

/// Says hello to the world
/// 
/// This endpoint returns a friendly greeting message.
/// 
/// # Responses
/// - 200: Successfully returned greeting
#[api_handler]
async fn hello() -> Json<HelloResponse> {
    Json(HelloResponse {
        message: "Hello, World!".to_string(),
    })
}

#[tokio::main]
async fn main() {
    // Create router with automatic OpenAPI generation
    let app = api_router!("My API", "1.0.0")
        .get("/hello", hello)
        .with_openapi_routes()  // Default: /openapi.json, /openapi.yaml
        .into_router();
    
    // Alternative: Custom prefix
    // .with_openapi_routes_prefix("/api/docs")  // Creates /api/docs.json, /api/docs.yaml

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    
    println!("Server running on http://127.0.0.1:3000");
    println!("OpenAPI spec at http://127.0.0.1:3000/openapi.json");
    
    axum::serve(listener, app).await.unwrap();
}
```

## How It Works

### 1. Handler Documentation

Annotate your Axum handlers with `#[api_handler]` to enable automatic documentation extraction:

```rust
/// Get user by ID
///
/// Retrieves detailed user information for the specified user ID.
/// The user ID must be a valid positive integer.
///
/// # Responses
/// - 200: Successfully retrieved user information
/// - 404: User not found
/// - 400: Invalid user ID format
#[api_handler]
async fn get_user(Path(id): Path<u32>) -> Json<User> {
    // Implementation
}
```

### 2. Router Creation

Use `api_router!` instead of `Router::new()` to create a router with OpenAPI support:

```rust
let app = api_router!("My API", "1.0.0")
    .get("/users/:id", get_user)
    .post("/users", create_user)
    .put("/users/:id", update_user)
    .delete("/users/:id", delete_user)
    .with_openapi_routes()  // Adds /openapi.json and /openapi.yaml endpoints
    .into_router();         // Converts to regular axum::Router
```

### 3. Documentation Format

The rustdoc comments are parsed with the following structure:

- **First line**: Becomes the OpenAPI summary
- **Remaining lines**: Become the OpenAPI description
- **# Parameters section**: Documents path, query, and header parameters
- **# Request Body section**: Documents the request body
- **# Responses section**: Documents HTTP response codes

#### Parameter Documentation Format

```rust
/// # Parameters
/// - id (path): The unique identifier of the resource
/// - limit (query): Maximum number of results to return
/// - api-key (header): API authentication key
```

#### Request Body Documentation Format

```rust
/// # Request Body
/// Content-Type: application/json
/// The request body should contain user information including name and email.
```

#### Response Documentation Format

```rust
/// # Responses
/// - 200: Success description
/// - 400: Bad request description
/// - 404: Not found description
/// - 500: Internal server error description
```

## Complete Example

Here's a full example showing all features:

```rust
use axum::{Json, extract::Path};
use serde::{Deserialize, Serialize};
use stonehm::{api_router, api_handler};
use stonehm_macros::StoneSchema;

#[derive(Serialize, Deserialize, StoneSchema)]
struct User {
    id: u32,
    name: String,
    email: String,
}

#[derive(Deserialize, StoneSchema)]
struct CreateUserRequest {
    name: String,
    email: String,
}

#[derive(Serialize, StoneSchema)]
struct UsersResponse {
    users: Vec<User>,
}

#[derive(Serialize, StoneSchema)]
struct ErrorResponse {
    error: String,
}

/// List all users
/// 
/// Returns a paginated list of all users in the system.
/// Use query parameters for pagination control.
/// 
/// # Responses
/// - 200: Successfully retrieved user list
/// - 500: Internal server error
#[api_handler]
async fn list_users() -> Json<UsersResponse> {
    // Implementation
    Json(UsersResponse { users: vec![] })
}

/// Get user by ID
///
/// Retrieves detailed information for a specific user.
///
/// # Parameters
/// - id (path): The unique user identifier
///
/// # Responses  
/// - 200: Successfully retrieved user
/// - 404: User not found
/// - 400: Invalid user ID
#[api_handler]
async fn get_user(Path(id): Path<u32>) -> Json<User> {
    Json(User {
        id,
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
    })
}

/// Create a new user
///
/// Creates a new user account with the provided information.
/// Email addresses must be unique.
///
/// # Request Body
/// Content-Type: application/json
/// User information with required name and email fields.
///
/// # Responses
/// - 201: User successfully created
/// - 400: Invalid request data
/// - 409: Email already exists
#[api_handler]
async fn create_user(Json(payload): Json<CreateUserRequest>) -> Json<User> {
    Json(User {
        id: 1,
        name: payload.name,
        email: payload.email,
    })
}

/// Update user information
///
/// Updates an existing user's information. All fields are optional.
///
/// # Responses
/// - 200: User successfully updated
/// - 404: User not found
/// - 400: Invalid request data
#[api_handler]
async fn update_user(
    Path(id): Path<u32>,
    Json(payload): Json<CreateUserRequest>,
) -> Json<User> {
    Json(User {
        id,
        name: payload.name,
        email: payload.email,
    })
}

/// Delete a user
///
/// Permanently deletes a user account. This action cannot be undone.
///
/// # Responses
/// - 204: User successfully deleted
/// - 404: User not found
/// - 403: Insufficient permissions
#[api_handler]
async fn delete_user(Path(id): Path<u32>) -> Result<(), StatusCode> {
    Ok(())
}

#[tokio::main]
async fn main() {
    let app = api_router!("User Management API", "1.0.0")
        .get("/users", list_users)
        .get("/users/:id", get_user)
        .post("/users", create_user)
        .put("/users/:id", update_user)
        .delete("/users/:id", delete_user)
        .with_openapi_routes()
        .into_router();

    // ... server setup
}
```

## API Reference

### Macros

#### `api_router!(title, version)`

Creates a new `DocumentedRouter` that tracks routes and generates OpenAPI documentation.

```rust
let app = api_router!("My API", "1.0.0");
```

#### `#[api_handler]`

Attribute macro that extracts rustdoc comments from handler functions.

```rust
#[api_handler]
async fn my_handler() -> Json<Response> {
    // Implementation
}
```

### DocumentedRouter Methods

The `DocumentedRouter` supports all standard HTTP methods with automatic documentation:

- `.get(path, handler)` - Register a GET route
- `.post(path, handler)` - Register a POST route  
- `.put(path, handler)` - Register a PUT route
- `.delete(path, handler)` - Register a DELETE route
- `.patch(path, handler)` - Register a PATCH route

### Special Methods

#### `.with_openapi_routes()`

Adds OpenAPI spec endpoints to the router using the default prefix:
- `/openapi.json` - JSON format
- `/openapi.yaml` - YAML format

#### `.with_openapi_routes_prefix(prefix)`

Adds OpenAPI spec endpoints to the router with a custom prefix:

```rust
// Default prefix
.with_openapi_routes()  // Creates /openapi.json and /openapi.yaml

// Custom prefix  
.with_openapi_routes_prefix("/api/docs")  // Creates /api/docs.json and /api/docs.yaml
.with_openapi_routes_prefix("/v1/spec")   // Creates /v1/spec.json and /v1/spec.yaml
```

#### `.into_router()`

Converts the `DocumentedRouter` into a regular `axum::Router`.

## Advanced Usage

### Custom Response Types

Document different response types for different status codes:

```rust
/// Search for users
///
/// Searches for users matching the given criteria.
///
/// # Responses
/// - 200: Search results returned successfully
/// - 400: Invalid search parameters
/// - 429: Rate limit exceeded
/// - 503: Search service temporarily unavailable
#[api_handler]
async fn search_users(Query(params): Query<SearchParams>) -> Json<SearchResults> {
    // Implementation
}
```

### Nested Routers

Stonehm routers can be nested like regular Axum routers:

```rust
let users_api = api_router!("Users API", "1.0.0")
    .get("/", list_users)
    .post("/", create_user);

let main_api = api_router!("Main API", "1.0.0")
    .nest("/users", users_api.into_router())
    .with_openapi_routes();
```

## Schema Generation

The crate automatically generates comprehensive request and response schemas:

- ✅ **Request body schemas** are automatically generated and included
- ✅ **Response body schemas** are automatically detected and included for 200 responses
- ✅ **Schema references** point to the generated component schemas
- ✅ **Complete OpenAPI 3.0 compliance** with proper component definitions

For 200 responses, the generated OpenAPI will include both the description and the complete response body structure:

```json
"responses": {
  "200": {
    "description": "Successfully retrieved user profile",
    "content": {
      "application/json": {
        "schema": {
          "$ref": "#/components/schemas/User"
        }
      }
    }
  }
}
```

## Contributing

We welcome contributions! Please see our contributing guidelines for details.

### Development Setup

```bash
# Clone the repository
git clone https://github.com/melito/stonehm.git
cd stonehm

# Run tests
cargo test

# Run the example
cd examples/hello_world
cargo run

# Or run from the root directory
cargo run -p hello_world

# Test OpenAPI generation
cargo run -p hello_world -- --test-schema

# Use custom OpenAPI prefix
cargo run -p hello_world -- --default
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.