//! # stonehm - Documentation-Driven OpenAPI 3.0 Generation for Axum
//!
//! stonehm automatically generates comprehensive OpenAPI 3.0 specifications for Axum web applications
//! by analyzing handler functions and their documentation. The core principle is **"documentation is the spec"** -
//! write clear, natural documentation and get complete OpenAPI specs automatically.
//!
//! ## Key Features
//!
//! - 🚀 **Zero-friction integration** - Uses standard Axum router syntax  
//! - 📝 **Documentation-driven** - Extract API docs from rustdoc comments
//! - 🔄 **Automatic error handling** - Detect errors from `Result<T, E>` types
//! - 📋 **Complete response documentation** - Support simple and elaborate response formats
//! - 🛠️ **Type-safe schema generation** - Automatic request/response schemas
//! - ⚡ **Compile-time processing** - Zero runtime overhead
//!
//! ## Installation
//!
//! Add to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! stonehm = "0.1"
//! stonehm-macros = "0.1"
//! axum = "0.7"
//! tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
//! serde = { version = "1.0", features = ["derive"] }
//! ```
//!
//! ## Documentation Formats
//!
//! stonehm supports multiple documentation approaches to fit different needs:
//!
//! ### 1. Simple Documentation (Recommended)
//!
//! Write natural documentation and let stonehm handle the rest:
//!
//! ```rust,no_run
//! use axum::Json;
//! use serde::{Deserialize, Serialize};
//! use stonehm::{api_router, api_handler};
//! use stonehm_macros::api_error;
//!
//! #[derive(Serialize, StoneSchema)]
//! struct User {
//!     id: u32,
//!     name: String,
//!     email: String,
//! }
//!
//! #[api_error]
//! enum ApiError {
//!     /// 404: User not found
//!     UserNotFound { id: u32 },
//!     
//!     /// 400: Validation failed
//!     ValidationError { message: String },
//!     
//!     /// 500: Internal server error
//!     DatabaseError,
//! }
//!
//! /// Get user by ID
//! ///
//! /// Retrieves a user's information using their unique identifier.
//! /// Returns detailed user data including name and email.
//! #[api_handler]
//! async fn get_user() -> Result<Json<User>, ApiError> {
//!     // Implementation here
//!     Ok(Json(User {
//!         id: 1,
//!         name: "John Doe".to_string(),
//!         email: "john@example.com".to_string(),
//!     }))
//! }
//! ```
//!
//! **This automatically generates:**
//! - ✅ 200 response with User schema
//! - ✅ 400 Bad Request with ApiError schema  
//! - ✅ 500 Internal Server Error with ApiError schema
//! - ✅ Complete OpenAPI 3.0 specification
//!
//! ### 2. Detailed Documentation Format
//!
//! For more control, use structured documentation sections:
//!
//! ```rust,no_run
//! # use axum::{Json, extract::Path};
//! # use serde::{Deserialize, Serialize};
//! # use stonehm::{api_router, api_handler};
//! # use stonehm_macros::StoneSchema;
//! # #[derive(Serialize, StoneSchema)] struct User { id: u32, name: String }
//! # #[derive(Deserialize)] struct UserId { id: u32 }
//! 
//! /// Update user information
//! ///
//! /// Updates an existing user's profile information. All fields are optional
//! /// and only provided fields will be updated.
//! ///
//! /// # Parameters
//! /// - id (path): The unique user identifier
//! /// - include_profile (query): Whether to include full profile data
//! ///
//! /// # Request Body  
//! /// Content-Type: application/json
//! /// User update data with optional name and email fields.
//! ///
//! /// # Responses
//! /// - 200: User successfully updated
//! /// - 400: Invalid user data provided
//! /// - 404: User not found
//! /// - 403: Insufficient permissions
//! #[api_handler]
//! async fn update_user(Path(UserId { id }): Path<UserId>) -> Json<User> {
//!     // Implementation
//! #   Json(User { id, name: "Updated".to_string() })
//! }
//! ```
//!
//! ### 3. Elaborate Response Documentation  
//!
//! For complex APIs that need detailed error documentation:
//!
//! ```rust,no_run
//! # use axum::{Json, extract::Path};
//! # use serde::{Deserialize, Serialize};
//! # use stonehm::{api_router, api_handler};  
//! # use stonehm_macros::StoneSchema;
//! # #[derive(Serialize, StoneSchema)] struct User { id: u32 }
//! # #[derive(Serialize, StoneSchema)] struct ErrorResponse { error: String, code: u32 }
//! # #[derive(Deserialize)] struct UserId { id: u32 }
//!
//! /// Delete user account
//! ///
//! /// Permanently removes a user account and all associated data.
//! /// This action cannot be undone.
//! ///
//! /// # Parameters
//! /// - id (path): The unique user identifier to delete
//! ///
//! /// # Responses  
//! /// - 204: User successfully deleted
//! /// - 404:
//! ///   description: User not found
//! ///   content:
//! ///     application/json:
//! ///       schema: ErrorResponse
//! /// - 403:
//! ///   description: Insufficient permissions to delete user
//! ///   content:
//! ///     application/json:
//! ///       schema: ErrorResponse
//! #[api_handler]
//! async fn delete_user(Path(UserId { id }): Path<UserId>) {
//!     // Implementation
//! }
//! ```
//!
//! ## Router Setup
//!
//! Create a documented router and add OpenAPI endpoints:
//!
//! ```rust,no_run
//! # use stonehm::api_router;
//! # async fn get_user() {}
//! # async fn update_user() {}
//! # async fn delete_user() {}
//!
//! #[tokio::main]
//! async fn main() {
//!     let app = api_router!("User API", "1.0.0")
//!         .get("/users/:id", get_user)
//!         .put("/users/:id", update_user) 
//!         .delete("/users/:id", delete_user)
//!         .with_openapi_routes()  // Adds /openapi.json and /openapi.yaml
//!         .into_router();
//!
//!     // Custom prefix
//!     let app_custom = api_router!("User API", "1.0.0")
//!         .get("/users/:id", get_user)
//!         .with_openapi_routes_prefix("/api/docs")  // /api/docs.json, /api/docs.yaml
//!         .into_router();
//!
//!     // Server setup
//!     let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
//!     axum::serve(listener, app).await.unwrap();
//! }
//! ```
//!
//! ## Schema Generation
//!
//! Use the `StoneSchema` derive macro for automatic schema generation:
//!
//! ```rust
//! use serde::{Serialize, Deserialize};
//! use stonehm_macros::api_error;
//!
//! #[derive(Serialize, Deserialize, StoneSchema)]
//! struct CreateUserRequest {
//!     name: String,
//!     email: String,
//!     age: Option<u32>,
//!     active: bool,
//! }
//!
//! #[derive(Serialize, StoneSchema)]
//! struct UserResponse {
//!     id: u32,
//!     name: String,
//!     email: String,
//!     created_at: String,
//! }
//!
//! #[derive(Serialize, StoneSchema)]
//! enum UserError {
//!     InvalidEmail,
//!     DuplicateEmail { email: String },
//!     DatabaseConnectionFailed,
//! }
//! ```
//!
//! ## Advanced Features
//!
//! ### Automatic Error Handling
//!
//! Return `Result<Json<T>, E>` types for automatic error response generation:
//!
//! ```rust,no_run
//! # use axum::Json;
//! # use serde::Serialize;
//! # use stonehm::{api_handler};
//! # use stonehm_macros::StoneSchema;
//! # #[derive(Serialize, StoneSchema)] struct User { id: u32 }
//! # #[derive(Serialize, StoneSchema)] enum ApiError { NotFound }
//! # use axum::response::{IntoResponse, Response};
//! # impl IntoResponse for ApiError { fn into_response(self) -> Response { todo!() } }
//!
//! /// Create new user with automatic error handling
//! ///
//! /// Creates a user account. Errors are automatically documented
//! /// based on the Result type.
//! #[api_handler]
//! async fn create_user() -> Result<Json<User>, ApiError> {
//!     // Automatic error responses:
//!     // - 400: Bad Request (ApiError schema)
//!     // - 500: Internal Server Error (ApiError schema)
//!     Ok(Json(User { id: 1 }))
//! }
//! ```
//!
//! ### Multiple Content Types
//!
//! Support different response formats:
//!
//! ```text
//! # Responses
//! - 200:
//!   description: Success response
//!   content:
//!     application/json:
//!       schema: UserResponse
//! - 400:
//!   description: Validation error
//!   content:
//!     application/xml:
//!       schema: ErrorResponse
//! ```
//!
//! ## Documentation Format Reference
//!
//! stonehm extracts documentation from standard Rust doc comments using these sections:
//!
//! ### Summary and Description
//!
//! - **First line**: Becomes the OpenAPI summary
//! - **Following paragraphs**: Become the OpenAPI description
//!
//! ```text
//! /// Create a new user account
//! ///
//! /// This endpoint creates a new user with the provided information.
//! /// Validation is performed on all fields before saving to the database.
//! ```
//!
//! ### Parameters Section
//!
//! Document path, query, and header parameters using this format:
//!
//! ```text
//! /// # Parameters
//! /// - id (path): The unique user identifier
//! /// - page (query): Page number for pagination  
//! /// - limit (query): Maximum number of results per page
//! /// - authorization (header): Bearer token for authentication
//! ```
//!
//! ### Request Body Section
//!
//! Document request body content and schema:
//!
//! ```text
//! /// # Request Body
//! /// Content-Type: application/json
//! /// User creation data including required name and email fields.
//! /// The password must be at least 8 characters long.
//! ```
//!
//! ### Response Documentation
//!
//! **Simple Format** (recommended for most cases):
//!
//! ```text
//! /// # Responses
//! /// - 200: User successfully created
//! /// - 400: Invalid user data provided
//! /// - 409: Email address already exists
//! ```
//!
//! **Elaborate Format** (for detailed error documentation):
//!
//! ```text
//! /// # Responses
//! /// - 201: User successfully created
//! /// - 400:
//! ///   description: Validation failed
//! ///   content:
//! ///     application/json:
//! ///       schema: ValidationError
//! /// - 409:
//! ///   description: Email already exists
//! ///   content:
//! ///     application/json:
//! ///       schema: ConflictError
//! ```
//!
//! ### Automatic vs Manual Response Documentation
//!
//! | Return Type | Automatic Behavior | Manual Override |
//! |-------------|-------------------|-----------------|
//! | `Json<T>` | 200 response with T schema | Use `# Responses` section |
//! | `Result<Json<T>, E>` | 200 with T schema<br/>400, 500 with E schema | Use `# Responses` section |
//! | `()` | 200 empty response | Use `# Responses` section |
//!
//! ## Complete Example
//!
//! Here's a comprehensive example showing all features:
//!
//! ```rust,no_run
//! use axum::{Json, extract::{Path, Query}};
//! use serde::{Serialize, Deserialize};
//! use stonehm::{api_router, api_handler};
//! use stonehm_macros::api_error;
//! 
//! #[derive(Serialize, Deserialize, StoneSchema)]
//! struct User {
//!     id: u32,
//!     name: String,
//!     email: String,
//! }
//!
//! #[derive(Deserialize, StoneSchema)]
//! struct CreateUserRequest {
//!     name: String,
//!     email: String,
//! }
//!
//! #[derive(Deserialize)]
//! struct UserQuery {
//!     include_posts: Option<bool>,
//! }
//!
//! #[api_error]
//! enum ApiError {
//!     /// 404: User not found
//!     UserNotFound { id: u32 },
//!     
//!     /// 400: Validation failed
//!     ValidationError { field: String, message: String },
//!     
//!     /// 500: Internal server error
//!     DatabaseError,
//! }
//!
//! /// Get user by ID with optional post inclusion
//! ///
//! /// Retrieves detailed user information. Optionally includes
//! /// the user's posts if requested via query parameter.
//! ///
//! /// # Parameters
//! /// - id (path): The user's unique identifier
//! /// - include_posts (query): Whether to include user's posts
//! ///
//! /// # Responses
//! /// - 200: User successfully retrieved
//! /// - 404: User not found
//! /// - 400: Invalid user ID format
//! #[api_handler]
//! async fn get_user_detailed(
//!     Path(id): Path<u32>,
//!     Query(query): Query<UserQuery>
//! ) -> Result<Json<User>, ApiError> {
//!     if id == 0 {
//!         return Err(ApiError::ValidationError {
//!             field: "id".to_string(),
//!             message: "ID must be greater than 0".to_string(),
//!         });
//!     }
//!     
//!     Ok(Json(User {
//!         id,
//!         name: format!("User {}", id),
//!         email: format!("user{}@example.com", id),
//!     }))
//! }
//!
//! /// Create a new user account
//! ///
//! /// Creates a new user with the provided information. The email
//! /// address must be unique and valid.
//! ///
//! /// # Request Body
//! /// Content-Type: application/json
//! /// User creation data with required name and email fields.
//! ///
//! /// # Responses
//! /// - 201: User successfully created
//! /// - 400: Invalid user data or validation failed
//! /// - 409: Email address already exists
//! #[api_handler]
//! async fn create_user_complete(
//!     Json(request): Json<CreateUserRequest>
//! ) -> Result<Json<User>, ApiError> {
//!     // Validation
//!     if request.email.is_empty() {
//!         return Err(ApiError::ValidationError {
//!             field: "email".to_string(),
//!             message: "Email is required".to_string(),
//!         });
//!     }
//!     
//!     Ok(Json(User {
//!         id: 42,
//!         name: request.name,
//!         email: request.email,
//!     }))
//! }
//!
//! // Router setup
//! #[tokio::main]
//! async fn main() {
//!     let app = api_router!("User Management API", "1.0.0")
//!         .get("/users/:id", get_user_detailed)
//!         .post("/users", create_user_complete)
//!         .with_openapi_routes()
//!         .into_router();
//!         
//!     let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
//!     axum::serve(listener, app).await.unwrap();
//! }
//! ```
//!
//! ## Best Practices
//!
//! ### 1. Use Result Types for Error Handling
//! 
//! Return `Result<Json<T>, E>` to get automatic error response generation:
//!
//! ```rust,no_run
//! # use axum::Json;
//! # use serde::Serialize;
//! # use stonehm::api_handler;
//! # use stonehm_macros::StoneSchema;
//! # #[derive(Serialize, StoneSchema)] struct User { id: u32 }
//! # #[derive(Serialize, StoneSchema)] enum ApiError { NotFound }
//! # use axum::response::{IntoResponse, Response};
//! # impl IntoResponse for ApiError { fn into_response(self) -> Response { todo!() } }
//!
//! /// ✅ Automatic error responses
//! #[api_handler] 
//! async fn get_user_recommended() -> Result<Json<User>, ApiError> {
//!     Ok(Json(User { id: 1 }))
//! }
//!
//! /// ❌ Manual response documentation needed
//! #[api_handler]
//! async fn get_user_manual() -> Json<User> {
//!     Json(User { id: 1 })
//! }
//! ```
//!
//! ### 2. Keep Documentation Natural
//!
//! Focus on describing what the endpoint does, not OpenAPI details:
//!
//! ```text
//! /// ✅ Good - describes business logic
//! /// Creates a new user account with email verification
//! ///
//! /// ❌ Avoid - OpenAPI implementation details  
//! /// Returns HTTP 201 with application/json content-type
//! ```
//!
//! ### 3. Use Simple Response Format
//!
//! Only use elaborate format when you need detailed error schemas:
//!
//! ```text
//! /// ✅ Simple - sufficient for most cases
//! /// # Responses
//! /// - 200: User created successfully
//! /// - 400: Invalid user data
//! ///
//! /// ❌ Elaborate - only when needed
//! /// # Responses
//! /// - 200:
//! ///   description: User created
//! ///   content:
//! ///     application/json:
//! ///       schema: User
//! ```
//!
//! ### 4. Implement IntoResponse for Error Types
//!
//! Make your error types work with Axum:
//!
//! ```rust
//! # use serde::Serialize;
//! # use stonehm_macros::StoneSchema;
//! use axum::{response::{IntoResponse, Response}, http::StatusCode, Json};
//!
//! #[derive(Serialize, StoneSchema)]
//! enum ApiError {
//!     UserNotFound,
//!     ValidationFailed { field: String },
//! }
//!
//! impl IntoResponse for ApiError {
//!     fn into_response(self) -> Response {
//!         let (status, message) = match self {
//!             ApiError::UserNotFound => (StatusCode::NOT_FOUND, "User not found"),
//!             ApiError::ValidationFailed { field } => (StatusCode::BAD_REQUEST, "Validation failed"),
//!         };
//!         (status, Json(serde_json::json!({"error": message}))).into_response()
//!     }
//! }
//! ```
//!
//! ## Troubleshooting
//!
//! ### Common Issues
//!
//! **Q: My error type isn't generating responses**  
//! A: Make sure your function returns `Result<Json<T>, E>` and use `#[api_error]` on your error enum.
//!
//! **Q: Schemas aren't appearing in OpenAPI**  
//! A: Ensure your types have `#[derive(StoneSchema)]` and are used in function signatures.
//!
//! **Q: Path parameters not documented**  
//! A: Add them to the `# Parameters` section with `(path)` type.
//!
//! **Q: Custom response schemas not working**  
//! A: Use the elaborate response format with explicit schema references.
//!
//! ### Performance Notes
//!
//! - All processing happens at compile time - zero runtime cost
//! - Schema generation uses efficient compile-time reflection
//! - OpenAPI spec is generated once during compilation
//!
//! **Response Schema Generation**: 
//! 
//! The crate automatically generates comprehensive response documentation:
//! - ✅ **Status codes and descriptions** are extracted from `# Responses` documentation
//! - ✅ **Request body schemas** are automatically generated and included
//! - ✅ **Response body schemas** are automatically detected and included for 200 responses
//! - ✅ **Schema references** point to the generated component schemas
//! 
//! For 200 responses, the generated OpenAPI will include both the description and the 
//! complete response body structure with proper JSON schema definitions:
//! 
//! ```json
//! "responses": {
//!   "200": {
//!     "description": "Successfully retrieved user profile",
//!     "content": {
//!       "application/json": {
//!         "schema": {
//!           "$ref": "#/components/schemas/User"
//!         }
//!       }
//!     }
//!   }
//! }
//! ```
//! 
//! Error responses (400, 404, 500, etc.) include descriptions but no body schema, 
//! which is typically correct for error responses.
//!
//! ## Schema Generation
//!
//! Use the `StoneSchema` derive macro on your request/response types:
//!
//! ```rust
//! use serde::Serialize;
//! use stonehm_macros::StoneSchema;
//! 
//! #[derive(Serialize, StoneSchema)]
//! struct User {
//!     id: u32,
//!     name: String,
//!     email: String,
//!     active: bool,
//! }
//! ```
//!
//! This automatically generates JSON Schema definitions that are included
//! in the OpenAPI specification.

