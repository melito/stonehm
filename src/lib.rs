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
use std::sync::{Arc, Mutex};
pub use keystone_macros::api_handler;
pub use inventory;

/// Registry entry for handler documentation
pub struct HandlerDocEntry {
    pub name: &'static str,
    pub get_docs: fn() -> HandlerDocumentation,
}

inventory::collect!(HandlerDocEntry);

#[derive(Debug, Clone)]
pub struct ResponseDoc {
    pub status_code: u16,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct ParameterDoc {
    pub name: String,
    pub description: String,
    pub param_type: String, // path, query, header
}

#[derive(Debug, Clone)]
pub struct RequestBodyDoc {
    pub description: String,
    pub content_type: String,
}

#[derive(Debug, Clone)]
pub struct HandlerDocumentation {
    pub summary: Option<&'static str>,
    pub description: Option<&'static str>,
    pub parameters: Vec<ParameterDoc>,
    pub request_body: Option<RequestBodyDoc>,
    pub responses: Vec<ResponseDoc>,
}

// Keep the old struct for backwards compatibility
#[derive(Debug, Clone)]
pub struct HandlerMetadata {
    pub summary: Option<&'static str>,
    pub description: Option<&'static str>,
}

#[derive(Debug, Clone)]
pub struct RouteInfo {
    pub path: String,
    pub method: Method,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub parameters: Vec<ParameterDoc>,
    pub request_body: Option<RequestBodyDoc>,
    pub responses: Vec<ResponseDoc>,
}

/// Trait for handlers that have documentation metadata
pub trait DocumentedHandler {
    fn metadata() -> HandlerMetadata {
        HandlerMetadata {
            summary: None,
            description: None,
        }
    }
}

/// Registry for handler documentation
pub trait HandlerRegistry {
    fn get_handler_docs(handler_name: &str) -> Option<HandlerMetadata>;
}

/// A router that automatically captures handler documentation
pub struct DocumentedRouter {
    inner: Router,
    routes: Arc<Mutex<Vec<RouteInfo>>>,
    spec: Arc<Mutex<OpenAPI>>,
}

impl DocumentedRouter {
    pub fn new(title: impl Into<String>, version: impl Into<String>) -> Self {
        let mut spec = OpenAPI::default();
        spec.openapi = "3.0.3".to_string();
        spec.info = Info {
            title: title.into(),
            version: version.into(),
            ..Default::default()
        };

        Self {
            inner: Router::new(),
            routes: Arc::new(Mutex::new(Vec::new())),
            spec: Arc::new(Mutex::new(spec)),
        }
    }

