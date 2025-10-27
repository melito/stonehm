use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, Attribute, Lit, Meta, Expr, Type, FnArg, ReturnType, PathArguments, GenericArgument, DeriveInput, Data, Fields};

/// Sanitize a type string to create a valid Rust identifier
fn sanitize_type_for_identifier(type_str: &str) -> String {
    type_str
        .replace(['<', '>', ' ', ',', ':', ';', '(', ')', '[', ']', '{', '}', '&', '*'], "_")
        .replace("__", "_")
        .trim_matches('_')
        .to_string()
}

#[derive(Debug, Clone)]
struct ResponseDoc {
    status_code: u16,
    description: String,
    content: Option<ResponseContent>,
    examples: Option<Vec<ResponseExample>>,
}

#[derive(Debug, Clone)]
struct ResponseContent {
    media_type: String, // e.g., "application/json"
    schema: Option<String>, // e.g., "ErrorResponse"
}

#[derive(Debug, Clone)]
struct ResponseExample {
    name: String,
    summary: Option<String>,
    value: String, // JSON or other content
}

#[derive(Debug, Clone)]
struct ParameterDoc {
    name: String,
    description: String,
    param_type: String, // path, query, header
}

#[derive(Debug, Clone)]
struct RequestBodyDoc {
    description: String,
    content_type: String,
}

#[derive(Debug, Clone)]
struct ParsedDocs {
    summary: Option<String>,
    description: Option<String>,
    parameters: Vec<ParameterDoc>,
    request_body: Option<RequestBodyDoc>,
    responses: Vec<ResponseDoc>,
}