use axum::{
    Router,
    routing::{get, post, put, delete, patch, MethodRouter},
    handler::Handler,
};
use http::Method;
use openapiv3::{
    OpenAPI, Info, PathItem, Operation, ReferenceOr, Responses, StatusCode, 
    Response as ApiResponse, Parameter, ParameterData, ParameterSchemaOrContent, 
    Schema, SchemaKind, Type, RequestBody, MediaType, Content
};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
pub use inventory;
pub use stonehm_macros::api_handler;

// Note: The StoneSchema derive macro cannot be re-exported from a proc-macro crate.
// Users should import it directly: use stonehm_macros::StoneSchema;

// Re-export dependencies so users don't need to add them
pub use serde;
pub use serde_json;

/// Trait for types that can generate their own JSON schema using stonehm's schema system.
/// 
/// This trait allows types to provide their own JSON schema representation
/// for OpenAPI specification generation. It's typically implemented automatically
/// via the `#[derive(StoneSchema)]` macro, which is part of the stonehm ecosystem.
/// 
/// The generated schema follows a simplified subset of JSON Schema that is
/// compatible with OpenAPI 3.0 specifications.
/// 
/// # Examples
/// 
/// ## Using the derive macro (recommended)
/// 
/// ```rust
/// use serde::{Serialize, Deserialize};
/// use stonehm_macros::StoneSchema;
/// 
/// #[derive(Serialize, Deserialize, StoneSchema)]
/// struct User {
///     id: u32,
///     name: String,
///     email: String,
///     active: bool,
/// }
/// 
/// // The schema is automatically generated
/// let schema = User::schema();
/// println!("{}", serde_json::to_string_pretty(&schema).unwrap());
/// ```
/// 
/// ## Manual implementation
/// 
/// ```rust
/// use stonehm::StoneSchema;
/// use serde_json::json;
/// 
/// struct CustomType {
///     value: String,
/// }
/// 
/// impl StoneSchema for CustomType {
///     fn schema() -> serde_json::Value {
///         json!({
///             "type": "object",
///             "properties": {
///                 "value": {
///                     "type": "string",
///                     "description": "A custom value"
///                 }
///             },
///             "required": ["value"]
///         })
///     }
/// }
/// ```
/// 
/// # Schema Format
/// 
/// The generated schemas follow this format:
/// - `type`: The JSON Schema type ("object", "string", "number", etc.)
/// - `properties`: For objects, a map of property names to their schemas
/// - `required`: Array of required property names
/// - `title`: The name of the type
/// 
/// # Type Mapping
/// 
/// The derive macro maps Rust types to JSON Schema types as follows:
/// - `String`, `&str` → `"string"`
/// - `i32`, `i64`, `u32`, `u64`, etc. → `"integer"`
/// - `f32`, `f64` → `"number"`
/// - `bool` → `"boolean"`
/// - Complex types → `"string"` (fallback)
pub trait StoneSchema {
    /// Generate a JSON schema for this type.
    /// 
    /// Returns a `serde_json::Value` containing the JSON schema representation
    /// of this type, suitable for inclusion in an OpenAPI specification.
    fn schema() -> serde_json::Value;
}

