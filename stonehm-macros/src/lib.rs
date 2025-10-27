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
                // Parse response lines like "- 200: Success" or "* 404: User not found"
                if line.starts_with("- ") || line.starts_with("* ") {
                    let response_text = &line[2..].trim();
                    if let Some(colon_pos) = response_text.find(':') {
                        let status_part = response_text[..colon_pos].trim();
                        let desc_part = response_text[colon_pos + 1..].trim();
                        
                        if let Ok(status_code) = status_part.parse::<u16>() {
                            responses.push(ResponseDoc {
                                status_code,
                                description: desc_part.to_string(),
                            });
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

/// Extract response type from function return type
fn extract_response_type(output: &ReturnType) -> Option<String> {
    if let ReturnType::Type(_, return_type) = output {
        if let Type::Path(type_path) = &**return_type {
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
    None
}

/// Macro to annotate API handlers and extract documentation
#[proc_macro_attribute]
pub fn api_handler(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;
    let parsed_docs = extract_docs(&input.attrs);
    
    // Extract type information from the function signature
    let request_body_type = extract_request_body_type(&input.sig.inputs);
    let response_type = extract_response_type(&input.sig.output);
    
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

    // Generate response documentation
    let response_tokens: Vec<_> = parsed_docs.responses.iter().map(|resp| {
        let status_code = resp.status_code;
        let description = &resp.description;
        quote! {
            stonehm::ResponseDoc {
                status_code: #status_code,
                description: #description.to_string(),
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
            &format!("__schema_{sanitized_type}"),
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
            &format!("__schema_{sanitized_type}"),
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

/// Derive macro for Stonehm's schema generation system.
/// 
/// This derive macro automatically implements the `StoneSchema` trait for structs,
/// enabling automatic JSON schema generation for OpenAPI specifications.
/// 
/// # Examples
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
///     active: bool,
/// }
/// 
/// // The StoneSchema trait is now implemented
/// let schema = User::schema();
/// ```
/// 
/// # Generated Schema
/// 
/// The macro generates a JSON schema with:
/// - `type: "object"` for structs
/// - `properties` containing each field with appropriate types
/// - `required` array listing all fields
/// - `title` set to the struct name
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