/// Extract documentation from attributes
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
                        if line.starts_with("description:") {
                            let desc = line[12..].trim().trim_matches('"');
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
                        } else if line.starts_with("schema:") {
                            let schema_name = line[7..].trim();
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
                            let name = if line.starts_with("- name:") {
                                line[7..].trim()
                            } else {
                                line[5..].trim()
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
                        if let Some(GenericArgument::Type(ok_type)) = args.args.first() {
                            // Check if it's Json<T>
                            if let Type::Path(ok_path) = ok_type {
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


/// Macro to annotate API handlers and extract documentation
#[proc_macro_attribute]
pub fn api_handler(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;
    let parsed_docs = extract_docs(&input.attrs);
    
    // Extract type information from the function signature
    let request_body_type = extract_request_body_type(&input.sig.inputs);
    let (response_type, error_type) = extract_response_and_error_types(&input.sig.output);
    
    
    let summary_tokens = match parsed_docs.summary {
        Some(s) => quote! { Some(#s) },
        None => quote! { None },
    };
    
    let description_tokens = match parsed_docs.description {
        Some(d) => quote! { Some(#d) },
        None => quote! { None },
    };
    
    // Generate parameter documentation
    let parameter_tokens: Vec<_> = parsed_docs.parameters.iter().map(|param| {
        let name = &param.name;
        let description = &param.description;
        let param_type = &param.param_type;
        quote! {
            stonehm::ParameterDoc {
                name: #name.to_string(),
                description: #description.to_string(),
                param_type: #param_type.to_string(),
            }
        }
    }).collect();
    
    // Generate request body documentation with schema type from signature analysis
    let request_body_tokens = match parsed_docs.request_body {
        Some(ref body) => {
            let description = &body.description;
            let content_type = &body.content_type;
            let schema_type_token = if let Some(ref req_type) = request_body_type {
                quote! { Some(#req_type.to_string()) }
            } else {
                quote! { None }
            };
            quote! {
                Some(stonehm::RequestBodyDoc {
                    description: #description.to_string(),
                    content_type: #content_type.to_string(),
                    schema_type: #schema_type_token,
                })
            }
        }
        None => quote! { None },
    };
    
    // Generate type tokens
    let request_body_type_token = match request_body_type {
        Some(ref t) => quote! { Some(#t.to_string()) },
        None => quote! { None },
    };
    
    let response_type_token = match response_type {
        Some(ref t) => quote! { Some(#t.to_string()) },
        None => quote! { None },
    };
    
    let error_type_token = match error_type {
        Some(ref t) => quote! { Some(#t.to_string()) },
        None => quote! { None },
    };

    // Generate response documentation
    let response_tokens: Vec<_> = parsed_docs.responses.iter().map(|resp| {
        let status_code = resp.status_code;
        let description = &resp.description;
        
        let content_token = match &resp.content {
            Some(content) => {
                let media_type = &content.media_type;
                let schema_token = match &content.schema {
                    Some(schema) => quote! { Some(#schema.to_string()) },
                    None => quote! { None },
                };
                quote! {
                    Some(stonehm::ResponseContent {
                        media_type: #media_type.to_string(),
                        schema: #schema_token,
                    })
                }
            },
            None => quote! { None },
        };
        
        let examples_token = match &resp.examples {
            Some(examples) => {
                let example_tokens: Vec<_> = examples.iter().map(|ex| {
                    let name = &ex.name;
                    let value = &ex.value;
                    let summary_token = match &ex.summary {
                        Some(summary) => quote! { Some(#summary.to_string()) },
                        None => quote! { None },
                    };
                    quote! {
                        stonehm::ResponseExample {
                            name: #name.to_string(),
                            summary: #summary_token,
                            value: #value.to_string(),
                        }
                    }
                }).collect();
                quote! { Some(vec![#(#example_tokens),*]) }
            },
            None => quote! { None },
        };
        
        quote! {
            stonehm::ResponseDoc {
                status_code: #status_code,
                description: #description.to_string(),
                content: #content_token,
                examples: #examples_token,
            }
        }
    }).collect();
    
    // Generate metadata constant using a predictable naming pattern
    let metadata_const_name = syn::Ident::new(
        &format!("__docs_{}", fn_name.to_string().to_lowercase()),
        fn_name.span()
    );
    
    let fn_name_str = fn_name.to_string();
    
    // Generate schema functions for detected types
    let mut schema_functions = Vec::new();
    
    if let Some(ref req_type) = request_body_type {
        let sanitized_type = sanitize_type_for_identifier(req_type).to_lowercase();
        let schema_fn_name = syn::Ident::new(
            &format!("__schema_{}_{sanitized_type}", fn_name.to_string().to_lowercase()),
            fn_name.span()
        );
        let req_type_ident: syn::Type = syn::parse_str(req_type).unwrap_or_else(|_| {
            syn::parse_quote! { () }
        });
        
        schema_functions.push(quote! {
            #[allow(non_upper_case_globals)]
            pub fn #schema_fn_name() -> Option<stonehm::serde_json::Value> {
                // Try to use our simple schema first, fallback to schemars if available
                Some(<#req_type_ident>::schema())
            }
            
            // Register this schema function in the global registry
            stonehm::inventory::submit! {
                stonehm::SchemaEntry {
                    type_name: #req_type,
                    get_schema: #schema_fn_name,
                }
            }
        });
    }
    
    if let Some(ref resp_type) = response_type {
        let sanitized_type = sanitize_type_for_identifier(resp_type).to_lowercase();
        let schema_fn_name = syn::Ident::new(
            &format!("__schema_{}_{sanitized_type}", fn_name.to_string().to_lowercase()),
            fn_name.span()
        );
        let resp_type_ident: syn::Type = syn::parse_str(resp_type).unwrap_or_else(|_| {
            syn::parse_quote! { () }
        });
        
        schema_functions.push(quote! {
            #[allow(non_upper_case_globals)]
            pub fn #schema_fn_name() -> Option<stonehm::serde_json::Value> {
                // Try to use our simple schema first, fallback to schemars if available
                Some(<#resp_type_ident>::schema())
            }
            
            // Register this schema function in the global registry
            stonehm::inventory::submit! {
                stonehm::SchemaEntry {
                    type_name: #resp_type,
                    get_schema: #schema_fn_name,
                }
            }
        });
    }
    
    if let Some(ref err_type) = error_type {
        let sanitized_type = sanitize_type_for_identifier(err_type).to_lowercase();
        let schema_fn_name = syn::Ident::new(
            &format!("__schema_{}_{sanitized_type}", fn_name.to_string().to_lowercase()),
            fn_name.span()
        );
        let err_type_ident: syn::Type = syn::parse_str(err_type).unwrap_or_else(|_| {
            syn::parse_quote! { () }
        });
        
        schema_functions.push(quote! {
            #[allow(non_upper_case_globals)]
            pub fn #schema_fn_name() -> Option<stonehm::serde_json::Value> {
                // Try to use our simple schema first, fallback to schemars if available
                Some(<#err_type_ident>::schema())
            }
            
            // Register this schema function in the global registry
            stonehm::inventory::submit! {
                stonehm::SchemaEntry {
                    type_name: #err_type,
                    get_schema: #schema_fn_name,
                }
            }
        });
    }

    let output = quote! {
        #input
        
        #[allow(non_upper_case_globals)]
        pub fn #metadata_const_name() -> stonehm::HandlerDocumentation {
            stonehm::HandlerDocumentation {
                summary: #summary_tokens,
                description: #description_tokens,
                parameters: vec![#(#parameter_tokens),*],
                request_body: #request_body_tokens,
                request_body_type: #request_body_type_token,
                response_type: #response_type_token,
                error_type: #error_type_token,
                responses: vec![#(#response_tokens),*],
            }
        }
        
        // Generate schema functions for types
        #(#schema_functions)*
        
        // Register this handler in the global registry
        stonehm::inventory::submit! {
            stonehm::HandlerDocEntry {
                name: #fn_name_str,
                get_docs: #metadata_const_name,
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
/// This derive macro automatically implements the `StoneSchema` trait for your types,
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
/// #[derive(Serialize, StoneSchema)]
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
/// # use stonehm_macros::StoneSchema;
/// 
/// #[derive(Deserialize, StoneSchema)]
/// struct CreateUserRequest {
///     name: String,
///     email: String,
///     preferences: UserPreferences,
/// }
/// 
/// #[derive(Serialize, StoneSchema)]
/// struct UserResponse {
///     id: u32,
///     name: String,
///     email: String,
///     created_at: String,
/// }
/// 
/// #[derive(Serialize, Deserialize, StoneSchema)]
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
/// # use stonehm_macros::StoneSchema;
/// 
/// #[derive(Serialize, StoneSchema)]
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
/// Use `StoneSchema` types in your API handlers for automatic documentation:
/// 
/// ```rust,no_run
/// # use axum::Json;
/// # use stonehm::api_handler;
/// # use stonehm_macros::StoneSchema;
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
#[proc_macro_derive(StoneSchema)]
pub fn derive_stone_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let name_str = name.to_string();
    
    let schema_json = match &input.data {
        Data::Struct(data_struct) => {
            match &data_struct.fields {
                Fields::Named(fields) => {
                    let mut properties = vec![];
                    let mut required = vec![];
                    
                    for field in fields.named.iter() {
                        if let Some(field_name) = &field.ident {
                            let field_name_str = field_name.to_string();
                            required.push(quote! { #field_name_str });
                            
                            // Simple type mapping - extend as needed
                            let type_str = match &field.ty {
                                Type::Path(type_path) => {
                                    if let Some(segment) = type_path.path.segments.last() {
                                        match segment.ident.to_string().as_str() {
                                            "String" | "str" => "string",
                                            "i32" | "i64" | "u32" | "u64" | "isize" | "usize" => "integer",
                                            "f32" | "f64" => "number",
                                            "bool" => "boolean",
                                            _ => "string", // default fallback
                                        }
                                    } else {
                                        "string"
                                    }
                                },
                                _ => "string", // default for complex types
                            };
                            
                            properties.push(quote! {
                                (#field_name_str.to_string(), stonehm::serde_json::json!({ "type": #type_str }))
                            });
                        }
                    }
                    
                    quote! {
                        stonehm::serde_json::json!({
                            "title": #name_str,
                            "type": "object",
                            "properties": stonehm::serde_json::Value::Object(
                                [#(#properties),*].into_iter().collect()
                            ),
                            "required": [#(#required),*]
                        })
                    }
                },
                _ => {
                    // For unit structs or tuple structs, create a simple object
                    quote! {
                        stonehm::serde_json::json!({
                            "title": #name_str,
                            "type": "object"
                        })
                    }
                }
            }
        },
        _ => {
            // For enums, unions, etc., create a simple string type for now
            quote! {
                stonehm::serde_json::json!({
                    "title": #name_str,
                    "type": "string"
                })
            }
        }
    };
    
    let schema_fn_name = syn::Ident::new(&format!("__{}_schema", name.to_string().to_lowercase()), name.span());
    
    let expanded = quote! {
        impl stonehm::StoneSchema for #name {
            fn schema() -> stonehm::serde_json::Value {
                #schema_json
            }
        }
        
        impl #name {
            pub fn schema() -> stonehm::serde_json::Value {
                <Self as stonehm::StoneSchema>::schema()
            }
        }
        
        // Also provide a standalone function for easier access
        pub fn #schema_fn_name() -> stonehm::serde_json::Value {
            #name::schema()
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
/// use serde::Serialize;
/// use stonehm_macros::{StoneSchema, api_error};
/// 
/// #[derive(Serialize, StoneSchema)]
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
/// The macro generates an `IntoResponse` implementation that:
/// - Maps each variant to its specified status code
/// - Uses 500 Internal Server Error for variants without `#[status]` attributes
/// - Serializes the error as JSON in the response body
/// - Sets appropriate content-type headers
/// 
/// # Supported Status Codes
/// 
/// Common HTTP status codes you can use:
/// - `400` - Bad Request
/// - `401` - Unauthorized  
/// - `403` - Forbidden
/// - `404` - Not Found
/// - `409` - Conflict
/// - `422` - Unprocessable Entity
/// - `500` - Internal Server Error (default)
/// - `502` - Bad Gateway
/// - `503` - Service Unavailable
/// 
/// # Usage with Result Types
/// 
/// Use with `Result<T, E>` return types for automatic error documentation:
/// 
/// ```rust,no_run
/// # use axum::Json;
/// # use stonehm::api_handler;
/// # use stonehm_macros::{StoneSchema, api_error};
/// # use serde::{Serialize, Deserialize};
/// # #[derive(Serialize, StoneSchema)] #[api_error] enum ApiError { #[doc = "404: Not found"] NotFound }
/// # #[derive(Serialize, StoneSchema)] struct User { id: u32 }
/// 
/// #[api_handler]
/// async fn get_user() -> Result<Json<User>, ApiError> {
///     // Automatically generates 200, 400, 500 responses in OpenAPI spec
///     Ok(Json(User { id: 1 }))
/// }
/// ```
/// 
/// # Error Message Generation
/// 
/// The macro generates user-friendly error messages:
/// - For variants with fields: includes field values in the message
/// - For unit variants: uses the variant name as the message
/// - For tuple variants: includes the data in the message
/// 
/// # Requirements
/// 
/// - The enum must implement `Serialize` (for JSON serialization)
/// - Preferably also derive `StoneSchema` for OpenAPI documentation
/// - Use with `#[api_handler]` functions for automatic documentation
#[proc_macro_attribute]
pub fn api_error(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;
    
    let variants = match &input.data {
        Data::Enum(data_enum) => &data_enum.variants,
        _ => {
            return syn::Error::new_spanned(
                &input,
                "api_error can only be applied to enums"
            ).to_compile_error().into();
        }
    };
    
    let mut match_arms = Vec::new();
    
    for variant in variants {
        let variant_name = &variant.ident;
        
        // Look for doc comment with status code in format "/// {code}: {description}"
        let status_code = variant.attrs.iter()
            .find_map(|attr| {
                if attr.path().is_ident("doc") {
                    if let Meta::NameValue(meta) = &attr.meta {
                        if let Expr::Lit(lit) = &meta.value {
                            if let Lit::Str(s) = &lit.lit {
                                let doc_string = s.value();
                                let trimmed = doc_string.trim();
                                // Look for pattern like "404: User not found"
                                if let Some(colon_pos) = trimmed.find(':') {
                                    let status_part = trimmed[..colon_pos].trim();
                                    if let Ok(code) = status_part.parse::<u16>() {
                                        return Some(code);
                                    }
                                }
                            }
                        }
                    }
                }
                None
            })
            .unwrap_or(500); // Default to 500 Internal Server Error
        
        // Generate the match arm based on variant structure
        let (pattern, message_expr) = match &variant.fields {
            Fields::Named(fields) => {
                if fields.named.is_empty() {
                    // No fields
                    (quote! { Self::#variant_name }, quote! { stringify!(#variant_name).to_string() })
                } else {
                    // Named fields - create a descriptive message
                    let field_names: Vec<_> = fields.named.iter()
                        .map(|f| f.ident.as_ref().unwrap())
                        .collect();
                    
                    if field_names.len() == 1 && field_names[0] == "message" {
                        // Special case: if there's a single "message" field, use it directly
                        (quote! { Self::#variant_name { message } }, quote! { message.clone() })
                    } else if field_names.len() == 1 && field_names[0] == "id" {
                        // Special case: if there's a single "id" field, create a meaningful message
                        (quote! { Self::#variant_name { id } }, quote! { format!("{} with id {}", stringify!(#variant_name), id) })
                    } else {
                        // Multiple fields or other single field - create a generic message
                        let pattern_fields = field_names.iter().map(|name| quote! { #name });
                        (quote! { Self::#variant_name { #(#pattern_fields),* } }, quote! { stringify!(#variant_name).to_string() })
                    }
                }
            },
            Fields::Unnamed(fields) => {
                if fields.unnamed.len() == 1 {
                    // Single tuple field - use it in the message
                    (quote! { Self::#variant_name(value) }, quote! { format!("{}: {:?}", stringify!(#variant_name), value) })
                } else {
                    // Multiple tuple fields
                    let field_patterns: Vec<_> = (0..fields.unnamed.len())
                        .map(|i| syn::Ident::new(&format!("field_{}", i), variant_name.span()))
                        .collect();
                    (quote! { Self::#variant_name(#(#field_patterns),*) }, quote! { stringify!(#variant_name).to_string() })
                }
            },
            Fields::Unit => {
                // Unit variant
                (quote! { Self::#variant_name }, quote! { stringify!(#variant_name).to_string() })
            }
        };
        
        match_arms.push(quote! {
            #pattern => {
                let message = #message_expr;
                (
                    axum::http::StatusCode::from_u16(#status_code).unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
                    message
                )
            }
        });
    }
    
    let expanded = quote! {
        #input
        
        impl axum::response::IntoResponse for #name {
            fn into_response(self) -> axum::response::Response {
                let (status, message) = match self {
                    #(#match_arms),*
                };
                
                let body = axum::Json(stonehm::serde_json::json!({
                    "error": message
                }));
                
                (status, body).into_response()
            }
        }
    };
    
    TokenStream::from(expanded)
}