    /// Route with automatic documentation lookup
    pub fn route<F>(self, path: &str, method_router: MethodRouter) -> Self {
        // For now, register without docs - we'll enhance this
        self.register_route(path, Method::GET, None, None, vec![], None, vec![]); // Default to GET, would need smarter detection
        
        Self {
            inner: self.inner.route(path, method_router),
            routes: self.routes,
            spec: self.spec,
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
        
        self.register_route(
            path, 
            Method::GET, 
            docs.summary, 
            docs.description, 
            docs.parameters,
            docs.request_body,
            docs.responses
        );
        
        Self {
            inner: self.inner.route(path, get(handler)),
            routes: self.routes,
            spec: self.spec,
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
        
        self.register_route(
            path, 
            Method::POST, 
            docs.summary, 
            docs.description, 
            docs.parameters,
            docs.request_body,
            docs.responses
        );
        
        Self {
            inner: self.inner.route(path, post(handler)),
            routes: self.routes,
            spec: self.spec,
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
        
        self.register_route(
            path, 
            Method::PUT, 
            docs.summary, 
            docs.description, 
            docs.parameters,
            docs.request_body,
            docs.responses
        );
        
        Self {
            inner: self.inner.route(path, put(handler)),
            routes: self.routes,
            spec: self.spec,
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
        
        self.register_route(
            path, 
            Method::DELETE, 
            docs.summary, 
            docs.description, 
            docs.parameters,
            docs.request_body,
            docs.responses
        );
        
        Self {
            inner: self.inner.route(path, delete(handler)),
            routes: self.routes,
            spec: self.spec,
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
        
        self.register_route(
            path, 
            Method::PATCH, 
            docs.summary, 
            docs.description, 
            docs.parameters,
            docs.request_body,
            docs.responses
        );
        
        Self {
            inner: self.inner.route(path, patch(handler)),
            routes: self.routes,
            spec: self.spec,
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
            responses: vec![],
        }
    }

    /// Register a route in our tracking system
    fn register_route(
        &self, 
        path: &str, 
        method: Method, 
        summary: Option<&str>, 
        description: Option<&str>,
        parameters: Vec<ParameterDoc>,
        request_body: Option<RequestBodyDoc>,
        responses: Vec<ResponseDoc>
    ) {
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
        
        // Update the OpenAPI spec
        let mut spec = self.spec.lock().unwrap();
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
        }
    }

    /// Nest a router at the given path
    pub fn nest(self, path: &str, router: Router) -> Self {
        Self {
            inner: self.inner.nest(path, router),
            routes: self.routes,
            spec: self.spec,
        }
    }

    /// Add routes to serve the OpenAPI spec with default prefix
    pub fn with_openapi_routes(self) -> Self {
        self.with_openapi_routes_prefix("/openapi")
    }

    /// Add routes to serve the OpenAPI spec with custom prefix
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

        let json_path = format!("{}.json", normalized_prefix);
        let yaml_path = format!("{}.yaml", normalized_prefix);

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
        }
    }

    /// Convert into a regular axum Router
    pub fn into_router(self) -> Router {
        self.inner
    }

    /// Get the current OpenAPI spec
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
        content.insert(
            body_doc.content_type.clone(),
            MediaType {
                schema: Some(ReferenceOr::Item(Schema {
                    schema_data: Default::default(),
                    schema_kind: SchemaKind::Type(Type::Object(Default::default())),
                })),
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
        // Add default response if none specified
        let mut success_response = ApiResponse::default();
        success_response.description = "Successful response".to_string();
        responses.responses.insert(
            StatusCode::Code(200),
            ReferenceOr::Item(success_response)
        );
    } else {
        // Add documented responses
        for response_doc in response_docs {
            let mut response = ApiResponse::default();
            response.description = response_doc.description.clone();
            
            responses.responses.insert(
                StatusCode::Code(response_doc.status_code),
                ReferenceOr::Item(response)
            );
        }
    }
    
    operation.responses = responses;
    
    operation
}

/// Create an operation with optional documentation and responses (backwards compatibility)
fn create_operation_with_responses(
    path: &str, 
    method: &Method, 
    summary: Option<&str>, 
    description: Option<&str>,
    response_docs: &[ResponseDoc]
) -> Operation {
    create_operation_with_params_and_responses(path, method, summary, description, &[], &None, response_docs)
}

/// Macro for creating a DocumentedRouter
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
            },
            ResponseDoc {
                status_code: 404,
                description: "Not found".to_string(),
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
    async fn test_documented_handler() -> Json<TestResponse> {
        Json(TestResponse {
            message: "test documented".to_string(),
        })
    }
    
    // Manually create the documentation function for this test handler
    #[allow(non_upper_case_globals)]
    pub fn __DOCS_TEST_DOCUMENTED_HANDLER() -> HandlerDocumentation {
        HandlerDocumentation {
            summary: Some("Test handler with documentation"),
            description: Some("This is a test handler with documentation for testing purposes."),
            parameters: vec![],
            request_body: None,
            responses: vec![
                ResponseDoc {
                    status_code: 200,
                    description: "Test successful".to_string(),
                },
                ResponseDoc {
                    status_code: 400,
                    description: "Test failed".to_string(),
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