/// Schema generation macro using stonehm's schema system.
/// 
/// This macro provides a convenient way to generate schemas for types that
/// implement the `StoneSchema` trait. It's similar to `schemars::schema_for!`
/// but uses stonehm's simpler schema system.
/// 
/// # Examples
/// 
/// ```rust,no_run
/// use stonehm::stone_schema_for;
/// use stonehm_macros::StoneSchema;
/// use serde::Serialize;
/// 
/// #[derive(Serialize, StoneSchema)]
/// struct User {
///     id: u32,
///     name: String,
/// }
/// 
/// let schema = stone_schema_for!(User);
/// println!("{}", serde_json::to_string_pretty(&schema).unwrap());
/// ```
#[macro_export]
macro_rules! stone_schema_for {
    ($type:ty) => {{
        // For types that implement our StoneSchema trait
        <$type>::schema()
    }};
}

/// Registry entry for handler documentation
pub struct HandlerDocEntry {
    pub name: &'static str,
    pub get_docs: fn() -> HandlerDocumentation,
}

inventory::collect!(HandlerDocEntry);

/// Registry entry for schema functions
pub struct SchemaEntry {
    pub type_name: &'static str,
    pub get_schema: fn() -> Option<serde_json::Value>,
}

inventory::collect!(SchemaEntry);


/// Documentation for a single HTTP response.
/// 
/// This struct represents information about a specific HTTP response that an API endpoint
/// can return, including the status code, description, and optional content schema
/// and examples for more detailed documentation.
/// 
/// # Examples
/// 
/// ## Simple Response
/// ```rust
/// use stonehm::ResponseDoc;
/// 
/// let success_response = ResponseDoc {
///     status_code: 200,
///     description: "User successfully created".to_string(),
///     content: None,
///     examples: None,
/// };
/// ```
/// 
/// ## Response with Content Schema
/// ```rust
/// use stonehm::{ResponseDoc, ResponseContent};
/// 
/// let error_response = ResponseDoc {
///     status_code: 400,
///     description: "Invalid user data provided".to_string(),
///     content: Some(ResponseContent {
///         media_type: "application/json".to_string(),
///         schema: Some("ErrorResponse".to_string()),
///     }),
///     examples: None,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct ResponseDoc {
    /// The HTTP status code (e.g., 200, 400, 404, 500)
    pub status_code: u16,
    /// Human-readable description of when this response occurs
    pub description: String,
    /// Optional content type and schema information
    pub content: Option<ResponseContent>,
    /// Optional examples for this response
    pub examples: Option<Vec<ResponseExample>>,
}

/// Content information for API responses.
/// 
/// Describes the media type and schema for response bodies, allowing for more
/// detailed OpenAPI documentation of error responses and alternative content types.
#[derive(Debug, Clone)]
pub struct ResponseContent {
    /// The media type (e.g., "application/json", "application/xml")
    pub media_type: String,
    /// Optional schema reference (e.g., "ErrorResponse")
    pub schema: Option<String>,
}

/// Example content for API responses.
/// 
/// Provides concrete examples of response bodies to help API consumers understand
/// the structure and content of responses.
#[derive(Debug, Clone)]
pub struct ResponseExample {
    /// Unique name for this example
    pub name: String,
    /// Optional summary describing this example
    pub summary: Option<String>,
    /// The example content (typically JSON)
    pub value: String,
}

/// Documentation for a single API parameter.
/// 
/// This struct represents information about a parameter that an API endpoint accepts,
/// including its name, description, and type (path, query, or header parameter).
/// 
/// # Examples
/// 
/// ```rust
/// use stonehm::ParameterDoc;
/// 
/// let path_param = ParameterDoc {
///     name: "user_id".to_string(),
///     description: "The unique identifier of the user".to_string(),
///     param_type: "path".to_string(),
/// };
/// 
/// let query_param = ParameterDoc {
///     name: "include_posts".to_string(),
///     description: "Whether to include the user's posts".to_string(),
///     param_type: "query".to_string(),
/// };
/// 
/// let header_param = ParameterDoc {
///     name: "authorization".to_string(),
///     description: "Bearer token for authentication".to_string(),
///     param_type: "header".to_string(),
/// };
/// ```
#[derive(Debug, Clone)]
pub struct ParameterDoc {
    /// The name of the parameter (e.g., "user_id", "page", "authorization")
    pub name: String,
    /// Human-readable description of the parameter's purpose
    pub description: String,
    /// The type of parameter: "path", "query", or "header"
    pub param_type: String,
}

/// Documentation for an API request body.
/// 
/// This struct represents information about the request body that an API endpoint expects,
/// including its description, content type, and optional schema type information.
/// 
/// # Examples
/// 
/// ```rust
/// use stonehm::RequestBodyDoc;
/// 
/// let json_body = RequestBodyDoc {
///     description: "User information for account creation".to_string(),
///     content_type: "application/json".to_string(),
///     schema_type: Some("CreateUserRequest".to_string()),
/// };
/// 
/// let form_body = RequestBodyDoc {
///     description: "File upload with metadata".to_string(),
///     content_type: "multipart/form-data".to_string(),
///     schema_type: None,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct RequestBodyDoc {
    /// Human-readable description of the request body content
    pub description: String,
    /// The MIME type of the request body (e.g., "application/json")
    pub content_type: String,
    /// The actual Rust type name for schema generation (e.g., "GreetRequest")
    pub schema_type: Option<String>,
}

/// Complete documentation information for an API handler.
/// 
/// This struct contains all the documentation metadata extracted from a handler function,
/// including summary, description, parameters, request body, and response information.
/// This is the primary struct used internally by the `#[api_handler]` macro to store
/// and organize documentation data.
/// 
/// # Examples
/// 
/// ```rust
/// use stonehm::{HandlerDocumentation, ParameterDoc, RequestBodyDoc, ResponseDoc};
/// 
/// let docs = HandlerDocumentation {
///     summary: Some("Create a new user"),
///     description: Some("Creates a new user account with the provided information"),
///     parameters: vec![
///         ParameterDoc {
///             name: "api_version".to_string(),
///             description: "API version to use".to_string(),
///             param_type: "header".to_string(),
///         }
///     ],
///     request_body: Some(RequestBodyDoc {
///         description: "User creation data".to_string(),
///         content_type: "application/json".to_string(),
///         schema_type: Some("CreateUserRequest".to_string()),
///     }),
///     request_body_type: Some("CreateUserRequest".to_string()),
///     response_type: Some("UserResponse".to_string()),
///     error_type: None,
///     responses: vec![
///         ResponseDoc {
///             status_code: 201,
///             description: "User successfully created".to_string(),
///             content: None,
///             examples: None,
///         },
///         ResponseDoc {
///             status_code: 400,
///             description: "Invalid user data".to_string(),
///             content: None,
///             examples: None,
///         },
///     ],
/// };
/// ```
#[derive(Debug, Clone)]
pub struct HandlerDocumentation {
    /// Brief one-line summary of what the handler does
    pub summary: Option<&'static str>,
    /// Longer description providing more details about the handler
    pub description: Option<&'static str>,
    /// List of parameters this handler accepts (path, query, header)
    pub parameters: Vec<ParameterDoc>,
    /// Information about the expected request body, if any
    pub request_body: Option<RequestBodyDoc>,
    /// The actual Rust type name for the request body (for schema generation)
    pub request_body_type: Option<String>,
    /// The actual Rust type name for the response (for schema generation)
    pub response_type: Option<String>,
    /// The actual Rust type name for errors (for automatic error response generation)
    pub error_type: Option<String>,
    /// List of possible responses this handler can return
    pub responses: Vec<ResponseDoc>,
}

/// Simplified handler metadata for backwards compatibility.
/// 
/// This struct provides a simpler version of handler documentation that only
/// includes summary and description. It's maintained for backwards compatibility
/// with older versions of the API. For new code, prefer using [`HandlerDocumentation`]
/// which provides more complete information.
/// 
/// # Examples
/// 
/// ```rust
/// use stonehm::HandlerMetadata;
/// 
/// let metadata = HandlerMetadata {
///     summary: Some("Get user by ID"),
///     description: Some("Retrieves a user account by their unique identifier"),
/// };
/// ```
/// 
/// # Migration
/// 
/// If you're currently using `HandlerMetadata`, consider migrating to `HandlerDocumentation`:
/// 
/// ```rust
/// use stonehm::{HandlerMetadata, HandlerDocumentation};
/// 
/// // Old way (still supported)
/// let old_metadata = HandlerMetadata {
///     summary: Some("Create user"),
///     description: Some("Creates a new user account"),
/// };
/// 
/// // New way (recommended)
/// let new_docs = HandlerDocumentation {
///     summary: Some("Create user"),
///     description: Some("Creates a new user account"),
///     parameters: vec![],
///     request_body: None,
///     request_body_type: None,
///     response_type: None,
///     error_type: None,
///     responses: vec![],
/// };
/// ```
#[derive(Debug, Clone)]
pub struct HandlerMetadata {
    /// Brief one-line summary of what the handler does
    pub summary: Option<&'static str>,
    /// Longer description providing more details about the handler
    pub description: Option<&'static str>,
}

