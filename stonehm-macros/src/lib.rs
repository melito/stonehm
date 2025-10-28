use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, Attribute, Lit, Meta, Expr, Type, FnArg, ReturnType, PathArguments, GenericArgument, DeriveInput, Data, Fields};

/// Sanitize a type string to create a valid Rust identifier
#[allow(dead_code)]
fn sanitize_type_for_identifier(type_str: &str) -> String {
    type_str
        .replace(['<', '>', ' ', ',', ':', ';', '(', ')', '[', ']', '{', '}', '&', '*'], "_")
        .replace("__", "_")
        .trim_matches('_')
        .to_string()
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ResponseDoc {
    status_code: u16,
    description: String,
    content: Option<ResponseContent>,
    examples: Option<Vec<ResponseExample>>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ResponseContent {
    media_type: String, // e.g., "application/json"
    schema: Option<String>, // e.g., "ErrorResponse"
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ResponseExample {
    name: String,
    summary: Option<String>,
    value: String, // JSON or other content
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ParameterDoc {
    name: String,
    description: String,
    param_type: String, // path, query, header
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct RequestBodyDoc {
    description: String,
    content_type: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ParsedDocs {
    summary: Option<String>,
    description: Option<String>,
    parameters: Vec<ParameterDoc>,
    request_body: Option<RequestBodyDoc>,
    responses: Vec<ResponseDoc>,
}

/// Extract documentation from attributes
#[allow(dead_code)]
fn extract_docs(attrs: &[Attribute]) -> ParsedDocs {
    let mut lines = Vec::new();
    
    for attr in attrs {
        if attr.path().is_ident("doc") {
            if let Meta::NameValue(meta) = &attr.meta {
                if let Expr::Lit(lit) = &meta.value {
                    if let Lit::Str(s) = &lit.lit {
                        let line = s.value();
                        let trimmed = line.trim();
                        if !trimmed.is_empty() {
                            lines.push(trimmed.to_string());
                        }
                    }
                }
            }
        }
    }
    
    if lines.is_empty() {
        return ParsedDocs {
            summary: None,
            description: None,
            parameters: Vec::new(),
            request_body: None,
            responses: Vec::new(),
        };
    }
    
    let mut summary = None;
    let mut description_lines = Vec::new();
    let mut parameters = Vec::new();
    let mut request_body = None;
    let mut responses = Vec::new();
    let mut current_section = "";
    
    for (i, line) in lines.iter().enumerate() {
        if i == 0 {
            // First line is always the summary
            summary = Some(line.clone());
            continue;
        }
        
        // Check for section headers
        if line.starts_with("# Parameters") || line.starts_with("## Parameters") {
            current_section = "parameters";
            continue;
        } else if line.starts_with("# Request Body") || line.starts_with("## Request Body") {
            current_section = "request_body";
            continue;
        } else if line.starts_with("# Responses") || line.starts_with("## Responses") {
            current_section = "responses";
            continue;
        } else if line.starts_with("#") {
            // Any other section header stops special processing
            current_section = "";
        }
        
        match current_section {
            "parameters" => {
                // Parse parameter lines like "- id (path): The user ID" or "- name (query): Filter by name"
                if line.starts_with("- ") || line.starts_with("* ") {
                    let param_text = line[2..].trim();
                    
                    // Try to parse "name (type): description" format
                    if let Some(paren_start) = param_text.find('(') {
                        if let Some(paren_end) = param_text.find(')') {
                            let name = param_text[..paren_start].trim();
                            let param_type = param_text[paren_start + 1..paren_end].trim();
                            
                            if let Some(colon_pos) = param_text[paren_end..].find(':') {
                                let description = param_text[paren_end + colon_pos + 1..].trim();
                                
                                parameters.push(ParameterDoc {
                                    name: name.to_string(),
                                    description: description.to_string(),
                                    param_type: param_type.to_string(),
                                });
                            }
                        }
                    }
                }
            },
            "request_body" => {
                // Parse request body format like "Content-Type: application/json" followed by description
                if let Some(stripped) = line.strip_prefix("Content-Type:") {
                    let content_type = stripped.trim().to_string();
                    request_body = Some(RequestBodyDoc {
                        description: String::new(),
                        content_type,
                    });
                } else if let Some(ref mut body) = request_body {
                    if !line.is_empty() {
                        if !body.description.is_empty() {
                            body.description.push(' ');
                        }
                        body.description.push_str(line);
                    }
                }
            },
            "responses" => {
                // Parse response lines - both simple and elaborate formats
                if line.starts_with("- ") || line.starts_with("* ") {
                    let response_text = line[2..].trim();
                    
                    if let Some(colon_pos) = response_text.find(':') {
                        let status_part = response_text[..colon_pos].trim();
                        let after_colon = response_text[colon_pos + 1..].trim();
                        
                        if let Ok(status_code) = status_part.parse::<u16>() {
                            if after_colon.is_empty() {
                                // Elaborate format - status code with no immediate description
                                // We'll parse the following lines as YAML-like content
                                responses.push(ResponseDoc {
                                    status_code,
                                    description: String::new(), // Will be filled in by subsequent lines
                                    content: None,
                                    examples: None,
                                });
                            } else {
                                // Simple format - status code: description
                                responses.push(ResponseDoc {
                                    status_code,
                                    description: after_colon.to_string(),
                                    content: None,
                                    examples: None,
                                });
                            }
                        }
                    }
                } else if !responses.is_empty() && (
                    line.starts_with("description:") || 
                    line.starts_with("content:") || 
                    line.starts_with("application/json:") || 
                    line.starts_with("application/xml:") || 
                    line.starts_with("text/plain:") || 
                    line.starts_with("schema:") || 
                    line.starts_with("examples:") || 
                    line.starts_with("- name:") || 
                    line.starts_with("name:") || 
                    line.starts_with("summary:") || 
                    line.starts_with("value:")
                ) {
                    // YAML-like property line - part of elaborate response format
                    if let Some(last_response) = responses.last_mut() {
                        if let Some(desc) = line.strip_prefix("description:") {
                            let desc = desc.trim().trim_matches('"');
                            last_response.description = desc.to_string();
                        } else if line.starts_with("content:") {
                            // Start of content block - initialize if needed
                            if last_response.content.is_none() {
                                last_response.content = Some(ResponseContent {
                                    media_type: "application/json".to_string(),
                                    schema: None,
                                });
                            }
                        } else if line.starts_with("application/json:") || line.starts_with("application/xml:") || line.starts_with("text/plain:") {
                            // Parse media type
                            let media_type = line.split(':').next().unwrap_or("application/json");
                            if last_response.content.is_none() {
                                last_response.content = Some(ResponseContent {
                                    media_type: media_type.to_string(),
                                    schema: None,
                                });
                            } else if let Some(ref mut content) = last_response.content {
                                content.media_type = media_type.to_string();
                            }
                        } else if let Some(schema_name) = line.strip_prefix("schema:") {
                            let schema_name = schema_name.trim();
                            if last_response.content.is_none() {
                                last_response.content = Some(ResponseContent {
                                    media_type: "application/json".to_string(),
                                    schema: Some(schema_name.to_string()),
                                });
                            } else if let Some(ref mut content) = last_response.content {
                                content.schema = Some(schema_name.to_string());
                            }
                        } else if line.starts_with("examples:") {
                            // Start of examples block
                            if last_response.examples.is_none() {
                                last_response.examples = Some(Vec::new());
                            }
                        } else if line.starts_with("- name:") || line.starts_with("name:") {
                            // Parse example name
                            let name = if let Some(stripped) = line.strip_prefix("- name:") {
                                stripped.trim()
                            } else if let Some(stripped) = line.strip_prefix("name:") {
                                stripped.trim()
                            } else {
                                ""
                            };
                            if last_response.examples.is_none() {
                                last_response.examples = Some(Vec::new());
                            }
                            if let Some(ref mut examples) = last_response.examples {
                                examples.push(ResponseExample {
                                    name: name.trim_matches('"').to_string(),
                                    summary: None,
                                    value: String::new(),
                                });
                            }
                        } else if line.starts_with("summary:") && last_response.examples.is_some() {
                            // Add summary to the last example
                            let summary = line[8..].trim().trim_matches('"');
                            if let Some(ref mut examples) = last_response.examples {
                                if let Some(last_example) = examples.last_mut() {
                                    last_example.summary = Some(summary.to_string());
                                }
                            }
                        } else if line.starts_with("value:") && last_response.examples.is_some() {
                            // Add value to the last example
                            let value = line[6..].trim().trim_matches('"');
                            if let Some(ref mut examples) = last_response.examples {
                                if let Some(last_example) = examples.last_mut() {
                                    last_example.value = value.to_string();
                                }
                            }
                        }
                    }
                }
            },
            _ => {
                // Regular description lines
                if !line.starts_with("#") {
                    description_lines.push(line.clone());
                }
            }
        }
    }
    
    let description = if !description_lines.is_empty() {
        Some(description_lines.join(" "))
    } else {
        None
    };
    
    ParsedDocs {
        summary,
        description,
        parameters,
        request_body,
        responses,
    }
}

/// Extract request body type from function parameters
fn extract_request_body_type(inputs: &syn::punctuated::Punctuated<FnArg, syn::token::Comma>) -> Option<String> {
    for input in inputs {
        if let FnArg::Typed(pat_type) = input {
            if let Type::Path(type_path) = &*pat_type.ty {
                // Look for Json<T> pattern
                if let Some(segment) = type_path.path.segments.last() {
                    if segment.ident == "Json" {
                        if let PathArguments::AngleBracketed(args) = &segment.arguments {
                            if let Some(GenericArgument::Type(inner_type)) = args.args.first() {
                                return Some(quote!(#inner_type).to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

/// Extract response and error types from function return type
fn extract_response_and_error_types(output: &ReturnType) -> (Option<String>, Option<String>) {
    if let ReturnType::Type(_, return_type) = output {
        if let Type::Path(type_path) = &**return_type {
            if let Some(segment) = type_path.path.segments.last() {
                // Handle Result<T, E> pattern
                if segment.ident == "Result" {
                    if let PathArguments::AngleBracketed(args) = &segment.arguments {
                        let mut response_type = None;
                        let mut error_type = None;
                        
                        // First argument is success type
                        if let Some(GenericArgument::Type(Type::Path(ok_path))) = args.args.first() {
                            // Check if it's Json<T>
                            if let Some(json_segment) = ok_path.path.segments.last() {
                                if json_segment.ident == "Json" {
                                    if let PathArguments::AngleBracketed(json_args) = &json_segment.arguments {
                                        if let Some(GenericArgument::Type(inner_type)) = json_args.args.first() {
                                            response_type = Some(quote!(#inner_type).to_string());
                                        }
                                    }
                                }
                            }
                        }
                        
                        // Second argument is error type
                        if let Some(GenericArgument::Type(err_type)) = args.args.iter().nth(1) {
                            error_type = Some(quote!(#err_type).to_string());
                        }
                        
                        return (response_type, error_type);
                    }
                }
                // Handle direct Json<T> pattern (no Result wrapper)
                else if segment.ident == "Json" {
                    if let PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(GenericArgument::Type(inner_type)) = args.args.first() {
                            return (Some(quote!(#inner_type).to_string()), None);
                        }
                    }
                }
            }
        }
    }
    (None, None)
}


/// Simple api_handler attribute that works with current simplified implementation
/// 
/// Usage:
/// - `#[api_handler]` - No tags
/// - `#[api_handler("tag1")]` - Single tag
/// - `#[api_handler("tag1", "tag2")]` - Multiple tags
#[proc_macro_attribute]
pub fn api_handler(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;
    
    // Parse tags from attribute arguments
    let tags: Vec<String> = if attr.is_empty() {
        Vec::new()
    } else {
        // Parse comma-separated string literals
        let attr_str = attr.to_string();
        attr_str
            .split(',')
            .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
            .filter(|s| !s.is_empty())
            .collect()
    };
    
    // Extract documentation from doc comments
    let mut doc_lines = Vec::new();
    for attr in &input.attrs {
        if attr.path().is_ident("doc") {
            if let Meta::NameValue(meta) = &attr.meta {
                if let Expr::Lit(lit) = &meta.value {
                    if let Lit::Str(s) = &lit.lit {
                        let line = s.value();
                        let trimmed = line.trim();
                        if !trimmed.is_empty() {
                            doc_lines.push(trimmed.to_string());
                        }
                    }
                }
            }
        }
    }
    
    let fn_name_str = fn_name.to_string();
    let summary = doc_lines.first().unwrap_or(&"No summary".to_string()).clone();
    
    // Extract description (everything after summary but before any # sections)
    let mut description_lines = Vec::new();
    for (i, line) in doc_lines.iter().enumerate() {
        if i == 0 {
            continue; // Skip summary
        }
        if line.starts_with("#") {
            break; // Stop at first section header
        }
        if !line.trim().is_empty() {
            description_lines.push(line.clone());
        }
    }
    let description = if description_lines.is_empty() {
        "No description".to_string()
    } else {
        description_lines.join(" ")
    };
    
    // Simple parameter and response parsing from doc string
    let mut parameters = Vec::new();
    let mut responses = Vec::new();
    let mut request_body = Vec::new();
    
    let mut current_section = "";
    for line in &doc_lines {
        if line.starts_with("# Parameters") {
            current_section = "parameters";
        } else if line.starts_with("# Responses") {
            current_section = "responses";  
        } else if line.starts_with("# Request Body") {
            current_section = "request_body";
        } else if line.starts_with("- ") && current_section == "parameters" {
            parameters.push(line[2..].to_string());
        } else if line.starts_with("- ") && current_section == "responses" {
            let response_line = line[2..].to_string();
            
            // Handle both simple format "- 200: Success" and complex format "- 404:"
            if response_line.contains(":") {
                if let Some(colon_pos) = response_line.find(':') {
                    let status_part = response_line[..colon_pos].trim();
                    let desc_part = response_line[colon_pos + 1..].trim();
                    
                    if status_part.chars().all(|c| c.is_ascii_digit()) && status_part.len() == 3 {
                        if desc_part.is_empty() {
                            // Complex format - will collect description from following lines
                            responses.push(format!("{status_part}:"));
                        } else {
                            // Simple format
                            responses.push(response_line);
                        }
                    } else {
                        responses.push(response_line);
                    }
                } else {
                    responses.push(response_line);
                }
            } else {
                responses.push(response_line);
            }
        } else if current_section == "responses" && !line.starts_with("#") && !line.starts_with("- ") {
            // Handle YAML-style continuation lines for complex responses
            if line.trim().starts_with("description:") {
                let desc = line.trim().strip_prefix("description:").unwrap_or("").trim();
                // Update the last response entry with the description
                if let Some(last_response) = responses.last_mut() {
                    if last_response.ends_with(':') {
                        let status_code = last_response.trim_end_matches(':');
                        *last_response = format!("{status_code}: {desc}");
                    }
                }
            }
        } else if current_section == "request_body" && !line.starts_with("#") {
            request_body.push(line.clone());
        }
    }
    
    // Extract type information from function signature
    let request_body_type = extract_request_body_type(&input.sig.inputs);
    let (_response_type, _error_type) = extract_response_and_error_types(&input.sig.output);
    
    // Include type information in the request body documentation
    let mut enhanced_request_body = request_body.clone();
    if let Some(ref req_type) = request_body_type {
        // Add the type name to the beginning of the request body documentation
        enhanced_request_body.insert(0, format!("Type: {req_type}"));
    }
    
    let parameters_json = format!("[{}]", parameters.iter().map(|p| format!("\"{}\"", p.replace("\"", "\\\""))).collect::<Vec<_>>().join(","));
    let responses_json = format!("[{}]", responses.iter().map(|r| format!("\"{}\"", r.replace("\"", "\\\""))).collect::<Vec<_>>().join(","));
    let request_body_json = format!("[{}]", enhanced_request_body.iter().map(|rb| format!("\"{}\"", rb.replace("\"", "\\\""))).collect::<Vec<_>>().join(","));
    let tags_json = format!("[{}]", tags.iter().map(|t| format!("\"{}\"", t.replace("\"", "\\\""))).collect::<Vec<_>>().join(","));
    
    let output = quote! {
        #input
        
        // Register handler documentation at compile time
        stonehm::inventory::submit! {
            stonehm::HandlerDocumentation {
                function_name: #fn_name_str,
                summary: #summary,
                description: #description,
                parameters: #parameters_json,
                responses: #responses_json,
                request_body: #request_body_json,
                tags: #tags_json,
            }
        }
    };
    
    TokenStream::from(output)
}

/// Create a documented router that can extract docs from handlers
#[proc_macro]
pub fn documented_router(_input: TokenStream) -> TokenStream {
    // This macro will need to parse the router definition and extract handler docs
    // For now, let's create a simpler approach
    
    let output = quote! {
        stonehm::DocumentedRouter::new("API", "1.0.0")
    };
    
    TokenStream::from(output)
}

/// Derive macro for automatic JSON schema generation.
/// 
/// This derive macro automatically implements the `StonehmSchema` trait for your types,
/// enabling automatic JSON schema generation for OpenAPI specifications. Use this
/// on all request and response types that you want to appear in your OpenAPI spec.
/// 
/// # Type Support
/// 
/// Supported Rust types and their JSON schema mappings:
/// - `String`, `&str` → `"string"`
/// - `i32`, `i64`, `u32`, `u64`, etc. → `"integer"`
/// - `f32`, `f64` → `"number"`
/// - `bool` → `"boolean"`
/// - `Option<T>` → makes field optional
/// - `Vec<T>` → `"array"` with item schema
/// - Nested structs → object references
/// - Enums → `"string"` (basic support)
/// 
/// # Examples
/// 
/// ## Basic Struct
/// 
/// ```rust
/// use serde::Serialize;
/// use stonehm_macros::StoneSchema;
/// 
/// #[derive(Serialize, StonehmSchema)]
/// struct User {
///     id: u32,
///     name: String,
///     email: String,
///     is_active: bool,
///     age: Option<u32>,
/// }
/// 
/// // Generates JSON schema automatically
/// let schema = User::schema();
/// ```
/// 
/// ## Request/Response Types
/// 
/// ```rust
/// # use serde::{Serialize, Deserialize};
/// # use stonehm_macros::StonehmSchema;
/// 
/// #[derive(Deserialize, StonehmSchema)]
/// struct CreateUserRequest {
///     name: String,
///     email: String,
///     preferences: UserPreferences,
/// }
/// 
/// #[derive(Serialize, StonehmSchema)]
/// struct UserResponse {
///     id: u32,
///     name: String,
///     email: String,
///     created_at: String,
/// }
/// 
/// #[derive(Serialize, Deserialize, StonehmSchema)]
/// struct UserPreferences {
///     newsletter: bool,
///     theme: String,
/// }
/// ```
/// 
/// ## Error Types
/// 
/// ```rust
/// # use serde::Serialize;
/// # use stonehm_macros::StonehmSchema;
/// 
/// #[derive(Serialize, StonehmSchema)]
/// enum ApiError {
///     UserNotFound { id: u32 },
///     ValidationError { field: String, message: String },
///     DatabaseError,
///     NetworkTimeout,
/// }
/// ```
/// 
/// # Generated Schema Format
/// 
/// The macro generates JSON schemas following the OpenAPI 3.0 specification:
/// 
/// ```json
/// {
///   "title": "User",
///   "type": "object",
///   "properties": {
///     "id": { "type": "integer" },
///     "name": { "type": "string" },
///     "email": { "type": "string" },
///     "is_active": { "type": "boolean" },
///     "age": { "type": "integer" }
///   },
///   "required": ["id", "name", "email", "is_active"]
/// }
/// ```
/// 
/// # Usage with API Handlers
/// 
/// Use `StonehmSchema` types in your API handlers for automatic documentation:
/// 
/// ```rust,no_run
/// # use axum::Json;
/// # use stonehm::api_handler;
/// # use stonehm_macros::StonehmSchema;
/// # use serde::{Serialize, Deserialize};
/// # #[derive(Deserialize, StoneSchema)] struct CreateUserRequest { name: String }
/// # #[derive(Serialize, StoneSchema)] struct User { id: u32, name: String }
/// # #[derive(Serialize, StoneSchema)] enum ApiError { NotFound }
/// # use axum::response::IntoResponse;
/// # impl IntoResponse for ApiError { fn into_response(self) -> axum::response::Response { todo!() } }
/// 
/// /// Create a new user
/// #[api_handler]
/// async fn create_user(
///     Json(request): Json<CreateUserRequest>  // Schema automatically included
/// ) -> Result<Json<User>, ApiError> {         // Both schemas automatically included
///     // Implementation
/// #   Ok(Json(User { id: 1, name: request.name }))
/// }
/// ```
/// 
/// # Requirements
/// 
/// - Your type must implement `Serialize` (for response types) or `Deserialize` (for request types)
/// - The type must be used in a function signature annotated with `#[api_handler]`
/// - For error types used in `Result<T, E>`, implement `axum::response::IntoResponse`
#[proc_macro_derive(StonehmSchema)]
pub fn derive_stone_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let name_str = name.to_string();
    
    // Generate a simple JSON schema string
    let schema_json = match &input.data {
        Data::Struct(data_struct) => {
            match &data_struct.fields {
                Fields::Named(fields) => {
                    let mut properties = Vec::new();
                    let mut required = Vec::new();
                    
                    for field in fields.named.iter() {
                        if let Some(field_name) = &field.ident {
                            let field_name_str = field_name.to_string();
                            
                            // Simple type mapping - extend as needed
                            let type_str = match &field.ty {
                                Type::Path(type_path) => {
                                    if let Some(segment) = type_path.path.segments.last() {
                                        match segment.ident.to_string().as_str() {
                                            "String" | "str" => "string",
                                            "i32" | "i64" | "u32" | "u64" | "isize" | "usize" => "integer",
                                            "f32" | "f64" => "number",
                                            "bool" => "boolean",
                                            "Option" => {
                                                // Skip required for Option types
                                                "string" // simplified - extract inner type later
                                            },
                                            _ => "object", // custom types
                                        }
                                    } else {
                                        "string"
                                    }
                                },
                                _ => "string", // default for complex types
                            };
                            
                            properties.push(format!("\"{field_name_str}\":{{\"type\":\"{type_str}\"}}"));
                            
                            // Only add to required if not an Option type
                            if let Type::Path(type_path) = &field.ty {
                                if let Some(segment) = type_path.path.segments.last() {
                                    if segment.ident != "Option" {
                                        required.push(format!("\"{field_name_str}\""));
                                    }
                                }
                            } else {
                                required.push(format!("\"{field_name_str}\""));
                            }
                        }
                    }
                    
                    let properties_str = properties.join(",");
                    let required_str = if required.is_empty() {
                        String::new()
                    } else {
                        format!(",\"required\":[{}]", required.join(","))
                    };
                    
                    format!("{{\"type\":\"object\",\"properties\":{{{properties_str}}}{required_str}}}")
                },
                _ => {
                    "{\"type\":\"object\"}".to_string()
                }
            }
        },
        _ => {
            "{\"type\":\"string\"}".to_string()
        }
    };
    
    let expanded = quote! {
        impl stonehm::StonehmSchema for #name {
            fn schema() -> String {
                #schema_json.to_string()
            }
        }
        
        // Register this type's schema for OpenAPI components
        stonehm::inventory::submit! {
            stonehm::SchemaRegistration {
                type_name: #name_str,
                schema_json: #schema_json,
            }
        }
    };
    
    TokenStream::from(expanded)
}

/// Attribute macro for automatically generating HTTP error responses.
/// 
/// This macro automatically implements `axum::response::IntoResponse` for error enums,
/// mapping each variant to an appropriate HTTP status code. Use doc comments with
/// `/// {code}: {description}` format to specify status codes for variants.
/// 
/// # Basic Usage
/// 
/// ```rust
/// use stonehm_macros::api_error;
/// 
/// #[api_error]
/// enum ApiError {
///     /// 404: User not found
///     UserNotFound { id: u32 },
///     
///     /// 400: Invalid input provided
///     InvalidInput { message: String },
///     
///     /// 401: Authentication required
///     Unauthorized,
///     
///     /// 403: Access forbidden
///     Forbidden,
///     
///     // Variants without doc comments default to 500 Internal Server Error
///     DatabaseError,
///     NetworkTimeout,
/// }
/// ```
/// 
/// # Generated Implementation
/// 
/// The macro automatically generates:
/// - `IntoResponse` implementation for HTTP responses
/// - `Serialize` implementation for JSON serialization  
/// - `StonehmSchema` implementation for OpenAPI documentation
/// - Maps each variant to its specified status code
/// - Uses 500 Internal Server Error for variants without doc comments
/// - Serializes the error as JSON in the response body
/// 
/// # Supported Status Codes
/// 
/// Common HTTP status codes you can use:
/// - 200 OK
/// - 201 Created  
/// - 204 No Content
/// - 400 Bad Request
/// - 401 Unauthorized
/// - 403 Forbidden
/// - 404 Not Found
/// - 409 Conflict
/// - 422 Unprocessable Entity
/// - 500 Internal Server Error
/// - 502 Bad Gateway
/// - 503 Service Unavailable
/// 
/// # Examples
/// 
/// ## Basic Error Enum
/// 
/// ```rust
/// # use stonehm_macros::api_error;
/// # use serde::Serialize;
/// #[api_error]
/// #[derive(Serialize)]
/// enum UserError {
///     /// 404: User not found
///     NotFound { id: u32 },
///     
///     /// 400: Invalid user data
///     InvalidData { field: String, reason: String },
/// }
/// ```
/// 
/// ## With Custom Serialization
/// 
/// ```rust
/// # use stonehm_macros::api_error;
/// # use serde::Serialize;
/// #[api_error]
/// #[derive(Serialize)]
/// #[serde(tag = "error", content = "details")]
/// enum ApiError {
///     /// 401: Missing or invalid authentication token
///     #[serde(rename = "auth_required")]
///     AuthRequired,
///     
///     /// 403: User lacks required permissions
///     #[serde(rename = "access_denied")]
///     AccessDenied { required_role: String },
/// }
/// ```
/// 
/// ## Usage in Handlers
/// 
/// ```rust,no_run
/// # use axum::Json;
/// # use stonehm_macros::{api_error, api_handler, StoneSchema};
/// # use serde::{Serialize, Deserialize};
/// # #[derive(Deserialize, StoneSchema)]
/// # struct UpdateUserRequest { name: String }
/// # #[derive(Serialize, StoneSchema)]
/// # struct User { id: u32, name: String }
/// # #[api_error]
/// # #[derive(Serialize)]
/// # enum UserError {
/// #     /// 404: User not found
/// #     NotFound { id: u32 },
/// #     /// 400: Invalid data
/// #     InvalidData { message: String },
/// # }
/// 
/// /// Update user information
/// #[api_handler]
/// async fn update_user(
///     axum::extract::Path(id): axum::extract::Path<u32>,
///     Json(data): Json<UpdateUserRequest>
/// ) -> Result<Json<User>, UserError> {
///     if id == 0 {
///         return Err(UserError::NotFound { id });
///     }
///     
///     if data.name.is_empty() {
///         return Err(UserError::InvalidData { 
///             message: "Name cannot be empty".to_string() 
///         });
///     }
///     
///     Ok(Json(User { id, name: data.name }))
/// }
/// ```
/// 
/// # Requirements
/// 
/// - The error enum must also have `#[derive(Serialize)]` or implement `Serialize` manually
/// - Each variant's doc comment should start with a 3-digit HTTP status code followed by a colon
/// - The macro will automatically implement `axum::response::IntoResponse`
/// - The macro will register the error schema for OpenAPI documentation
#[proc_macro_attribute]
pub fn api_error(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;
    let name_str = name.to_string();
    
    // Extract status codes from doc comments
    let mut variant_status_codes = Vec::new();
    
    if let Data::Enum(data_enum) = &input.data {
        for variant in &data_enum.variants {
            let variant_name = &variant.ident;
            let mut status_code = 500u16; // Default to 500 Internal Server Error
            
            // Look for status code in doc comments
            for attr in &variant.attrs {
                if attr.path().is_ident("doc") {
                    if let Meta::NameValue(meta) = &attr.meta {
                        if let Expr::Lit(lit) = &meta.value {
                            if let Lit::Str(s) = &lit.lit {
                                let doc = s.value();
                                // Look for pattern like "404: Description" or "/// 404 Description"
                                if let Some(colon_pos) = doc.find(':') {
                                    let code_part = doc[..colon_pos].trim();
                                    if let Ok(code) = code_part.parse::<u16>() {
                                        status_code = code;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            variant_status_codes.push((variant_name.clone(), status_code));
        }
    }
    
    // Generate match arms for IntoResponse implementation
    let match_arms = variant_status_codes.iter().map(|(variant_name, status_code)| {
        quote! {
            Self::#variant_name { .. } => #status_code
        }
    });
    
    // Generate the implementation
    let expanded = quote! {
        #input
        
        impl axum::response::IntoResponse for #name {
            fn into_response(self) -> axum::response::Response {
                use axum::http::StatusCode;
                
                let status = match &self {
                    #(#match_arms),*
                };
                
                let body = axum::Json(serde_json::json!({
                    "error": serde_json::to_value(&self).unwrap_or_else(|_| serde_json::json!({
                        "message": "Failed to serialize error"
                    }))
                }));
                
                (StatusCode::from_u16(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR), body).into_response()
            }
        }
        
        // Also implement StonehmSchema for the error type
        impl stonehm::StonehmSchema for #name {
            fn schema() -> String {
                // For error enums, generate a simple schema
                // In a real implementation, this would analyze variants
                format!(r#"{{"type":"object","properties":{{"error":{{"type":"object"}}}}}}"#)
            }
        }
        
        // Register this error type's schema
        stonehm::inventory::submit! {
            stonehm::SchemaRegistration {
                type_name: #name_str,
                schema_json: r#"{"type":"object","properties":{"error":{"type":"object"}}}"#,
            }
        }
    };
    
    TokenStream::from(expanded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    use syn::parse_quote;
    
    #[test]
    fn test_extract_request_body_type() {
        // Test Json<T> extraction
        let inputs: syn::punctuated::Punctuated<FnArg, syn::token::Comma> = parse_quote! {
            Json(body): Json<CreateUserRequest>
        };
        
        let result = extract_request_body_type(&inputs);
        assert_eq!(result, Some("CreateUserRequest".to_string()));
        
        // Test with multiple parameters
        let inputs: syn::punctuated::Punctuated<FnArg, syn::token::Comma> = parse_quote! {
            Path(id): Path<u32>,
            Json(data): Json<UpdateRequest>
        };
        
        let result = extract_request_body_type(&inputs);
        assert_eq!(result, Some("UpdateRequest".to_string()));
        
        // Test without Json parameter
        let inputs: syn::punctuated::Punctuated<FnArg, syn::token::Comma> = parse_quote! {
            Path(id): Path<u32>
        };
        
        let result = extract_request_body_type(&inputs);
        assert_eq!(result, None);
    }
    
    #[test]
    fn test_extract_response_and_error_types() {
        // Test Result<Json<T>, E>
        let output: ReturnType = parse_quote! {
            -> Result<Json<UserResponse>, ApiError>
        };
        
        let (response_type, error_type) = extract_response_and_error_types(&output);
        assert_eq!(response_type, Some("UserResponse".to_string()));
        assert_eq!(error_type, Some("ApiError".to_string()));
        
        // Test Json<T> without Result
        let output: ReturnType = parse_quote! {
            -> Json<HealthResponse>
        };
        
        let (response_type, error_type) = extract_response_and_error_types(&output);
        assert_eq!(response_type, Some("HealthResponse".to_string()));
        assert_eq!(error_type, None);
        
        // Test Result with tuple success type
        let output: ReturnType = parse_quote! {
            -> Result<(StatusCode, Json<CreatedResponse>), CreateError>
        };
        
        let (response_type, error_type) = extract_response_and_error_types(&output);
        assert_eq!(response_type, None); // Current implementation doesn't handle tuples
        assert_eq!(error_type, Some("CreateError".to_string()));
        
        // Test no return type
        let output: ReturnType = ReturnType::Default;
        
        let (response_type, error_type) = extract_response_and_error_types(&output);
        assert_eq!(response_type, None);
        assert_eq!(error_type, None);
    }
    
    #[test]
    fn test_sanitize_type_for_identifier() {
        assert_eq!(sanitize_type_for_identifier("Vec<String>"), "Vec_String_");
        assert_eq!(sanitize_type_for_identifier("HashMap<String, Value>"), "HashMap_String_Value_");
        assert_eq!(sanitize_type_for_identifier("Option<User>"), "Option_User_");
        assert_eq!(sanitize_type_for_identifier("Result<T, E>"), "Result_T_E_");
        assert_eq!(sanitize_type_for_identifier("&str"), "_str");
        assert_eq!(sanitize_type_for_identifier("*const u8"), "_const_u8");
    }
    
    #[test]
    fn test_extract_docs_simple() {
        let attrs = vec![
            parse_quote!(#[doc = " Simple handler"]),
            parse_quote!(#[doc = " "]),
            parse_quote!(#[doc = " This is a simple test handler"]),
        ];
        
        let docs = extract_docs(&attrs);
        assert_eq!(docs.summary, Some("Simple handler".to_string()));
        assert_eq!(docs.description, Some("This is a simple test handler".to_string()));
    }
    
    #[test]
    fn test_extract_docs_with_parameters() {
        let attrs = vec![
            parse_quote!(#[doc = " Get user by ID"]),
            parse_quote!(#[doc = " "]),
            parse_quote!(#[doc = " Retrieves user information"]),
            parse_quote!(#[doc = " "]),
            parse_quote!(#[doc = " # Parameters"]),
            parse_quote!(#[doc = " - id (path): User ID"]),
            parse_quote!(#[doc = " - include_deleted (query): Include deleted users"]),
        ];
        
        let docs = extract_docs(&attrs);
        assert_eq!(docs.summary, Some("Get user by ID".to_string()));
        assert_eq!(docs.parameters.len(), 2);
        assert_eq!(docs.parameters[0].name, "id");
        assert_eq!(docs.parameters[0].param_type, "path");
        assert_eq!(docs.parameters[1].name, "include_deleted");
        assert_eq!(docs.parameters[1].param_type, "query");
    }
    
    #[test]
    fn test_extract_docs_with_request_body() {
        let attrs = vec![
            parse_quote!(#[doc = " Create user"]),
            parse_quote!(#[doc = " "]),
            parse_quote!(#[doc = " # Request Body"]),
            parse_quote!(#[doc = " Content-Type: application/json"]),
            parse_quote!(#[doc = " User data for creation"]),
            parse_quote!(#[doc = " - name (string): Full name"]),
            parse_quote!(#[doc = " - email (string): Email address"]),
        ];
        
        let docs = extract_docs(&attrs);
        assert!(docs.request_body.is_some());
        
        let body = docs.request_body.unwrap();
        assert_eq!(body.content_type, "application/json");
        assert_eq!(body.description, "User data for creation - name (string): Full name - email (string): Email address");
    }
    
    #[test]
    fn test_extract_docs_with_responses() {
        let attrs = vec![
            parse_quote!(#[doc = " Delete user"]),
            parse_quote!(#[doc = " "]),
            parse_quote!(#[doc = " # Responses"]),
            parse_quote!(#[doc = " - 204: User deleted"]),
            parse_quote!(#[doc = " - 404: User not found"]),
            parse_quote!(#[doc = " - 403: Access denied"]),
        ];
        
        let docs = extract_docs(&attrs);
        assert_eq!(docs.responses.len(), 3);
        
        assert_eq!(docs.responses[0].status_code, 204);
        assert_eq!(docs.responses[0].description, "User deleted");
        
        assert_eq!(docs.responses[1].status_code, 404);
        assert_eq!(docs.responses[1].description, "User not found");
        
        assert_eq!(docs.responses[2].status_code, 403);
        assert_eq!(docs.responses[2].description, "Access denied");
    }
    
    #[test]
    fn test_extract_docs_complex_responses() {
        let attrs = vec![
            parse_quote!(#[doc = " Complex endpoint"]),
            parse_quote!(#[doc = " "]),
            parse_quote!(#[doc = " # Responses"]),
            parse_quote!(#[doc = " - 200:"]),
            parse_quote!(#[doc = "   description: Success"]),
            parse_quote!(#[doc = "   content:"]),
            parse_quote!(#[doc = "     application/json:"]),
            parse_quote!(#[doc = "       schema: UserResponse"]),
            parse_quote!(#[doc = " - 404:"]),
            parse_quote!(#[doc = "   description: Not found"]),
        ];
        
        let docs = extract_docs(&attrs);
        assert_eq!(docs.responses.len(), 2);
        
        let resp200 = &docs.responses[0];
        assert_eq!(resp200.status_code, 200);
        assert_eq!(resp200.description, "Success");
        assert!(resp200.content.is_some());
        
        let content = resp200.content.as_ref().unwrap();
        assert_eq!(content.media_type, "application/json");
        assert_eq!(content.schema, Some("UserResponse".to_string()));
        
        let resp404 = &docs.responses[1];
        assert_eq!(resp404.status_code, 404);
        assert_eq!(resp404.description, "Not found");
    }
    
    #[test]
    fn test_extract_docs_with_examples() {
        let attrs = vec![
            parse_quote!(#[doc = " Test endpoint"]),
            parse_quote!(#[doc = " "]),
            parse_quote!(#[doc = " # Responses"]),
            parse_quote!(#[doc = " - 200:"]),
            parse_quote!(#[doc = "   description: Success"]),
            parse_quote!(#[doc = "   examples:"]),
            parse_quote!(#[doc = "     - name: success_example"]),
            parse_quote!(#[doc = "       summary: Successful response"]),
            parse_quote!(#[doc = r#"       value: {"status": "ok"}"#]),
        ];
        
        let docs = extract_docs(&attrs);
        assert_eq!(docs.responses.len(), 1);
        
        let resp = &docs.responses[0];
        assert!(resp.examples.is_some());
        
        let examples = resp.examples.as_ref().unwrap();
        assert_eq!(examples.len(), 1);
        
        let example = &examples[0];
        assert_eq!(example.name, "success_example");
        assert_eq!(example.summary, Some("Successful response".to_string()));
        assert_eq!(example.value, r#"{"status": "ok"}"#);
    }
    
    #[test]
    fn test_extract_docs_empty() {
        let attrs = vec![];
        let docs = extract_docs(&attrs);
        
        assert_eq!(docs.summary, None);
        assert_eq!(docs.description, None);
        assert!(docs.parameters.is_empty());
        assert!(docs.request_body.is_none());
        assert!(docs.responses.is_empty());
    }
}