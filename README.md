# Stonehm - Documentation-Driven OpenAPI Generation for Axum

Stonehm automatically generates comprehensive OpenAPI 3.0 specifications for Axum web applications by analyzing handler functions and their documentation. The core principle is **"documentation is the spec"** - write clear, natural documentation and get complete OpenAPI specs automatically.

## 🌟 Key Features

- 🚀 **Zero-friction integration** - Drop-in replacement for `axum::Router`
- 📝 **Documentation-driven** - Extract API docs from standard rustdoc comments
- 🔄 **Automatic error handling** - Detect errors from `Result<T, E>` return types
- 📋 **Multiple response formats** - Support simple and elaborate response documentation
- 🛠️ **Type-safe schema generation** - Automatic request/response schemas via derive macros
- ⚡ **Compile-time processing** - Zero runtime overhead for documentation generation
- 🎯 **Best practices built-in** - Encourages good API documentation habits

## 🚨 What Makes Stonehm Different

**Traditional approach**: Write code, then write separate OpenAPI specs
```yaml
# Separate OpenAPI file to maintain
paths:
  /users/{id}:
    get:
      summary: Get user by ID
      parameters:
        - name: id
          in: path
          required: true
          schema:
            type: integer
      responses:
        '200':
          description: User found
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/User'
        '404':
          description: User not found
```

**Stonehm approach**: Write natural documentation, get OpenAPI automatically
```rust
/// Get user by ID
///
/// Retrieves a user's information using their unique identifier.
#[api_handler]
async fn get_user(Path(id): Path<u32>) -> Result<Json<User>, ApiError> {
    // Implementation automatically generates:
    // ✅ Path parameter documentation  
    // ✅ Success response with User schema
    // ✅ Error responses with ApiError schema
    // ✅ Complete OpenAPI 3.0 specification
}
```

## 🚀 Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
stonehm = "0.1"
stonehm-macros = "0.1"
axum = "0.7"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
serde = { version = "1.0", features = ["derive"] }
```

### 30-Second Example

```rust
use axum::{Json, extract::Path};
use serde::{Serialize, Deserialize};
use stonehm::{api_router, api_handler};
use stonehm_macros::{StoneSchema, api_error};

// Define your data types
#[derive(Serialize, StoneSchema)]
struct User {
    id: u32,
    name: String,
    email: String,
}

#[derive(Serialize, StoneSchema)]
#[api_error]
enum ApiError {
    /// 404: User not found
    UserNotFound { id: u32 },
    
    /// 500: Internal server error
    DatabaseError,
}

/// Get user by ID
///
/// Retrieves a user's information using their unique identifier.
/// Returns detailed user data including name and email.
#[api_handler]
async fn get_user(Path(id): Path<u32>) -> Result<Json<User>, ApiError> {
    // Automatic OpenAPI generation includes:
    // ✅ Path parameter documentation
    // ✅ 200 response with User schema
    // ✅ 400 Bad Request with ApiError schema
    // ✅ 500 Internal Server Error with ApiError schema
    
    Ok(Json(User {
        id,
        name: format!("User {}", id),
        email: format!("user{}@example.com", id),
    }))
}