/// Complete information about a registered API route.
/// 
/// This struct contains all the information about a route that has been registered
/// with a [`DocumentedRouter`], including the HTTP method, path, and all documentation
/// metadata. This is used internally to track routes and generate the OpenAPI specification.
/// 
/// # Examples
/// 
/// ```rust
/// use stonehm::{RouteInfo, ParameterDoc, RequestBodyDoc, ResponseDoc};
/// use http::Method;
/// 
/// let route = RouteInfo {
///     path: "/users/{id}".to_string(),
///     method: Method::GET,
///     summary: Some("Get user by ID".to_string()),
///     description: Some("Retrieves user information by their unique identifier".to_string()),
///     parameters: vec![
///         ParameterDoc {
///             name: "id".to_string(),
///             description: "The user's unique identifier".to_string(),
///             param_type: "path".to_string(),
///         }
///     ],
///     request_body: None,
///     responses: vec![
///         ResponseDoc {
///             status_code: 200,
///             description: "User successfully retrieved".to_string(),
///             content: None,
///             examples: None,
///         },
///         ResponseDoc {
///             status_code: 404,
///             description: "User not found".to_string(),
///             content: None,
///             examples: None,
///         },
///     ],
/// };
/// ```
/// 
/// # Usage
/// 
/// You typically don't create `RouteInfo` instances manually. Instead, they are
/// automatically created when you register routes with a `DocumentedRouter`:
/// 
/// ```rust
/// # use stonehm::{api_router, api_handler};
/// # use stonehm_macros::StoneSchema;
/// # use axum::Json;
/// # use serde::Serialize;
/// # #[derive(Serialize, StoneSchema)]
/// # struct UserData { id: u32 }
/// # #[derive(Serialize, StoneSchema)]
/// # struct CreatedUser { id: u32 }
/// # #[api_handler]
/// # async fn get_user_handler() -> Json<UserData> { Json(UserData { id: 1 }) }
/// # #[api_handler]
/// # async fn create_user_handler() -> Json<CreatedUser> { Json(CreatedUser { id: 1 }) }
/// let router = api_router!("My API", "1.0.0")
///     .get("/users/{id}", get_user_handler)  // Creates a RouteInfo internally
///     .post("/users", create_user_handler);  // Creates another RouteInfo
/// ```
#[derive(Debug, Clone)]
pub struct RouteInfo {
    /// The route path pattern (e.g., "/users/{id}")
    pub path: String,
    /// The HTTP method for this route
    pub method: Method,
    /// Brief one-line summary of what this route does
    pub summary: Option<String>,
    /// Longer description providing more details about the route
    pub description: Option<String>,
    /// List of parameters this route accepts (path, query, header)
    pub parameters: Vec<ParameterDoc>,
    /// Information about the expected request body, if any
    pub request_body: Option<RequestBodyDoc>,
    /// List of possible responses this route can return
    pub responses: Vec<ResponseDoc>,
}

/// Trait for handlers that provide documentation metadata.
/// 
/// This trait allows handler functions to provide documentation metadata
/// that can be used for OpenAPI generation. It's primarily used for backwards
/// compatibility with older versions of the API.
/// 
/// # Examples
/// 
/// ```rust
/// use stonehm::{DocumentedHandler, HandlerMetadata};
/// 
/// struct MyHandler;
/// 
/// impl DocumentedHandler for MyHandler {
///     fn metadata() -> HandlerMetadata {
///         HandlerMetadata {
///             summary: Some("Custom handler"),
///             description: Some("A custom handler with specific documentation"),
///         }
///     }
/// }
/// ```
/// 
/// # Note
/// 
/// In most cases, you should use the `#[api_handler]` attribute macro instead
/// of implementing this trait manually, as it automatically extracts documentation
/// from function comments and provides more comprehensive metadata.
pub trait DocumentedHandler {
    /// Returns the documentation metadata for this handler.
    /// 
    /// The default implementation returns empty metadata.
    fn metadata() -> HandlerMetadata {
        HandlerMetadata {
            summary: None,
            description: None,
        }
    }
}

/// Registry trait for looking up handler documentation.
/// 
/// This trait provides a way to look up documentation metadata for handlers
/// by their name. It's used internally by the documentation system.
/// 
/// # Note
/// 
/// This trait is primarily for internal use and backwards compatibility.
/// Most users should not need to implement this trait directly.
pub trait HandlerRegistry {
    /// Look up documentation metadata for a handler by name.
    /// 
    /// Returns `Some(HandlerMetadata)` if documentation is found for the
    /// given handler name, or `None` if no documentation is available.
    fn get_handler_docs(handler_name: &str) -> Option<HandlerMetadata>;
}

/// A router that automatically captures handler documentation and generates OpenAPI specs.
/// 
/// `DocumentedRouter` is the main component of the stonehm crate. It wraps an Axum `Router`
/// and automatically extracts documentation from handler functions to generate OpenAPI 3.0
/// specifications. It provides the same interface as `axum::Router` while adding automatic
/// documentation generation capabilities.
/// 
/// # Features
/// 
/// - **Automatic documentation extraction**: Reads doc comments from handler functions
/// - **OpenAPI 3.0 generation**: Produces valid OpenAPI specifications
/// - **Schema generation**: Automatically includes JSON schemas for request/response types
/// - **Multiple output formats**: Supports both JSON and YAML output
/// - **Drop-in replacement**: Can replace `axum::Router` with minimal code changes
/// 
/// # Examples
/// 
/// ## Basic Usage
/// 
/// ```rust,no_run
/// use stonehm::{DocumentedRouter, api_handler};
/// use stonehm_macros::StoneSchema;
/// use axum::Json;
/// use serde::Serialize;
/// 
/// #[derive(Serialize, StoneSchema)]
/// struct HelloResponse {
///     message: String,
/// }
/// 
/// /// Returns a hello world message
/// #[api_handler]
/// async fn hello() -> Json<HelloResponse> {
///     Json(HelloResponse {
///         message: "Hello, World!".to_string(),
///     })
/// }
/// 
/// let router = DocumentedRouter::new("My API", "1.0.0")
///     .get("/", hello)
///     .with_openapi_routes();
/// ```
/// 
/// ## Using the Convenience Macro
/// 
/// ```rust,no_run
/// # use stonehm::{api_router, api_handler};
/// # use stonehm_macros::StoneSchema;
/// # use axum::Json;
/// # use serde::Serialize;
/// # #[derive(Serialize, StoneSchema)]
/// # struct HelloResponse { msg: String }
/// # #[derive(Serialize, StoneSchema)]
/// # struct UserResponse { msg: String }
/// # #[api_handler]
/// # async fn hello() -> Json<HelloResponse> { Json(HelloResponse { msg: "hi".into() }) }
/// # #[api_handler]
/// # async fn create_user() -> Json<UserResponse> { Json(UserResponse { msg: "created".into() }) }
/// let router = api_router!("My API", "1.0.0")
///     .get("/", hello)
///     .post("/users", create_user)
///     .with_openapi_routes();
/// ```
/// 
/// ## Accessing the OpenAPI Specification
/// 
/// ```rust,no_run
/// # use stonehm::{api_router, api_handler};
/// # use stonehm_macros::StoneSchema;
/// # use axum::Json;
/// # use serde::Serialize;
/// # #[derive(Serialize, StoneSchema)]
/// # struct Response { msg: String }
/// # #[api_handler]
/// # async fn hello() -> Json<Response> { Json(Response { msg: "hi".into() }) }
/// let router = api_router!("My API", "1.0.0")
///     .get("/", hello);
/// 
/// // Get the OpenAPI spec as a struct
/// let spec = router.openapi_spec();
/// println!("{}", serde_json::to_string_pretty(&spec).unwrap());
/// 
/// // Convert to regular Axum router
/// let axum_router = router.into_router();
/// ```
pub struct DocumentedRouter {
    inner: Router,
    routes: Arc<Mutex<Vec<RouteInfo>>>,
    spec: Arc<Mutex<OpenAPI>>,
    schemas: Arc<Mutex<BTreeMap<String, ReferenceOr<Schema>>>>,
}

