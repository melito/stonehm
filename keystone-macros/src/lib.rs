use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, Attribute, Lit, Meta, Expr};

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
                if line.starts_with("Content-Type:") {
                    let content_type = line["Content-Type:".len()..].trim().to_string();
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

/// Macro to annotate API handlers and extract documentation
#[proc_macro_attribute]
pub fn api_handler(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;
    let parsed_docs = extract_docs(&input.attrs);
    
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
            keystone::ParameterDoc {
                name: #name.to_string(),
                description: #description.to_string(),
                param_type: #param_type.to_string(),
            }
        }
    }).collect();
    
    // Generate request body documentation
    let request_body_tokens = match parsed_docs.request_body {
        Some(ref body) => {
            let description = &body.description;
            let content_type = &body.content_type;
            quote! {
                Some(keystone::RequestBodyDoc {
                    description: #description.to_string(),
                    content_type: #content_type.to_string(),
                })
            }
        }
        None => quote! { None },
    };

    // Generate response documentation
    let response_tokens: Vec<_> = parsed_docs.responses.iter().map(|resp| {
        let status_code = resp.status_code;
        let description = &resp.description;
        quote! {
            keystone::ResponseDoc {
                status_code: #status_code,
                description: #description.to_string(),
            }
        }
    }).collect();
    
    // Generate metadata constant using a predictable naming pattern
    let metadata_const_name = syn::Ident::new(
        &format!("__DOCS_{}", fn_name.to_string().to_uppercase()),
        fn_name.span()
    );
    
    let fn_name_str = fn_name.to_string();
    
    let output = quote! {
        #input
        
        #[allow(non_upper_case_globals)]
        pub fn #metadata_const_name() -> keystone::HandlerDocumentation {
            keystone::HandlerDocumentation {
                summary: #summary_tokens,
                description: #description_tokens,
                parameters: vec![#(#parameter_tokens),*],
                request_body: #request_body_tokens,
                responses: vec![#(#response_tokens),*],
            }
        }
        
        // Register this handler in the global registry
        keystone::inventory::submit! {
            keystone::HandlerDocEntry {
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
        keystone::DocumentedRouter::new("API", "1.0.0")
    };
    
    TokenStream::from(output)
}