#[tokio::main]
async fn main() {
    let app = api_router!("User API", "1.0.0")
        .get("/users/:id", get_user)
        .with_openapi_routes()  // Adds /openapi.json and /openapi.yaml
        .into_router();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("Server running on http://127.0.0.1:3000");
    println!("OpenAPI spec: http://127.0.0.1:3000/openapi.json");
    
    axum::serve(listener, app).await.unwrap();
}
```

**That's it!** You now have a fully documented API with automatic OpenAPI generation.

## 📖 Documentation Approaches

Stonehm supports three documentation approaches to fit different needs:

### 1. 🌟 Automatic Documentation (Recommended)

Let Stonehm infer everything from your code structure:

```rust
/// Get user profile
///
/// Retrieves the current user's profile information.
#[api_handler]
async fn get_profile() -> Result<Json<User>, ApiError> {
    // Automatically generates:
    // ✅ 200 response with User schema
    // ✅ 400 Bad Request with ApiError schema
    // ✅ 500 Internal Server Error with ApiError schema
    Ok(Json(User::default()))
}
```

### 2. 📝 Structured Documentation

Add detailed parameter and response documentation:

```rust
/// Update user profile
///
/// Updates the user's profile information. Only provided fields
/// will be updated, others remain unchanged.
///
/// # Parameters
/// - id (path): The user's unique identifier
/// - version (query): API version to use
/// - authorization (header): Bearer token for authentication
///
/// # Request Body
/// Content-Type: application/json
/// User update data with optional fields for name, email, and preferences.
///
/// # Responses
/// - 200: User successfully updated
/// - 400: Invalid user data provided
/// - 401: Authentication required
/// - 404: User not found
/// - 422: Validation failed
#[api_handler]
async fn update_profile(
    Path(id): Path<u32>,
    Json(request): Json<UpdateUserRequest>
) -> Result<Json<User>, ApiError> {
    // Implementation
}
```

### 3. 🔧 Elaborate Documentation

For complex APIs requiring detailed error schemas:

```rust
/// Delete user account
///
/// Permanently removes a user account and all associated data.
/// This action cannot be undone.
///
/// # Parameters
/// - id (path): The unique user identifier to delete
///
/// # Responses
/// - 204: User successfully deleted
/// - 404:
///   description: User not found
///   content:
///     application/json:
///       schema: NotFoundError
/// - 403:
///   description: Insufficient permissions to delete user
///   content:
///     application/json:
///       schema: PermissionError
/// - 409:
///   description: Cannot delete user with active subscriptions
///   content:
///     application/json:
///       schema: ConflictError
#[api_handler]
async fn delete_user(Path(id): Path<u32>) -> Result<(), ApiError> {
    // Implementation
}
```

## 🔧 Schema Generation

Stonehm uses the `StoneSchema` derive macro for automatic schema generation:

```rust
use serde::{Serialize, Deserialize};
use stonehm_macros::StoneSchema;

#[derive(Serialize, Deserialize, StoneSchema)]
struct CreateUserRequest {
    name: String,
    email: String,
    age: Option<u32>,
    preferences: UserPreferences,
}

#[derive(Serialize, StoneSchema)]
struct UserResponse {
    id: u32,
    name: String,
    email: String,
    created_at: String,
    is_active: bool,
}

#[derive(Serialize, StoneSchema)]
#[api_error]
enum ApiError {
    /// 400: Invalid input provided
    InvalidInput { field: String, message: String },
    
    /// 404: User not found
    UserNotFound { id: u32 },
    
    /// 409: Email already exists
    EmailAlreadyExists { email: String },
    
    /// 500: Internal server error
    DatabaseError,
    
    /// 422: Validation failed
    ValidationFailed,
}
```

**Supported types**: All primitive types, `Option<T>`, `Vec<T>`, nested structs, and enums.

## 🚀 Router Setup

### Basic Setup

```rust
use stonehm::api_router;