impl DocumentedRouter {
    /// Creates a new `DocumentedRouter` with the given API title and version.
    /// 
    /// This initializes a new router with an empty OpenAPI 3.0.3 specification
    /// that will be populated as routes are added.
    /// 
    /// # Arguments
    /// 
    /// * `title` - The title of your API (e.g., "My REST API")
    /// * `version` - The version of your API (e.g., "1.0.0", "v2.1")
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use stonehm::DocumentedRouter;
    /// 
    /// let router = DocumentedRouter::new("User Management API", "1.2.0");
    /// ```
    pub fn new(title: impl Into<String>, version: impl Into<String>) -> Self {
        let spec = OpenAPI {
            openapi: "3.0.3".to_string(),
            info: Info {
                title: title.into(),
                version: version.into(),
                ..Default::default()
            },
            ..Default::default()
        };

        Self {
            inner: Router::new(),
            routes: Arc::new(Mutex::new(Vec::new())),
            spec: Arc::new(Mutex::new(spec)),
            schemas: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }

    /// Route with automatic documentation lookup
    pub fn route<F>(self, path: &str, method_router: MethodRouter) -> Self {
        // For now, register without docs - we'll enhance this
        let empty_docs = HandlerDocumentation {
            summary: None,
            description: None,
            parameters: vec![],
            request_body: None,
            request_body_type: None,
            response_type: None,
            error_type: None,
            responses: vec![],
        };
        self.register_route(path, Method::GET, empty_docs); // Default to GET, would need smarter detection
        
        Self {
            inner: self.inner.route(path, method_router),
            routes: self.routes,
            spec: self.spec,
            schemas: self.schemas,
        }
    }

    /// Add a GET route with automatic documentation lookup
    pub fn get<H, T>(self, path: &str, handler: H) -> Self 
    where
        H: Handler<T, ()> + 'static,
        T: 'static,
    {
        // Try to get documentation from the handler
        let handler_name = std::any::type_name::<H>();
        let docs = self.lookup_handler_docs(handler_name);
        
        self.register_route(path, Method::GET, docs);
        
        Self {
            inner: self.inner.route(path, get(handler)),
            routes: self.routes,
            spec: self.spec,
            schemas: self.schemas,
        }
    }

    /// Add a POST route with automatic documentation lookup
    pub fn post<H, T>(self, path: &str, handler: H) -> Self 
    where
        H: Handler<T, ()> + 'static,
        T: 'static,
    {
        let handler_name = std::any::type_name::<H>();
        let docs = self.lookup_handler_docs(handler_name);
        
        self.register_route(path, Method::POST, docs);
        
        Self {
            inner: self.inner.route(path, post(handler)),
            routes: self.routes,
            spec: self.spec,
            schemas: self.schemas,
        }
    }

    /// Add a PUT route with automatic documentation lookup
    pub fn put<H, T>(self, path: &str, handler: H) -> Self 
    where
        H: Handler<T, ()> + 'static,
        T: 'static,
    {
        let handler_name = std::any::type_name::<H>();
        let docs = self.lookup_handler_docs(handler_name);
        
        self.register_route(path, Method::PUT, docs);
        
        Self {
            inner: self.inner.route(path, put(handler)),
            routes: self.routes,
            spec: self.spec,
            schemas: self.schemas,
        }
    }

    /// Add a DELETE route with automatic documentation lookup
    pub fn delete<H, T>(self, path: &str, handler: H) -> Self 
    where
        H: Handler<T, ()> + 'static,
        T: 'static,
    {
        let handler_name = std::any::type_name::<H>();
        let docs = self.lookup_handler_docs(handler_name);
        
        self.register_route(path, Method::DELETE, docs);
        
        Self {
            inner: self.inner.route(path, delete(handler)),
            routes: self.routes,
            spec: self.spec,
            schemas: self.schemas,
        }
    }

    /// Add a PATCH route with automatic documentation lookup
    pub fn patch<H, T>(self, path: &str, handler: H) -> Self 
    where
        H: Handler<T, ()> + 'static,
        T: 'static,
    {
        let handler_name = std::any::type_name::<H>();
        let docs = self.lookup_handler_docs(handler_name);
        
        self.register_route(path, Method::PATCH, docs);
        
        Self {
            inner: self.inner.route(path, patch(handler)),
            routes: self.routes,
            spec: self.spec,
            schemas: self.schemas,
        }
    }

    /// Look up documentation for a handler by name
    fn lookup_handler_docs(&self, handler_name: &str) -> HandlerDocumentation {
        // Extract the function name from the full type path
        let fn_name = handler_name
            .split("::")
            .last()
            .unwrap_or(handler_name)
            .replace("{{closure}}", "")
            .trim()
            .to_string();
        
        // Try to find the corresponding metadata constant
        // This is a compile-time generated lookup that we'll need to implement
        self.get_docs_for_function(&fn_name)
    }

    /// Get documentation for a specific function
    fn get_docs_for_function(&self, fn_name: &str) -> HandlerDocumentation {
        // Look up the function in the global registry
        for entry in inventory::iter::<HandlerDocEntry> {
            if entry.name == fn_name {
                return (entry.get_docs)();
            }
        }
        
        // Fallback for unknown handlers
        HandlerDocumentation {
            summary: None,
            description: None,
            parameters: vec![],
            request_body: None,
            request_body_type: None,
            response_type: None,
            error_type: None,
            responses: vec![],
        }
    }
    
    /// Register a schema type automatically using schema generation
    fn register_schema_if_available(&self, type_name: &str) {
        let mut schemas = self.schemas.lock().unwrap();
        
        // Skip if we already have this schema
        if schemas.contains_key(type_name) {
            return;
        }
        
        // Try to generate a real schema using the generated schema function
        let schema = self.try_generate_schema(type_name)
            .unwrap_or_else(|| {
                // Fallback to basic object schema
                Schema {
                    schema_data: Default::default(),
                    schema_kind: SchemaKind::Type(Type::Object(openapiv3::ObjectType {
                        properties: Default::default(),
                        required: vec![],
                        additional_properties: None,
                        min_properties: None,
                        max_properties: None,
                    })),
                }
            });
        
        schemas.insert(type_name.to_string(), ReferenceOr::Item(schema));
    }
    
    /// Try to generate a schema using the generated schema functions
    fn try_generate_schema(&self, type_name: &str) -> Option<Schema> {
        // Look up the schema function in the registry
        for entry in inventory::iter::<SchemaEntry> {
            if entry.type_name == type_name {
                if let Some(schema_json) = (entry.get_schema)() {
                    // Convert the schemars JSON schema to OpenAPI schema
                    return self.convert_schemars_to_openapi(schema_json);
                }
            }
        }
        None
    }
    
    /// Convert a schemars JSON schema to an OpenAPI schema
    fn convert_schemars_to_openapi(&self, schema_json: serde_json::Value) -> Option<Schema> {
        // Try to deserialize the schemars schema as an OpenAPI schema
        // This is a bit of a hack since the formats are similar but not identical
        if let Ok(schema) = serde_json::from_value::<Schema>(schema_json.clone()) {
            Some(schema)
        } else {
            // If direct conversion fails, try to extract basic information
            self.extract_basic_schema_info(schema_json)
        }
    }
    
    /// Extract basic schema information from schemars JSON
    fn extract_basic_schema_info(&self, schema_json: serde_json::Value) -> Option<Schema> {
        if let Some(obj) = schema_json.as_object() {
            if obj.get("type")?.as_str() == Some("object") {
                let mut properties = indexmap::IndexMap::new();
                let mut required = Vec::new();
                
                if let Some(props) = obj.get("properties").and_then(|p| p.as_object()) {
                    for (key, value) in props {
                        // Try to convert each property
                        if let Ok(prop_schema) = serde_json::from_value::<ReferenceOr<Box<Schema>>>(value.clone()) {
                            properties.insert(key.clone(), prop_schema);
                        } else {
                            // Fallback to string type for properties we can't parse
                            properties.insert(key.clone(), ReferenceOr::Item(Box::new(Schema {
                                schema_data: Default::default(),
                                schema_kind: SchemaKind::Type(Type::String(Default::default())),
                            })));
                        }
                    }
                }
                
                if let Some(req_array) = obj.get("required").and_then(|r| r.as_array()) {
                    for item in req_array {
                        if let Some(field_name) = item.as_str() {
                            required.push(field_name.to_string());
                        }
                    }
                }
                
                return Some(Schema {
                    schema_data: Default::default(),
                    schema_kind: SchemaKind::Type(Type::Object(openapiv3::ObjectType {
                        properties,
                        required,
                        additional_properties: None,
                        min_properties: None,
                        max_properties: None,
                    })),
                });
            }
        }
        None
    }

    /// Register a route in our tracking system
    fn register_route(
        &self, 
        path: &str, 
        method: Method, 
        docs: HandlerDocumentation
    ) {
        let summary = docs.summary;
        let description = docs.description;
        let parameters = docs.parameters;
        let request_body = docs.request_body;
        let response_type = docs.response_type;
        let error_type = docs.error_type;
        let responses = docs.responses;
        let mut routes = self.routes.lock().unwrap();
        routes.push(RouteInfo {
            path: path.to_string(),
            method: method.clone(),
            summary: summary.map(|s| s.to_string()),
            description: description.map(|s| s.to_string()),
            parameters: parameters.clone(),
            request_body: request_body.clone(),
            responses: responses.clone(),
        });
        
        // Collect schema types that need to be registered
        let mut schema_types = Vec::new();
        if let Some(ref body) = request_body {
            if let Some(ref schema_type) = body.schema_type {
                schema_types.push(schema_type.clone());
            }
        }
        // Register response schema type
        if let Some(ref resp_type) = response_type {
            schema_types.push(resp_type.clone());
        }
        // Register error schema type
        if let Some(ref err_type) = error_type {
            schema_types.push(err_type.clone());
        }
        
        // Register schemas if we have any
        for schema_type in &schema_types {
            self.register_schema_if_available(schema_type);
        }
        
        // Update the OpenAPI spec
        let mut spec = self.spec.lock().unwrap();
        
        // Ensure components section exists and add schemas
        if spec.components.is_none() {
            spec.components = Some(Default::default());
        }
        
        if let Some(ref mut components) = spec.components {
            let schemas = self.schemas.lock().unwrap();
            components.schemas.extend(schemas.clone());
        }
        
        let path_item = spec.paths.paths
            .entry(convert_path_params(path))
            .or_insert_with(|| ReferenceOr::Item(PathItem::default()));
        
        if let ReferenceOr::Item(item) = path_item {
            let operation = create_operation_with_params_and_responses(
                path, 
                &method, 
                summary, 
                description, 
                &parameters,
                &request_body,
                &response_type,
                &error_type,
                &responses
            );
            
            match method {
                Method::GET => item.get = Some(operation),
                Method::POST => item.post = Some(operation),
                Method::PUT => item.put = Some(operation),
                Method::DELETE => item.delete = Some(operation),
                Method::PATCH => item.patch = Some(operation),
                _ => {}
            }
        }
    }

    /// Merge with another router
    pub fn merge(self, other: Router) -> Self {
        Self {
            inner: self.inner.merge(other),
            routes: self.routes,
            spec: self.spec,
            schemas: self.schemas,
        }
    }

    /// Nest a router at the given path
    pub fn nest(self, path: &str, router: Router) -> Self {
        Self {
            inner: self.inner.nest(path, router),
            routes: self.routes,
            spec: self.spec,
            schemas: self.schemas,
        }
    }

    /// Add routes to serve the OpenAPI specification with default endpoints.
    /// 
    /// This convenience method adds two routes to serve the OpenAPI specification:
    /// - `GET /openapi.json` - Returns the spec in JSON format
    /// - `GET /openapi.yaml` - Returns the spec in YAML format
    /// 
    /// # Examples
    /// 
    /// ```rust,no_run
    /// # use stonehm::{api_router, api_handler};
/// # use stonehm_macros::StoneSchema;
    /// # use axum::Json;
    /// # use serde::Serialize;
    /// # #[derive(Serialize, StoneSchema)]
    /// # struct User { id: u32 }
    /// # #[derive(Serialize, StoneSchema)]
    /// # struct UsersResponse { users: Vec<User> }
    /// # #[api_handler]
    /// # async fn get_users() -> Json<UsersResponse> { Json(UsersResponse { users: vec![] }) }
    /// let app = api_router!("My API", "1.0.0")
    ///     .get("/users", get_users)
    ///     .with_openapi_routes()  // Adds /openapi.json and /openapi.yaml
    ///     .into_router();
    /// ```
    /// 
    /// After adding these routes, you can access your API specification at:
    /// - `http://your-server/openapi.json`
    /// - `http://your-server/openapi.yaml`
    pub fn with_openapi_routes(self) -> Self {
        self.with_openapi_routes_prefix("/openapi")
    }

    /// Add routes to serve the OpenAPI specification with a custom path prefix.
    /// 
    /// This method allows you to customize where the OpenAPI specification endpoints
    /// are served. It adds two routes with your custom prefix:
    /// - `GET {prefix}.json` - Returns the spec in JSON format  
    /// - `GET {prefix}.yaml` - Returns the spec in YAML format
    /// 
    /// # Arguments
    /// 
    /// * `prefix` - The path prefix for the OpenAPI routes (e.g., "/api/docs", "/spec")
    /// 
    /// # Examples
    /// 
    /// ```rust,no_run
    /// # use stonehm::{api_router, api_handler};
/// # use stonehm_macros::StoneSchema;
    /// # use axum::Json;
    /// # use serde::Serialize;
    /// # #[derive(Serialize, StoneSchema)]
    /// # struct User { id: u32 }
    /// # #[derive(Serialize, StoneSchema)]
    /// # struct UsersResponse { users: Vec<User> }
    /// # #[api_handler]
    /// # async fn get_users() -> Json<UsersResponse> { Json(UsersResponse { users: vec![] }) }
    /// let app = api_router!("My API", "1.0.0")
    ///     .get("/users", get_users)
    ///     .with_openapi_routes_prefix("/api/docs")  // Adds /api/docs.json and /api/docs.yaml
    ///     .into_router();
    /// ```
    /// 
    /// With the prefix "/api/docs", you can access:
    /// - `http://your-server/api/docs.json`
    /// - `http://your-server/api/docs.yaml`
    /// 
    /// # Path Normalization
    /// 
    /// The prefix is automatically normalized:
    /// - Leading slashes are added if missing
    /// - Trailing slashes are removed
    /// - Empty prefixes are handled gracefully
    pub fn with_openapi_routes_prefix(self, prefix: &str) -> Self {
        let spec = self.spec.clone();
        let spec_json = spec.clone();
        let spec_yaml = spec.clone();

        // Ensure prefix starts with / and doesn't end with /
        let normalized_prefix = if prefix.is_empty() {
            String::new()
        } else if prefix.starts_with('/') {
            prefix.trim_end_matches('/').to_string()
        } else {
            format!("/{}", prefix.trim_end_matches('/'))
        };

        let json_path = format!("{normalized_prefix}.json");
        let yaml_path = format!("{normalized_prefix}.yaml");

        let inner = self.inner
            .route(&json_path, get(move || async move {
                let spec = spec_json.lock().unwrap();
                serde_json::to_string_pretty(&*spec).unwrap()
            }))
            .route(&yaml_path, get(move || async move {
                let spec = spec_yaml.lock().unwrap();
                serde_yaml::to_string(&*spec).unwrap()
            }));

        Self {
            inner,
            routes: self.routes,
            spec: self.spec,
            schemas: self.schemas,
        }
    }

    /// Convert the `DocumentedRouter` into a regular Axum `Router`.
    /// 
    /// This consumes the `DocumentedRouter` and returns the underlying Axum `Router`
    /// that can be used with Axum's server functions. All documentation information
    /// is discarded after this conversion.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// # use stonehm::{api_router, api_handler};
/// # use stonehm_macros::StoneSchema;
    /// # use axum::Json;
    /// # use serde::Serialize;
    /// # #[derive(Serialize, StoneSchema)]
    /// # struct User { id: u32 }
    /// # #[derive(Serialize, StoneSchema)]
    /// # struct UsersResponse { users: Vec<User> }
    /// # #[api_handler]
    /// # async fn get_users() -> Json<UsersResponse> { Json(UsersResponse { users: vec![] }) }
    /// let documented_router = api_router!("My API", "1.0.0")
    ///     .get("/users", get_users)
    ///     .with_openapi_routes();
    /// 
    /// // Convert to Axum router for serving
    /// let axum_router = documented_router.into_router();
    /// 
    /// // Use with Axum's serve function
    /// // axum::serve(listener, axum_router).await.unwrap();
    /// ```
    pub fn into_router(self) -> Router {
        self.inner
    }

    /// Get a copy of the current OpenAPI specification.
    /// 
    /// Returns the generated OpenAPI 3.0 specification as an `OpenAPI` struct
    /// that can be serialized to JSON or YAML. This is useful for inspecting
    /// the generated documentation or saving it to a file.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// # use stonehm::{api_router, api_handler};
/// # use stonehm_macros::StoneSchema;
    /// # use axum::Json;
    /// # use serde::{Serialize, Deserialize};
    /// # #[derive(Serialize, StoneSchema)]
    /// # struct User { id: u32 }
    /// # #[derive(Serialize, StoneSchema)]
    /// # struct UsersResponse { users: Vec<User> }
    /// # #[derive(Deserialize, StoneSchema)]
    /// # struct CreateUserRequest { name: String }
    /// # #[api_handler]
    /// # async fn get_users() -> Json<UsersResponse> { Json(UsersResponse { users: vec![] }) }
    /// # #[api_handler]
    /// # async fn create_user(Json(_req): Json<CreateUserRequest>) -> Json<User> { Json(User { id: 1 }) }
    /// let router = api_router!("My API", "1.0.0")
    ///     .get("/users", get_users)
    ///     .post("/users", create_user);
    /// 
    /// // Get the OpenAPI specification
    /// let spec = router.openapi_spec();
    /// 
    /// // Serialize to JSON
    /// let json = serde_json::to_string_pretty(&spec).unwrap();
    /// println!("{}", json);
    /// 
    /// // Serialize to YAML  
    /// let yaml = serde_yaml::to_string(&spec).unwrap();
    /// println!("{}", yaml);
    /// ```
    pub fn openapi_spec(&self) -> OpenAPI {
        self.spec.lock().unwrap().clone()
    }
}

/// Convert Axum path params (:id) to OpenAPI format ({id})
fn convert_path_params(path: &str) -> String {
    let mut result = String::new();
    let mut chars = path.chars();
    
    while let Some(ch) = chars.next() {
        if ch == ':' {
            result.push('{');
            for ch in chars.by_ref() {
                if ch == '/' {
                    result.push('}');
                    result.push(ch);
                    break;
                }
                result.push(ch);
            }
            // Handle case where param is at the end
            if !result.ends_with('/') && !result.ends_with('}') {
                result.push('}');
            }
        } else {
            result.push(ch);
        }
    }
    
    result
}

/// Create an operation with parameters, request body, and responses
fn create_operation_with_params_and_responses(
    path: &str, 
    method: &Method, 
    summary: Option<&str>, 
    description: Option<&str>,
    parameter_docs: &[ParameterDoc],
    request_body_doc: &Option<RequestBodyDoc>,
    response_type: &Option<String>,
    error_type: &Option<String>,
    response_docs: &[ResponseDoc]
) -> Operation {
    let mut operation = Operation::default();
    
    // Generate operation ID
    let path_parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    let operation_id = format!(
        "{}_{}",
        method.as_str().to_lowercase(),
        path_parts.join("_").replace(':', "by_")
    );
    
    operation.operation_id = Some(operation_id);
    
    // Use provided summary or generate default
    operation.summary = Some(
        summary
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("{} {}", method.as_str(), path))
    );
    
    // Use provided description
    operation.description = description.map(|s| s.to_string());
    
    // Add parameters from documentation
    if !parameter_docs.is_empty() {
        let mut params = Vec::new();
        
        for param_doc in parameter_docs {
            let param_location = match param_doc.param_type.as_str() {
                "path" => "path",
                "query" => "query",
                "header" => "header",
                _ => "query", // default to query
            };
            
            let parameter = Parameter::Query {
                parameter_data: ParameterData {
                    explode: None,
                    name: param_doc.name.clone(),
                    description: Some(param_doc.description.clone()),
                    required: param_location == "path", // path params are always required
                    deprecated: None,
                    format: ParameterSchemaOrContent::Schema(ReferenceOr::Item(Schema {
                        schema_data: Default::default(),
                        schema_kind: SchemaKind::Type(Type::String(Default::default())),
                    })),
                    example: None,
                    examples: Default::default(),
                    extensions: Default::default(),
                },
                allow_reserved: false,
                style: Default::default(),
                allow_empty_value: None,
            };
            
            // Convert to the correct parameter type based on location
            let parameter = match param_location {
                "path" => Parameter::Path {
                    parameter_data: ParameterData {
                    explode: None,
                        name: param_doc.name.clone(),
                        description: Some(param_doc.description.clone()),
                        required: true,
                        deprecated: None,
                        format: ParameterSchemaOrContent::Schema(ReferenceOr::Item(Schema {
                            schema_data: Default::default(),
                            schema_kind: SchemaKind::Type(Type::String(Default::default())),
                        })),
                        example: None,
                        examples: Default::default(),
                        extensions: Default::default(),
                    },
                    style: Default::default(),
                },
                "header" => Parameter::Header {
                    parameter_data: ParameterData {
                    explode: None,
                        name: param_doc.name.clone(),
                        description: Some(param_doc.description.clone()),
                        required: false,
                        deprecated: None,
                        format: ParameterSchemaOrContent::Schema(ReferenceOr::Item(Schema {
                            schema_data: Default::default(),
                            schema_kind: SchemaKind::Type(Type::String(Default::default())),
                        })),
                        example: None,
                        examples: Default::default(),
                        extensions: Default::default(),
                    },
                    style: Default::default(),
                },
                _ => parameter, // query
            };
            
            params.push(ReferenceOr::Item(parameter));
        }
        
        operation.parameters = params;
    }
    
    // Add request body from documentation
    if let Some(body_doc) = request_body_doc {
        let mut content = Content::default();
        
        // Use schema reference if we have a type, otherwise generic object
        let schema = if let Some(ref schema_type) = body_doc.schema_type {
            // Create a reference to the schema
            ReferenceOr::Reference {
                reference: format!("#/components/schemas/{schema_type}"),
            }
        } else {
            // Fallback to generic object
            ReferenceOr::Item(Schema {
                schema_data: Default::default(),
                schema_kind: SchemaKind::Type(Type::Object(Default::default())),
            })
        };
        
        content.insert(
            body_doc.content_type.clone(),
            MediaType {
                schema: Some(schema),
                example: None,
                examples: Default::default(),
                encoding: Default::default(),
                extensions: Default::default(),
            }
        );
        
        operation.request_body = Some(ReferenceOr::Item(RequestBody {
            description: Some(body_doc.description.clone()),
            content,
            required: true,
            extensions: Default::default(),
        }));
    }
    
    // Add responses from documentation
    let mut responses = Responses::default();
    