#[tokio::main]
async fn main() {
    let app = api_router!("My API", "1.0.0")
        .get("/users/:id", get_user)
        .post("/users", create_user)
        .put("/users/:id", update_user)
        .delete("/users/:id", delete_user)
        .with_openapi_routes()  // Adds /openapi.json and /openapi.yaml
        .into_router();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

### Custom OpenAPI Endpoints

```rust
// Default endpoints
.with_openapi_routes()  // Creates /openapi.json and /openapi.yaml

// Custom prefix
.with_openapi_routes_prefix("/api/docs")  // Creates /api/docs.json and /api/docs.yaml

// Custom paths
.with_openapi_routes_prefix("/v1/spec")   // Creates /v1/spec.json and /v1/spec.yaml
```

## 📚 Documentation Format Reference

### Summary and Description

```text
/// Brief one-line summary
///
/// Detailed description that can span multiple paragraphs.
/// This becomes the OpenAPI description field.
```

### Parameters Section

```text
/// # Parameters  
/// - id (path): The unique user identifier
/// - page (query): Page number for pagination
/// - limit (query): Maximum results per page  
/// - authorization (header): Bearer token for authentication
```

### Request Body Section

```text
/// # Request Body
/// Content-Type: application/json
/// Detailed description of the expected request body structure
/// and any validation requirements.
```

### Response Documentation

**Simple format** (covers most use cases):
```text
/// # Responses
/// - 200: User successfully created
/// - 400: Invalid user data provided
/// - 409: Email address already exists
```

**Elaborate format** (for detailed error documentation):
```text
/// # Responses
/// - 201: User successfully created
/// - 400:
///   description: Validation failed
///   content:
///     application/json:
///       schema: ValidationError
/// - 409:
///   description: Email already exists
///   content:
///     application/json:
///       schema: ConflictError
```

## 🎯 Best Practices

### 1. Use Result Types for Error Handling

Return `Result<Json<T>, E>` to get automatic error responses:

```rust
/// ✅ Recommended - Automatic error handling
#[api_handler]
async fn get_user() -> Result<Json<User>, ApiError> {
    Ok(Json(User { id: 1, name: "John".to_string() }))
}

/// ❌ Manual - Requires explicit response documentation
#[api_handler]  
async fn get_user_manual() -> Json<User> {
    Json(User { id: 1, name: "John".to_string() })
}
```

### 2. Use api_error Macro for Error Types

```rust
use stonehm_macros::{StoneSchema, api_error};

#[derive(Serialize, StoneSchema)]
#[api_error]
enum ApiError {
    /// 404: User not found
    UserNotFound { id: u32 },
    
    /// 400: Validation failed
    ValidationError { field: String, message: String },
    
    /// 500: Internal server error
    DatabaseError,
}
```

The `api_error` macro automatically generates the `IntoResponse` implementation,
eliminating boilerplate and reducing errors.

### 3. Keep Documentation Natural

Focus on business logic, not OpenAPI details:

```text
/// ✅ Good - describes what the endpoint does
/// Creates a new user account with email verification

/// ❌ Avoid - implementation details
/// Returns HTTP 201 with application/json content-type
```

### 4. Choose the Right Documentation Level

```text
/// Simple for basic APIs
/// # Responses
/// - 200: Success
/// - 400: Bad request

/// Elaborate for complex error handling
/// # Responses  
/// - 400:
///   description: Validation failed
///   content:
///     application/json:
///       schema: ValidationError
```

## 🔍 Automatic vs Manual Response Documentation

| Return Type | Automatic Behavior | When to Use Manual |
|-------------|--------------------|--------------------|
| `Json<T>` | 200 response with T schema | Simple endpoints |
| `Result<Json<T>, E>` | 200 with T schema<br/>400, 500 with E schema | Most endpoints (recommended) |
| `()` or `StatusCode` | 200 empty response | DELETE operations |
| Custom types | Depends on implementation | Advanced use cases |

## 🚨 Common Troubleshooting

**Q: My error responses aren't appearing**  
A: Ensure your function returns `Result<Json<T>, E>` and `E` implements `IntoResponse`.

**Q: Schemas aren't in the OpenAPI spec**  
A: Add `#[derive(StoneSchema)]` to your types and use them in function signatures.

**Q: Path parameters not documented**  
A: Add them to the `# Parameters` section with `(path)` type specification.

**Q: Custom response schemas not working**  
A: Use the elaborate response format with explicit schema references.

## 📖 API Reference

### Macros

| Macro | Purpose | Example |
|-------|---------|---------|
| `api_router!(title, version)` | Create documented router | `api_router!("My API", "1.0.0")` |
| `#[api_handler]` | Mark handler for documentation | `#[api_handler] async fn get_user() {}` |
| `#[derive(StoneSchema)]` | Generate JSON schema | `#[derive(Serialize, StoneSchema)] struct User {}` |

### Router Methods

```rust
let app = api_router!("API", "1.0.0")
    .get("/users", list_users)           // GET route
    .post("/users", create_user)         // POST route  
    .put("/users/:id", update_user)      // PUT route
    .delete("/users/:id", delete_user)   // DELETE route
    .patch("/users/:id", patch_user)     // PATCH route
    .with_openapi_routes()               // Add OpenAPI endpoints
    .into_router();                      // Convert to axum::Router
```

### OpenAPI Endpoints

| Method | Creates | Description |
|--------|---------|-------------|
| `.with_openapi_routes()` | `/openapi.json`<br/>`/openapi.yaml` | Default OpenAPI endpoints |
| `.with_openapi_routes_prefix("/api")` | `/api.json`<br/>`/api.yaml` | Custom prefix |

### Response Type Mapping

| Rust Type | OpenAPI Response | Automatic Errors |
|-----------|------------------|------------------|
| `Json<T>` | 200 with T schema | None |
| `Result<Json<T>, E>` | 200 with T schema | 400, 500 with E schema |
| `()` | 204 No Content | None |
| `StatusCode` | Custom status | None |

## 🎓 Examples

### Full REST API Example

```rust
use axum::{Json, extract::{Path, Query}};
use serde::{Serialize, Deserialize};
use stonehm::{api_router, api_handler};
use stonehm_macros::StoneSchema;

#[derive(Serialize, Deserialize, StoneSchema)]
struct User {
    id: u32,
    name: String,
    email: String,
    created_at: String,
}

#[derive(Deserialize, StoneSchema)]
struct CreateUserRequest {
    name: String,
    email: String,
}

#[derive(Deserialize)]
struct UserQuery {
    page: Option<u32>,
    limit: Option<u32>,
}

#[derive(Serialize, StoneSchema)]
#[api_error]
enum ApiError {
    /// 404: User not found
    UserNotFound { id: u32 },
    
    /// 400: Validation failed
    ValidationError { field: String, message: String },
    
    /// 500: Internal server error
    DatabaseError,
}

/// List users with pagination
///
/// Retrieves a paginated list of users from the database.
///
/// # Parameters
/// - page (query): Page number (default: 1)
/// - limit (query): Users per page (default: 10, max: 100)
#[api_handler]
async fn list_users(Query(query): Query<UserQuery>) -> Result<Json<Vec<User>>, ApiError> {
    Ok(Json(vec![]))
}

/// Get user by ID
///
/// Retrieves detailed user information by ID.
#[api_handler]
async fn get_user(Path(id): Path<u32>) -> Result<Json<User>, ApiError> {
    Ok(Json(User {
        id,
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
        created_at: "2024-01-01T00:00:00Z".to_string(),
    }))
}

/// Create new user
///
/// Creates a new user account with the provided information.
///
/// # Request Body
/// Content-Type: application/json
/// User creation data with required name and email fields.
#[api_handler]
async fn create_user(Json(req): Json<CreateUserRequest>) -> Result<Json<User>, ApiError> {
    Ok(Json(User {
        id: 42,
        name: req.name,
        email: req.email,
        created_at: "2024-01-01T00:00:00Z".to_string(),
    }))
}

#[tokio::main]
async fn main() {
    let app = api_router!("User Management API", "1.0.0")
        .get("/users", list_users)
        .get("/users/:id", get_user)
        .post("/users", create_user)
        .with_openapi_routes()
        .into_router();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("🚀 Server running on http://127.0.0.1:3000");
    println!("📖 OpenAPI spec: http://127.0.0.1:3000/openapi.json");
    axum::serve(listener, app).await.unwrap();
}
```

## 🛠️ Development

### Running Examples

```bash
# Clone the repository
git clone https://github.com/melito/stonehm.git
cd stonehm

# Run the example server
cargo run -p hello_world

# Test OpenAPI generation
cargo run -p hello_world -- --test-schema

# Use default endpoints (/openapi.json, /openapi.yaml)
cargo run -p hello_world -- --default
```

### Testing Schema Generation

```bash
# Generate and view the OpenAPI spec
cargo run -p hello_world -- --test-schema | jq '.'

# Check specific endpoints
cargo run -p hello_world -- --test-schema | jq '.paths."/users".post'

# View all schemas
cargo run -p hello_world -- --test-schema | jq '.components.schemas'
```

## 📝 Contributing

We welcome contributions! Please feel free to submit issues and pull requests.

### Development Setup

```bash
# Run tests
cargo test

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy -- -D warnings

# Test all examples
cargo test --workspace
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

<div align="center">

**[Documentation](https://docs.rs/stonehm) | [Crates.io](https://crates.io/crates/stonehm) | [Repository](https://github.com/melito/stonehm)**

Made with ❤️ for the Rust community

</div>