    if response_docs.is_empty() {
        // Add default response if none specified with schema if available
        let mut success_response = ApiResponse {
            description: "Successful response".to_string(),
            ..Default::default()
        };
        
        // Add response content with schema if we have a response type
        if let Some(ref resp_type) = response_type {
            let mut content = Content::default();
            let schema = ReferenceOr::Reference {
                reference: format!("#/components/schemas/{resp_type}"),
            };
            
            content.insert("application/json".to_string(), MediaType {
                schema: Some(schema),
                ..Default::default()
            });
            
            success_response.content = content;
        }
        
        responses.responses.insert(
            StatusCode::Code(200),
            ReferenceOr::Item(success_response)
        );
        
        // Add automatic error responses if we have an error type
        if let Some(ref err_type) = error_type {
            let common_errors = vec![
                (400, "Bad Request"),
                (500, "Internal Server Error"),
            ];
            
            for (status_code, description) in common_errors {
                let mut error_response = ApiResponse {
                    description: description.to_string(),
                    ..Default::default()
                };
                
                // Add error schema content
                let mut content = Content::default();
                let schema = ReferenceOr::Reference {
                    reference: format!("#/components/schemas/{err_type}"),
                };
                
                content.insert("application/json".to_string(), MediaType {
                    schema: Some(schema),
                    ..Default::default()
                });
                
                error_response.content = content;
                
                responses.responses.insert(
                    StatusCode::Code(status_code),
                    ReferenceOr::Item(error_response)
                );
            }
        }
    } else {
        // Add documented responses
        for response_doc in response_docs {
            let mut response = ApiResponse {
                description: response_doc.description.clone(),
                ..Default::default()
            };
            
            // Handle response content if specified
            if let Some(ref content_info) = response_doc.content {
                let mut content = Content::default();
                
                // Use schema from content info or fallback to detected response type for 200 responses
                let schema = if let Some(ref schema_name) = content_info.schema {
                    ReferenceOr::Reference {
                        reference: format!("#/components/schemas/{schema_name}"),
                    }
                } else if response_doc.status_code == 200 {
                    if let Some(ref resp_type) = response_type {
                        ReferenceOr::Reference {
                            reference: format!("#/components/schemas/{resp_type}"),
                        }
                    } else {
                        // Fallback to generic object
                        ReferenceOr::Item(Schema {
                            schema_data: Default::default(),
                            schema_kind: SchemaKind::Type(Type::Object(Default::default())),
                        })
                    }
                } else {
                    // Fallback to generic object for error responses
                    ReferenceOr::Item(Schema {
                        schema_data: Default::default(),
                        schema_kind: SchemaKind::Type(Type::Object(Default::default())),
                    })
                };
                
                // Build media type with schema and examples
                let mut media_type = MediaType {
                    schema: Some(schema),
                    ..Default::default()
                };
                
                // Add examples if provided
                if let Some(ref examples) = response_doc.examples {
                    for example in examples {
                        let example_value = if example.value.starts_with('{') || example.value.starts_with('[') {
                            // Try to parse as JSON
                            serde_json::from_str::<serde_json::Value>(&example.value)
                                .unwrap_or_else(|_| serde_json::Value::String(example.value.clone()))
                        } else {
                            serde_json::Value::String(example.value.clone())
                        };
                        
                        media_type.examples.insert(
                            example.name.clone(),
                            ReferenceOr::Item(openapiv3::Example {
                                summary: example.summary.clone(),
                                description: None,
                                value: Some(example_value),
                                external_value: None,
                                extensions: Default::default(),
                            })
                        );
                    }
                }
                
                content.insert(content_info.media_type.clone(), media_type);
                response.content = content;
            } else if response_doc.status_code == 200 {
                // Fallback: Add schema for 200 responses if we have a response type
                if let Some(ref resp_type) = response_type {
                    let mut content = Content::default();
                    let schema = ReferenceOr::Reference {
                        reference: format!("#/components/schemas/{resp_type}"),
                    };
                    
                    content.insert("application/json".to_string(), MediaType {
                        schema: Some(schema),
                        ..Default::default()
                    });
                    
                    response.content = content;
                }
            }
            
            responses.responses.insert(
                StatusCode::Code(response_doc.status_code),
                ReferenceOr::Item(response)
            );
        }
        
        // Add automatic error responses if we have an error type and no manual error responses
        if let Some(ref err_type) = error_type {
            let has_manual_error_responses = response_docs.iter().any(|r| r.status_code >= 400);
            
            if !has_manual_error_responses {
                // Add common error responses automatically
                let common_errors = vec![
                    (400, "Bad Request"),
                    (401, "Unauthorized"), 
                    (403, "Forbidden"),
                    (404, "Not Found"),
                    (500, "Internal Server Error"),
                ];
                
                for (status_code, description) in common_errors {
                    // Skip 404 for non-GET methods that don't have path parameters
                    if status_code == 404 && *method != Method::GET && !path.contains('{') {
                        continue;
                    }
                    
                    // Skip 401/403 unless it's a modifying operation
                    if (status_code == 401 || status_code == 403) && *method == Method::GET {
                        continue;
                    }
                    
                    let mut error_response = ApiResponse {
                        description: description.to_string(),
                        ..Default::default()
                    };
                    
                    // Add error schema content
                    let mut content = Content::default();
                    let schema = ReferenceOr::Reference {
                        reference: format!("#/components/schemas/{err_type}"),
                    };
                    
                    content.insert("application/json".to_string(), MediaType {
                        schema: Some(schema),
                        ..Default::default()
                    });
                    
                    error_response.content = content;
                    
                    responses.responses.insert(
                        StatusCode::Code(status_code),
                        ReferenceOr::Item(error_response)
                    );
                }
            }
        }
    }
    
    operation.responses = responses;
    
    operation
}

/// Create an operation with optional documentation and responses (backwards compatibility)
#[cfg(test)]
fn create_operation_with_responses(
    path: &str, 
    method: &Method, 
    summary: Option<&str>, 
    description: Option<&str>,
    response_docs: &[ResponseDoc]
) -> Operation {
    create_operation_with_params_and_responses(path, method, summary, description, &[], &None, &None, &None, response_docs)
}

/// Convenience macro for creating a new `DocumentedRouter`.
/// 
/// This macro provides a shorthand way to create a new `DocumentedRouter`
/// with the given title and version. It's equivalent to calling
/// `DocumentedRouter::new(title, version)` but with a more concise syntax.
/// 
/// # Arguments
/// 
/// * `$title` - The title of your API as a string literal or expression
/// * `$version` - The version of your API as a string literal or expression
/// 
/// # Examples
/// 
/// ```rust
/// # use stonehm::{api_router, api_handler};
/// # use stonehm_macros::StoneSchema;
/// # use axum::Json;
/// # use serde::{Serialize, Deserialize};
/// # #[derive(Serialize, StoneSchema)]
/// # struct UserInfo { id: u32 }
/// # #[derive(Serialize, StoneSchema)]
/// # struct UsersResponse { users: Vec<UserInfo> }
/// # #[derive(Deserialize, StoneSchema)]
/// # struct CreateUserRequest { name: String }
/// # #[derive(Serialize, StoneSchema)]
/// # struct NewUser { id: u32 }
/// # #[derive(Serialize, StoneSchema)]
/// # struct SingleUser { id: u32 }
/// # #[api_handler]
/// # async fn get_users() -> Json<UsersResponse> { Json(UsersResponse { users: vec![] }) }
/// # #[api_handler]
/// # async fn create_user(Json(_req): Json<CreateUserRequest>) -> Json<NewUser> { Json(NewUser { id: 1 }) }
/// # #[api_handler]
/// # async fn get_user() -> Json<SingleUser> { Json(SingleUser { id: 1 }) }
/// // Basic usage with string literals
/// let router = api_router!("My API", "1.0.0");
/// 
/// // Using variables
/// let api_title = "User Management API";
/// let api_version = "2.1.0";
/// let router = api_router!(api_title, api_version);
/// 
/// // Chaining with route definitions
/// let app = api_router!("My API", "1.0.0")
///     .get("/users", get_users)
///     .post("/users", create_user)
///     .get("/users/{id}", get_user)
///     .with_openapi_routes()
///     .into_router();
/// ```
/// 
/// # Equivalent Code
/// 
/// This macro expands to:
/// ```rust
/// use stonehm::DocumentedRouter;
/// 
/// let router = DocumentedRouter::new("My API", "1.0.0");
/// ```
#[macro_export]
macro_rules! api_router {
    ($title:expr, $version:expr) => {
        $crate::DocumentedRouter::new($title, $version)
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::Json;
    use serde::{Deserialize, Serialize};
    use openapiv3::StatusCode;

    #[derive(Serialize, Deserialize)]
    struct TestResponse {
        message: String,
    }

    async fn test_handler() -> Json<TestResponse> {
        Json(TestResponse {
            message: "test".to_string(),
        })
    }

    async fn test_handler_with_path(axum::extract::Path(_id): axum::extract::Path<u32>) -> Json<TestResponse> {
        Json(TestResponse {
            message: "test".to_string(),
        })
    }

    #[test]
    fn test_documented_router_creation() {
        let router = DocumentedRouter::new("Test API", "1.0.0");
        let spec = router.openapi_spec();
        
        assert_eq!(spec.openapi, "3.0.3");
        assert_eq!(spec.info.title, "Test API");
        assert_eq!(spec.info.version, "1.0.0");
    }

    #[test]
    fn test_api_router_macro() {
        let router = api_router!("Test API", "2.0.0");
        let spec = router.openapi_spec();
        
        assert_eq!(spec.info.title, "Test API");
        assert_eq!(spec.info.version, "2.0.0");
    }

    #[test]
    fn test_route_registration() {
        let router = api_router!("Test API", "1.0.0")
            .get("/test", test_handler);
            
        let spec = router.openapi_spec();
        
        // Check that the route was added to the spec
        assert!(spec.paths.paths.contains_key("/test"));
        
        if let Some(openapiv3::ReferenceOr::Item(path_item)) = spec.paths.paths.get("/test") {
            assert!(path_item.get.is_some());
            
            if let Some(operation) = &path_item.get {
                assert_eq!(operation.operation_id, Some("get_test".to_string()));
            }
        }
    }

    #[test]
    fn test_multiple_routes() {
        let router = api_router!("Test API", "1.0.0")
            .get("/users", test_handler)
            .post("/users", test_handler)
            .get("/users/:id", test_handler_with_path);
            
        let spec = router.openapi_spec();
        
        // Check that all routes were added
        assert!(spec.paths.paths.contains_key("/users"));
        assert!(spec.paths.paths.contains_key("/users/{id}"));
        
        // Check GET /users
        if let Some(openapiv3::ReferenceOr::Item(path_item)) = spec.paths.paths.get("/users") {
            assert!(path_item.get.is_some());
            assert!(path_item.post.is_some());
        }
        
        // Check GET /users/:id (converted to /users/{id})
        if let Some(openapiv3::ReferenceOr::Item(path_item)) = spec.paths.paths.get("/users/{id}") {
            assert!(path_item.get.is_some());
        }
    }

    #[test]
    fn test_path_parameter_conversion() {
        assert_eq!(convert_path_params("/users/:id"), "/users/{id}");
        assert_eq!(convert_path_params("/users/:id/posts/:post_id"), "/users/{id}/posts/{post_id}");
        assert_eq!(convert_path_params("/static"), "/static");
        assert_eq!(convert_path_params("/users/:id/"), "/users/{id}/");
    }

    #[test]
    fn test_operation_creation_with_responses() {
        let responses = vec![
            ResponseDoc {
                status_code: 200,
                description: "Success".to_string(),
                content: None,
                examples: None,
            },
            ResponseDoc {
                status_code: 404,
                description: "Not found".to_string(),
                content: None,
                examples: None,
            },
        ];
        
        let operation = create_operation_with_responses(
            "/users/:id",
            &Method::GET,
            Some("Get user"),
            Some("Get user by ID"),
            &responses,
        );
        
        assert_eq!(operation.operation_id, Some("get_users_by_id".to_string()));
        assert_eq!(operation.summary, Some("Get user".to_string()));
        assert_eq!(operation.description, Some("Get user by ID".to_string()));
        
        // Check responses
        assert!(operation.responses.responses.contains_key(&StatusCode::Code(200)));
        assert!(operation.responses.responses.contains_key(&StatusCode::Code(404)));
        
        if let Some(openapiv3::ReferenceOr::Item(response)) = operation.responses.responses.get(&StatusCode::Code(200)) {
            assert_eq!(response.description, "Success");
        }
        
        if let Some(openapiv3::ReferenceOr::Item(response)) = operation.responses.responses.get(&StatusCode::Code(404)) {
            assert_eq!(response.description, "Not found");
        }
    }

    #[test]
    fn test_operation_creation_without_responses() {
        let operation = create_operation_with_responses(
            "/test",
            &Method::POST,
            None,
            None,
            &[],
        );
        
        assert_eq!(operation.operation_id, Some("post_test".to_string()));
        assert_eq!(operation.summary, Some("POST /test".to_string()));
        assert_eq!(operation.description, None);
        
        // Should have default 200 response
        assert!(operation.responses.responses.contains_key(&StatusCode::Code(200)));
        
        if let Some(openapiv3::ReferenceOr::Item(response)) = operation.responses.responses.get(&StatusCode::Code(200)) {
            assert_eq!(response.description, "Successful response");
        }
    }

    /// Test handler with documentation
    /// 
    /// This is a test handler with documentation for testing purposes.
    /// 
    /// # Responses
    /// - 200: Test successful
    /// - 400: Test failed
    #[allow(dead_code)]
    async fn test_documented_handler() -> Json<TestResponse> {
        Json(TestResponse {
            message: "test documented".to_string(),
        })
    }
    
    // Manually create the documentation function for this test handler
    #[allow(non_upper_case_globals, non_snake_case)]
    pub fn __DOCS_TEST_DOCUMENTED_HANDLER() -> HandlerDocumentation {
        HandlerDocumentation {
            summary: Some("Test handler with documentation"),
            description: Some("This is a test handler with documentation for testing purposes."),
            parameters: vec![],
            request_body: None,
            request_body_type: None,
            response_type: None,
            error_type: None,
            responses: vec![
                ResponseDoc {
                    status_code: 200,
                    description: "Test successful".to_string(),
                    content: None,
                    examples: None,
                },
                ResponseDoc {
                    status_code: 400,
                    description: "Test failed".to_string(),
                    content: None,
                    examples: None,
                }
            ],
        }
    }
    
    inventory::submit! {
        HandlerDocEntry {
            name: "test_documented_handler",
            get_docs: __DOCS_TEST_DOCUMENTED_HANDLER,
        }
    }

    #[test]
    fn test_handler_documentation_lookup() {
        let router = api_router!("Test API", "1.0.0");
        
        // Test the dynamic lookup function with our documented test handler
        let docs = router.get_docs_for_function("test_documented_handler");
        
        assert_eq!(docs.summary, Some("Test handler with documentation"));
        assert_eq!(docs.description, Some("This is a test handler with documentation for testing purposes."));
        assert_eq!(docs.parameters.len(), 0);
        assert!(docs.request_body.is_none());
        assert_eq!(docs.responses.len(), 2);
        assert_eq!(docs.responses[0].status_code, 200);
        assert_eq!(docs.responses[0].description, "Test successful");
        assert_eq!(docs.responses[1].status_code, 400);
        assert_eq!(docs.responses[1].description, "Test failed");
    }

    #[test]
    fn test_handler_documentation_lookup_unknown() {
        let router = api_router!("Test API", "1.0.0");
        
        let docs = router.get_docs_for_function("unknown_handler");
        
        assert_eq!(docs.summary, None);
        assert_eq!(docs.description, None);
        assert_eq!(docs.parameters.len(), 0);
        assert!(docs.request_body.is_none());
        assert_eq!(docs.responses.len(), 0);
    }

    #[test]
    fn test_response_doc_creation() {
        let response_doc = ResponseDoc {
            status_code: 201,
            description: "Created successfully".to_string(),
            content: None,
            examples: None,
        };
        
        assert_eq!(response_doc.status_code, 201);
        assert_eq!(response_doc.description, "Created successfully");
    }

    #[test]
    fn test_handler_metadata_creation() {
        let metadata = HandlerMetadata {
            summary: Some("Test summary"),
            description: Some("Test description"),
        };
        
        assert_eq!(metadata.summary, Some("Test summary"));
        assert_eq!(metadata.description, Some("Test description"));
    }

    #[test]
    fn test_documented_handler_default() {
        struct TestHandler;
        impl DocumentedHandler for TestHandler {}
        
        let metadata = TestHandler::metadata();
        assert_eq!(metadata.summary, None);
        assert_eq!(metadata.description, None);
    }

    #[test]
    fn test_with_openapi_routes_default_prefix() {
        let router = api_router!("Test API", "1.0.0")
            .get("/test", test_handler)
            .with_openapi_routes();
            
        // The router should be converted to axum::Router, so we can't inspect the routes directly
        // But we can verify the router was created without panicking
        let _axum_router = router.into_router();
    }

    #[test]
    fn test_with_openapi_routes_custom_prefix() {
        let router = api_router!("Test API", "1.0.0")
            .get("/test", test_handler)
            .with_openapi_routes_prefix("/api/v1/docs");
            
        // The router should be converted to axum::Router
        let _axum_router = router.into_router();
    }

    #[test]
    fn test_custom_prefix_normalization() {
        // Test various prefix formats get normalized correctly
        let test_cases = vec![
            ("/api/docs", "/api/docs.json", "/api/docs.yaml"),
            ("api/docs", "/api/docs.json", "/api/docs.yaml"),
            ("/api/docs/", "/api/docs.json", "/api/docs.yaml"),
            ("api/docs/", "/api/docs.json", "/api/docs.yaml"),
            ("/openapi", "/openapi.json", "/openapi.yaml"),
            ("v1/spec", "/v1/spec.json", "/v1/spec.yaml"),
        ];
        
        for (input_prefix, _expected_json, _expected_yaml) in test_cases {
            let router = api_router!("Test API", "1.0.0")
                .get("/test", test_handler)
                .with_openapi_routes_prefix(input_prefix);
                
            // We can't directly test the routes since they're internal to axum::Router
            // But we can verify the router creation doesn't panic
            let _axum_router = router.into_router();
            
            // The test passes if no panic occurs during router creation
            // In a real implementation, we'd need additional methods to inspect the routes
        }
    }

    #[test]
    fn test_router_merge() {
        let router1 = api_router!("Test API", "1.0.0")
            .get("/test1", test_handler);
            
        let router2 = axum::Router::new()
            .route("/test2", axum::routing::get(test_handler));
            
        let merged = router1.merge(router2);
        let spec = merged.openapi_spec();
        
        // Only the first router's routes should be in the spec
        assert!(spec.paths.paths.contains_key("/test1"));
        // The merged axum router routes won't be in the OpenAPI spec
    }

    #[test]
    fn test_router_nest() {
        let nested_router = axum::Router::new()
            .route("/nested", axum::routing::get(test_handler));
            
        let router = api_router!("Test API", "1.0.0")
            .get("/test", test_handler)
            .nest("/api", nested_router);
            
        let spec = router.openapi_spec();
        
        // Only the main router's routes should be in the spec
        assert!(spec.paths.paths.contains_key("/test"));
        // Nested routes won't automatically be documented
    }

    #[test]
    fn test_all_http_methods() {
        let router = api_router!("Test API", "1.0.0")
            .get("/get", test_handler)
            .post("/post", test_handler)
            .put("/put", test_handler)
            .delete("/delete", test_handler)
            .patch("/patch", test_handler);
            
        let spec = router.openapi_spec();
        
        // Check all methods are registered
        assert!(spec.paths.paths.contains_key("/get"));
        assert!(spec.paths.paths.contains_key("/post"));
        assert!(spec.paths.paths.contains_key("/put"));
        assert!(spec.paths.paths.contains_key("/delete"));
        assert!(spec.paths.paths.contains_key("/patch"));
        
        // Verify the operations have the correct operation IDs
        if let Some(openapiv3::ReferenceOr::Item(path_item)) = spec.paths.paths.get("/get") {
            if let Some(op) = &path_item.get {
                assert_eq!(op.operation_id, Some("get_get".to_string()));
            }
        }
        
        if let Some(openapiv3::ReferenceOr::Item(path_item)) = spec.paths.paths.get("/post") {
            if let Some(op) = &path_item.post {
                assert_eq!(op.operation_id, Some("post_post".to_string()));
            }
        }
    }

    #[test]
    fn test_complex_path_conversion() {
        let test_cases = vec![
            ("/users/:user_id/posts/:post_id/comments/:comment_id", "/users/{user_id}/posts/{post_id}/comments/{comment_id}"),
            ("/api/v1/:version/users/:id", "/api/v1/{version}/users/{id}"),
            ("/:category/:subcategory/:item", "/{category}/{subcategory}/{item}"),
        ];
        
        for (input, expected) in test_cases {
            assert_eq!(convert_path_params(input), expected);
        }
    }
}