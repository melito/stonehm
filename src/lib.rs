//! Automatic OpenAPI 3.0 spec generation for Axum applications.
//!
//! Annotate handlers with `#[api_handler]`, derive `StonehmSchema` on
//! request/response types, and build routes through [`ApiRouter`] —
//! the crate walks the resulting metadata at runtime to emit a
//! complete OpenAPI document via [`openapi_value`](ApiRouter::openapi_value)
//! / [`openapi_json`](ApiRouter::openapi_json), or
//! [`aggregated_openapi_value`] when an app is split across modules.
//!
//! See the README for end-to-end examples.

// Lets the proc-macro-derived `impl stonehm::StonehmSchema for ...` blocks
// resolve cleanly inside this crate's own tests. Without this, the
// macro-emitted absolute path `::stonehm::StonehmSchema` refers to nothing
// when the derive runs from inside `stonehm` itself.
extern crate self as stonehm;

use axum::{
    routing::{get, post, put, delete, patch},
    Router,
};
use serde_json::{json, Map as JsonMap, Value};
use std::collections::HashMap;

// Simple OpenAPI types
#[derive(Debug, Clone)]
pub struct OpenAPI {
    pub info: Info,
    pub paths: HashMap<String, PathItem>,
    pub components: Option<Components>,
    pub tags: Vec<Tag>,
}

#[derive(Debug, Clone)]
pub struct Tag {
    pub name: String,
    pub description: Option<String>,
    pub external_docs: Option<ExternalDocs>,
}

#[derive(Debug, Clone)]
pub struct ExternalDocs {
    pub description: Option<String>,
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct Components {
    pub schemas: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct RouteInfo {
    pub path: String,
    pub method: String,
    pub function_name: String,
    pub summary: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct HandlerDocumentation {
    pub function_name: &'static str,
    pub summary: &'static str,
    pub description: &'static str,
    pub parameters: &'static str,
    pub responses: &'static str,
    pub request_body: &'static str,
    pub tags: &'static str,
}

#[derive(Debug, Clone)]
pub struct SchemaRegistration {
    pub type_name: &'static str,
    pub schema_json: &'static str,
}

inventory::collect!(HandlerDocumentation);
inventory::collect!(SchemaRegistration);

impl OpenAPI {
    pub fn new(title: &str, version: &str) -> Self {
        Self {
            info: Info { 
                title: title.to_string(), 
                version: version.to_string(),
                description: None,
                terms_of_service: None,
                contact: None,
                license: None,
            },
            paths: HashMap::new(),
            components: None,
            tags: Vec::new(),
        }
    }
    
}

#[derive(Debug, Clone)]
pub struct Info {
    pub title: String,
    pub version: String,
    pub description: Option<String>,
    pub terms_of_service: Option<String>,
    pub contact: Option<Contact>,
    pub license: Option<License>,
}

#[derive(Debug, Clone)]
pub struct Contact {
    pub name: Option<String>,
    pub url: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Clone)]
pub struct License {
    pub name: String,
    pub url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PathItem;

// Simple trait for schema generation
pub trait StonehmSchema {
    fn schema() -> String {
        r#"{"type":"object"}"#.to_string()
    }
}

/// Builder that wraps an [`axum::Router`] and accumulates OpenAPI metadata
/// from each `#[api_handler]`-annotated handler you mount.
///
/// The `S` type parameter is the router state, matching axum's
/// `Router<S>`. Default `S = ()` — for stateless routers you don't need to
/// touch this. For stateful routers (e.g. `Router<AppState>`), parameterize
/// `ApiRouter<AppState>` and call `.with_state(state)` to fix the state and
/// recover an `ApiRouter<()>` you can mount the OpenAPI doc routes on.
pub struct ApiRouter<S = ()> {
    router: Router<S>,
    openapi: OpenAPI,
    routes: Vec<RouteInfo>,
    used_schemas: std::collections::HashSet<String>,
}

impl<S> ApiRouter<S>
where
    S: Clone + Send + Sync + 'static,
{
    pub fn new(title: &str, version: &str) -> Self {
        Self {
            router: Router::new(),
            openapi: OpenAPI::new(title, version),
            routes: Vec::new(),
            used_schemas: std::collections::HashSet::new(),
        }
    }

    pub fn route(mut self, path: &str, method_router: axum::routing::MethodRouter<S>) -> Self {
        self.router = self.router.route(path, method_router);
        self
    }

    pub fn get<H, T>(mut self, path: &str, handler: H) -> Self
    where
        H: axum::handler::Handler<T, S>,
        T: 'static,
    {
        let fn_name = std::any::type_name::<H>()
            .split("::")
            .last()
            .unwrap_or("unknown")
            .to_string();

        self.routes.push(RouteInfo {
            path: path.to_string(),
            method: "GET".to_string(),
            function_name: fn_name,
            summary: Some(format!("GET {path}")),
            description: None,
        });
        self.openapi.paths.insert(path.to_string(), PathItem);
        self.route(path, get(handler))
    }

    pub fn post<H, T>(mut self, path: &str, handler: H) -> Self
    where
        H: axum::handler::Handler<T, S>,
        T: 'static,
    {
        let fn_name = std::any::type_name::<H>()
            .split("::")
            .last()
            .unwrap_or("unknown")
            .to_string();

        self.routes.push(RouteInfo {
            path: path.to_string(),
            method: "POST".to_string(),
            function_name: fn_name,
            summary: Some(format!("POST {path}")),
            description: None,
        });
        self.openapi.paths.insert(path.to_string(), PathItem);
        self.route(path, post(handler))
    }

    pub fn put<H, T>(mut self, path: &str, handler: H) -> Self
    where
        H: axum::handler::Handler<T, S>,
        T: 'static,
    {
        let fn_name = std::any::type_name::<H>()
            .split("::")
            .last()
            .unwrap_or("unknown")
            .to_string();

        self.routes.push(RouteInfo {
            path: path.to_string(),
            method: "PUT".to_string(),
            function_name: fn_name,
            summary: Some(format!("PUT {path}")),
            description: None,
        });
        self.openapi.paths.insert(path.to_string(), PathItem);
        self.route(path, put(handler))
    }

    pub fn delete<H, T>(mut self, path: &str, handler: H) -> Self
    where
        H: axum::handler::Handler<T, S>,
        T: 'static,
    {
        let fn_name = std::any::type_name::<H>()
            .split("::")
            .last()
            .unwrap_or("unknown")
            .to_string();

        self.routes.push(RouteInfo {
            path: path.to_string(),
            method: "DELETE".to_string(),
            function_name: fn_name,
            summary: Some(format!("DELETE {path}")),
            description: None,
        });
        self.openapi.paths.insert(path.to_string(), PathItem);
        self.route(path, delete(handler))
    }

    pub fn patch<H, T>(mut self, path: &str, handler: H) -> Self
    where
        H: axum::handler::Handler<T, S>,
        T: 'static,
    {
        let fn_name = std::any::type_name::<H>()
            .split("::")
            .last()
            .unwrap_or("unknown")
            .to_string();

        self.routes.push(RouteInfo {
            path: path.to_string(),
            method: "PATCH".to_string(),
            function_name: fn_name,
            summary: Some(format!("PATCH {path}")),
            description: None,
        });
        self.openapi.paths.insert(path.to_string(), PathItem);
        self.route(path, patch(handler))
    }

    /// Provide the router's state, converting `ApiRouter<S>` into
    /// `ApiRouter<()>`. Mirrors `axum::Router::with_state`. The OpenAPI
    /// metadata is preserved across the conversion.
    pub fn with_state(self, state: S) -> ApiRouter<()> {
        ApiRouter {
            router: self.router.with_state(state),
            openapi: self.openapi,
            routes: self.routes,
            used_schemas: self.used_schemas,
        }
    }
    
    pub fn openapi_spec(&self) -> &OpenAPI {
        &self.openapi
    }
    
    /// Set the API description
    pub fn description(mut self, description: &str) -> Self {
        self.openapi.info.description = Some(description.to_string());
        self
    }
    
    /// Set the terms of service URL
    pub fn terms_of_service(mut self, terms_of_service: &str) -> Self {
        self.openapi.info.terms_of_service = Some(terms_of_service.to_string());
        self
    }
    
    /// Set contact information
    pub fn contact(mut self, name: Option<&str>, url: Option<&str>, email: Option<&str>) -> Self {
        self.openapi.info.contact = Some(Contact {
            name: name.map(|s| s.to_string()),
            url: url.map(|s| s.to_string()),
            email: email.map(|s| s.to_string()),
        });
        self
    }
    
    /// Set contact email only
    pub fn contact_email(mut self, email: &str) -> Self {
        self.openapi.info.contact = Some(Contact {
            name: None,
            url: None,
            email: Some(email.to_string()),
        });
        self
    }
    
    /// Set license information
    pub fn license(mut self, name: &str, url: Option<&str>) -> Self {
        self.openapi.info.license = Some(License {
            name: name.to_string(),
            url: url.map(|s| s.to_string()),
        });
        self
    }
    
    /// Add a tag definition
    pub fn tag(mut self, name: &str, description: Option<&str>) -> Self {
        self.openapi.tags.push(Tag {
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            external_docs: None,
        });
        self
    }
    
    /// Add a tag with external documentation
    pub fn tag_with_docs(mut self, name: &str, description: Option<&str>, docs_description: Option<&str>, docs_url: &str) -> Self {
        self.openapi.tags.push(Tag {
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            external_docs: Some(ExternalDocs {
                description: docs_description.map(|s| s.to_string()),
                url: docs_url.to_string(),
            }),
        });
        self
    }
    
    /// Build the OpenAPI document as a `serde_json::Value`. Does the same work
    /// as `openapi_json()` but returns the structured form — useful when you
    /// want to merge in extra fields (e.g. `servers`, `security`) before
    /// serializing.
    pub fn openapi_value(&mut self) -> Value {
        self.used_schemas.clear();

        // info { title, version, [description, termsOfService, contact, license] }
        let mut info = JsonMap::new();
        info.insert("title".into(), Value::String(self.openapi.info.title.clone()));
        info.insert("version".into(), Value::String(self.openapi.info.version.clone()));
        if let Some(ref description) = self.openapi.info.description {
            info.insert("description".into(), Value::String(description.clone()));
        }
        if let Some(ref terms_of_service) = self.openapi.info.terms_of_service {
            info.insert("termsOfService".into(), Value::String(terms_of_service.clone()));
        }
        if let Some(ref contact) = self.openapi.info.contact {
            let mut c = JsonMap::new();
            if let Some(ref name) = contact.name { c.insert("name".into(), Value::String(name.clone())); }
            if let Some(ref url) = contact.url { c.insert("url".into(), Value::String(url.clone())); }
            if let Some(ref email) = contact.email { c.insert("email".into(), Value::String(email.clone())); }
            if !c.is_empty() {
                info.insert("contact".into(), Value::Object(c));
            }
        }
        if let Some(ref license) = self.openapi.info.license {
            let mut l = JsonMap::new();
            l.insert("name".into(), Value::String(license.name.clone()));
            if let Some(ref url) = license.url {
                l.insert("url".into(), Value::String(url.clone()));
            }
            info.insert("license".into(), Value::Object(l));
        }

        // Collect handler docs once.
        let handler_docs: HashMap<&str, &HandlerDocumentation> = inventory::iter::<HandlerDocumentation>()
            .map(|doc| (doc.function_name, doc))
            .collect();

        // Group routes by OpenAPI path. Multiple methods on the same path get
        // merged into one PathItem object. BTreeMap (not HashMap) so the
        // emitted JSON has stable, alphabetical path ordering — important
        // for test parity (`/openapi.json` and `/api/openapi.json` aliases
        // must serialize byte-identically) and for diff-friendly snapshots.
        let mut path_methods: std::collections::BTreeMap<String, Vec<RouteInfo>> =
            std::collections::BTreeMap::new();
        for route in &self.routes {
            let openapi_path = self.convert_path_to_openapi(&route.path);
            path_methods.entry(openapi_path).or_default().push(route.clone());
        }

        let mut paths = JsonMap::new();
        for (openapi_path, routes) in path_methods {
            let mut methods_obj = JsonMap::new();
            for route in &routes {
                let doc = handler_docs.get(route.function_name.as_str());

                let (summary, description) = match doc {
                    Some(d) => (d.summary.to_string(), d.description.to_string()),
                    None => (
                        route.summary.clone().unwrap_or_else(|| format!("{} {}", route.method, route.path)),
                        "No description available".to_string(),
                    ),
                };

                let mut op = JsonMap::new();
                op.insert("summary".into(), Value::String(summary));
                op.insert("description".into(), Value::String(description));

                if let Some(doc) = doc {
                    if !doc.tags.is_empty() && doc.tags != "[]" {
                        let tags = self.parse_tags_to_openapi(doc.tags);
                        if let Some(arr) = tags.as_array() {
                            if !arr.is_empty() {
                                op.insert("tags".into(), tags);
                            }
                        }
                    }
                    if !doc.parameters.is_empty() && doc.parameters != "[]" {
                        let params = self.parse_parameters_to_openapi(doc.parameters);
                        if let Some(arr) = params.as_array() {
                            if !arr.is_empty() {
                                op.insert("parameters".into(), params);
                            }
                        }
                    }
                    if !doc.request_body.is_empty() && doc.request_body != "[]" {
                        op.insert("requestBody".into(), self.parse_request_body_to_openapi(doc.request_body));
                    }
                    if !doc.responses.is_empty() && doc.responses != "[]" {
                        op.insert("responses".into(), self.parse_responses_to_openapi(doc.responses));
                    } else {
                        op.insert("responses".into(), json!({"200": {"description": "Successful response"}}));
                    }
                } else {
                    op.insert("responses".into(), json!({"200": {"description": "Successful response"}}));
                }

                methods_obj.insert(route.method.to_lowercase(), Value::Object(op));
            }
            paths.insert(openapi_path, Value::Object(methods_obj));
        }

        // Top-level document.
        let mut doc = JsonMap::new();
        doc.insert("openapi".into(), Value::String("3.0.0".into()));
        doc.insert("info".into(), Value::Object(info));
        doc.insert("paths".into(), Value::Object(paths));

        if !self.openapi.tags.is_empty() {
            let tags: Vec<Value> = self.openapi.tags.iter().map(|tag| {
                let mut t = JsonMap::new();
                t.insert("name".into(), Value::String(tag.name.clone()));
                if let Some(ref description) = tag.description {
                    t.insert("description".into(), Value::String(description.clone()));
                }
                if let Some(ref external_docs) = tag.external_docs {
                    let mut ext = JsonMap::new();
                    ext.insert("url".into(), Value::String(external_docs.url.clone()));
                    if let Some(ref d) = external_docs.description {
                        ext.insert("description".into(), Value::String(d.clone()));
                    }
                    t.insert("externalDocs".into(), Value::Object(ext));
                }
                Value::Object(t)
            }).collect();
            doc.insert("tags".into(), Value::Array(tags));
        }

        // components.schemas — only emit schemas that were actually referenced.
        // schema_json is a pre-rendered JSON string from the StonehmSchema derive,
        // so parse it back into a Value so it nests properly.
        //
        // Transitive closure: a parent schema may $ref child schemas
        // (e.g. CommandRequest → SceneCommand → ...). Walk every newly
        // emitted schema for $refs and pull those in too, until the
        // worklist empties. Without this, deep type graphs lose the
        // inner types and downstream tools dereference dangling refs.
        let all_schemas: HashMap<String, Value> = inventory::iter::<SchemaRegistration>()
            .map(|reg| {
                let parsed: Value = serde_json::from_str(reg.schema_json)
                    .unwrap_or_else(|_| json!({"type": "object"}));
                (reg.type_name.to_string(), parsed)
            })
            .collect();

        let mut used_schemas_map = JsonMap::new();
        let mut worklist: Vec<String> = self.used_schemas.iter().cloned().collect();
        let mut emitted: std::collections::HashSet<String> = std::collections::HashSet::new();
        while let Some(name) = worklist.pop() {
            if !emitted.insert(name.clone()) { continue; }
            let Some(schema) = all_schemas.get(&name) else { continue; };
            collect_refs(schema, &mut worklist);
            used_schemas_map.insert(name, schema.clone());
        }

        if !used_schemas_map.is_empty() {
            doc.insert(
                "components".into(),
                json!({"schemas": Value::Object(used_schemas_map)}),
            );
        }

        Value::Object(doc)
    }

    pub fn openapi_json(&mut self) -> String {
        serde_json::to_string(&self.openapi_value())
            .unwrap_or_else(|_| r#"{"openapi":"3.0.0","info":{"title":"error","version":"0"},"paths":{}}"#.to_string())
    }
    
    /// Get a list of unused schemas (schemas that are registered but not referenced in any endpoint)
    pub fn get_unused_schemas(&mut self) -> Vec<String> {
        // If used_schemas is empty, we need to populate it by analyzing the endpoints
        if self.used_schemas.is_empty() {
            // Generate OpenAPI spec to populate used_schemas (but don't use the result)
            let _ = self.openapi_json();
        }
        
        let mut unused_schemas = Vec::new();
        for schema_reg in inventory::iter::<SchemaRegistration>() {
            let schema_name = schema_reg.type_name.to_string();
            if !self.used_schemas.contains(&schema_name) {
                unused_schemas.push(schema_name);
            }
        }
        unused_schemas.sort();
        unused_schemas
    }
    
    /// Get unused schemas without triggering OpenAPI generation (for testing)
    pub fn get_unused_schemas_current(&self) -> Vec<String> {
        let mut unused_schemas = Vec::new();
        for schema_reg in inventory::iter::<SchemaRegistration>() {
            let schema_name = schema_reg.type_name.to_string();
            if !self.used_schemas.contains(&schema_name) {
                unused_schemas.push(schema_name);
            }
        }
        unused_schemas.sort();
        unused_schemas
    }
    
    /// Print warnings for unused schemas
    pub fn warn_unused_schemas(&mut self) {
        let unused = self.get_unused_schemas();
        if !unused.is_empty() {
            eprintln!("Warning: The following schemas are defined but never used in the OpenAPI spec:");
            for schema in &unused {
                eprintln!("  - {schema}");
            }
            eprintln!("Consider removing unused schema definitions or ensuring they are properly referenced in endpoint documentation.");
        }
    }
    
    /// Decode the macro's wire format. The `#[api_handler]` macro stuffs each
    /// rustdoc line into a JSON-array-of-strings literal (e.g. `["foo", "bar"]`).
    /// Returns the lines, or empty vec for `"[]"` / empty / malformed inputs.
    fn decode_doc_lines(s: &str) -> Vec<String> {
        if s.is_empty() || s == "[]" {
            return Vec::new();
        }
        match serde_json::from_str::<Vec<String>>(s) {
            Ok(v) => v,
            Err(_) => Vec::new(),
        }
    }

    fn parse_parameters_to_openapi(&self, params_str: &str) -> Value {
        // Parse parameter strings like "id (path): The unique identifier..."
        // into proper OpenAPI parameter objects.
        let lines = Self::decode_doc_lines(params_str);
        let mut out: Vec<Value> = Vec::new();
        for param in lines {
            if let Some(colon_pos) = param.find(':') {
                let left = param[..colon_pos].trim();
                let description = param[colon_pos + 1..].trim();
                if let (Some(paren_start), Some(paren_end)) = (left.find('('), left.find(')')) {
                    if paren_start < paren_end {
                        let name = left[..paren_start].trim();
                        let param_in = left[paren_start + 1..paren_end].trim();
                        out.push(json!({
                            "name": name,
                            "in": param_in,
                            "description": description,
                            "required": param_in == "path",
                            "schema": {"type": "string"},
                        }));
                        continue;
                    }
                }
            }
            // Fallback for malformed parameter line.
            out.push(json!({
                "name": "unknown",
                "in": "query",
                "description": param,
                "schema": {"type": "string"},
            }));
        }
        Value::Array(out)
    }
    
    fn convert_path_to_openapi(&self, axum_path: &str) -> String {
        // Convert Axum path format (:param) to OpenAPI format ({param})
        axum_path.split('/').map(|segment| {
            if let Some(stripped) = segment.strip_prefix(':') {
                format!("{{{stripped}}}")
            } else {
                segment.to_string()
            }
        }).collect::<Vec<_>>().join("/")
    }
    
    fn parse_request_body_to_openapi(&mut self, request_body_str: &str) -> Value {
        let lines = Self::decode_doc_lines(request_body_str);
        if lines.is_empty() {
            return json!({
                "required": true,
                "content": {"application/json": {"schema": {"type": "object"}}},
            });
        }

        // Same length-then-alpha sort as `parse_responses_to_openapi` — see
        // the comment there for the rationale.
        let mut registered_schemas: Vec<String> = inventory::iter::<SchemaRegistration>()
            .map(|reg| reg.type_name.to_string())
            .collect();
        registered_schemas.sort_by(|a, b| b.len().cmp(&a.len()).then_with(|| a.cmp(b)));
        let registered_set: std::collections::HashSet<&str> =
            registered_schemas.iter().map(|s| s.as_str()).collect();

        // Check for explicit type information first (from the macro enhancement).
        for line in &lines {
            if let Some(type_name) = line.strip_prefix("Type: ") {
                if registered_set.contains(type_name) {
                    self.used_schemas.insert(type_name.to_string());
                    return json!({
                        "required": true,
                        "description": "Request body",
                        "content": {
                            "application/json": {
                                "schema": {"$ref": format!("#/components/schemas/{type_name}")},
                            },
                        },
                    });
                }
            }
        }

        // Fallback: substring-match a registered schema name anywhere in the docs.
        for schema_name in &registered_schemas {
            if lines.iter().any(|l| l.contains(schema_name)) {
                self.used_schemas.insert(schema_name.clone());
                return json!({
                    "required": true,
                    "description": "Request body",
                    "content": {
                        "application/json": {
                            "schema": {"$ref": format!("#/components/schemas/{schema_name}")},
                        },
                    },
                });
            }
        }

        // Otherwise, parse `- name (type): description` field lines.
        let mut description = "Request body".to_string();
        let content_type = "application/json";
        let mut properties = JsonMap::new();

        for line in &lines {
            if line.contains("Content-Type:") {
                continue;
            } else if let Some(field_desc) = line.strip_prefix("- ") {
                if let Some(colon_pos) = field_desc.find(':') {
                    let left = field_desc[..colon_pos].trim();
                    let desc = field_desc[colon_pos + 1..].trim();
                    if let (Some(paren_start), Some(paren_end)) = (left.find('('), left.find(')')) {
                        if paren_start < paren_end {
                            let field_name = left[..paren_start].trim();
                            let field_type = left[paren_start + 1..paren_end].trim();
                            properties.insert(
                                field_name.to_string(),
                                json!({"type": field_type, "description": desc}),
                            );
                        }
                    }
                }
            } else if !line.is_empty() {
                description = line.clone();
            }
        }

        let schema = if properties.is_empty() {
            json!({"type": "object"})
        } else {
            json!({"type": "object", "properties": Value::Object(properties)})
        };

        json!({
            "required": true,
            "description": description,
            "content": {content_type: {"schema": schema}},
        })
    }
    
    fn parse_responses_to_openapi(&mut self, responses_str: &str) -> Value {
        let lines = Self::decode_doc_lines(responses_str);
        if lines.is_empty() {
            return json!({"200": {"description": "Successful response"}});
        }

        // Sort registered schema names by length descending so the
        // substring matcher picks the most specific match (e.g.
        // `ProjectTag` over the `Project` substring inside it). Within
        // equal lengths, sort alphabetically so the choice is also
        // deterministic across runs.
        let mut registered_schemas: Vec<String> = inventory::iter::<SchemaRegistration>()
            .map(|reg| reg.type_name.to_string())
            .collect();
        registered_schemas.sort_by(|a, b| b.len().cmp(&a.len()).then_with(|| a.cmp(b)));

        // Parse response lines like "200: Success" or "404: Not found".
        let responses: Vec<(String, String)> = lines
            .iter()
            .filter_map(|line| {
                let colon_pos = line.find(':')?;
                let status_code = line[..colon_pos].trim();
                let description = line[colon_pos + 1..].trim();
                if status_code.len() == 3 && status_code.chars().all(|c| c.is_ascii_digit()) {
                    Some((status_code.to_string(), description.to_string()))
                } else {
                    None
                }
            })
            .collect();

        if responses.is_empty() {
            return json!({"200": {"description": "Successful response"}});
        }

        let mut out = JsonMap::new();
        for (code, desc) in &responses {
            let value = match code.as_str() {
                "204" => json!({"description": desc}),
                code if code.starts_with('2') => {
                    // 2xx — try to attach a registered response schema.
                    let mut schema: Value = json!({"type": "object", "properties": {}});
                    for schema_name in &registered_schemas {
                        if desc.to_lowercase().contains(&schema_name.to_lowercase()) {
                            self.used_schemas.insert(schema_name.clone());
                            schema = json!({"$ref": format!("#/components/schemas/{schema_name}")});
                            break;
                        }
                    }
                    json!({
                        "description": desc,
                        "content": {"application/json": {"schema": schema}},
                    })
                }
                _ => {
                    // 4xx/5xx — try to attach a registered error schema.
                    let mut error_schema: Option<Value> = None;
                    // Exact name match first.
                    for schema_name in &registered_schemas {
                        if schema_name.ends_with("Error") && desc.contains(schema_name) {
                            self.used_schemas.insert(schema_name.clone());
                            error_schema = Some(json!({"$ref": format!("#/components/schemas/{schema_name}")}));
                            break;
                        }
                    }
                    // Fallback: any `*Error` schema if the description mentions "error".
                    if error_schema.is_none() && desc.to_lowercase().contains("error") {
                        for schema_name in &registered_schemas {
                            if schema_name.ends_with("Error") {
                                self.used_schemas.insert(schema_name.clone());
                                error_schema = Some(json!({"$ref": format!("#/components/schemas/{schema_name}")}));
                                break;
                            }
                        }
                    }
                    match error_schema {
                        Some(s) => json!({
                            "description": desc,
                            "content": {"application/json": {"schema": s}},
                        }),
                        None => json!({"description": desc}),
                    }
                }
            };
            out.insert(code.clone(), value);
        }
        Value::Object(out)
    }
    
    fn parse_tags_to_openapi(&self, tags_str: &str) -> Value {
        let lines = Self::decode_doc_lines(tags_str);
        Value::Array(lines.into_iter().map(Value::String).collect())
    }
    
    /// Mount the OpenAPI document at `/openapi.json`.
    ///
    /// Snapshots the spec at the time of this call, so handlers added
    /// later via `.merge(...)` won't appear in the served document.
    /// For multi-module aggregation use [`aggregated_openapi_value`]
    /// from a custom handler instead.
    pub fn with_openapi_routes(mut self) -> Self {
        let json_spec = self.openapi_json();
        let router = self.router
            .route("/openapi.json", get(move || async move {
                axum::Json(json_spec)
            }));

        Self { router, openapi: self.openapi, routes: self.routes, used_schemas: self.used_schemas }
    }

    /// Like [`with_openapi_routes`](Self::with_openapi_routes) but
    /// mounts at `<prefix>.json` instead of `/openapi.json`. Pass an
    /// empty prefix to use the default `/openapi`.
    pub fn with_openapi_routes_prefix(mut self, prefix: &str) -> Self {
        let json_spec = self.openapi_json();

        // Normalize the prefix.
        let normalized_prefix = if prefix.is_empty() {
            "/openapi".to_string()
        } else if prefix.starts_with('/') {
            prefix.trim_end_matches('/').to_string()
        } else {
            format!("/{}", prefix.trim_end_matches('/'))
        };

        let json_path = format!("{normalized_prefix}.json");

        let router = self.router
            .route(&json_path, get(move || async move {
                axum::Json(json_spec)
            }));

        Self { router, openapi: self.openapi, routes: self.routes, used_schemas: self.used_schemas }
    }

    pub fn into_router(self) -> Router<S> {
        self.router
    }

    /// Snapshot the OpenAPI metadata accumulated so far, then return the
    /// underlying axum router. The snapshot is appended to the
    /// process-wide registry (`global_routes`) so a downstream
    /// aggregator (one OpenAPI doc serving the whole app) can stitch
    /// every module's routes into a single spec without each module
    /// having to plumb its `ApiRouter` instance up to a shared
    /// builder.
    ///
    /// The snapshot includes the `info` block — title + version are
    /// kept per-module so the aggregator can pick whichever one it
    /// prefers (typically the first registered, or a hand-written
    /// override).
    pub fn finalize(self) -> Router<S> {
        let routes = self.routes.clone();
        let info = self.openapi.info.clone();
        // Push to the global registry. If the lock is poisoned a
        // previous module crashed mid-registration; keep going so
        // we don't tear down the whole app for a docs glitch.
        let registry = global_route_registry();
        if let Ok(mut entries) = registry.lock() {
            entries.push(GlobalRoutesEntry { info, routes });
        }
        self.router
    }
}

/// Walk a JSON schema and collect every type name referenced via
/// `{"$ref": "#/components/schemas/<name>"}`. Used to compute the
/// transitive closure of schemas the OpenAPI doc needs.
fn collect_refs(value: &Value, out: &mut Vec<String>) {
    const PREFIX: &str = "#/components/schemas/";
    match value {
        Value::Object(map) => {
            if let Some(Value::String(s)) = map.get("$ref") {
                if let Some(name) = s.strip_prefix(PREFIX) {
                    out.push(name.to_string());
                }
            }
            for v in map.values() {
                collect_refs(v, out);
            }
        }
        Value::Array(arr) => {
            for v in arr { collect_refs(v, out); }
        }
        _ => {}
    }
}

/// One entry in the process-wide route registry. Each call to
/// `ApiRouter::finalize` pushes one of these.
#[derive(Clone)]
pub struct GlobalRoutesEntry {
    pub info: Info,
    pub routes: Vec<RouteInfo>,
}

/// The process-wide route registry. `finalize()` writes here;
/// `aggregated_openapi_value` reads here.
fn global_route_registry() -> &'static std::sync::Mutex<Vec<GlobalRoutesEntry>> {
    use std::sync::{Mutex, OnceLock};
    static REGISTRY: OnceLock<Mutex<Vec<GlobalRoutesEntry>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(Vec::new()))
}

/// Build a single OpenAPI document covering every route that any
/// `ApiRouter` has registered via `finalize` so far. Use this from
/// your top-level `/openapi.json` handler when your app is split
/// across many modules — each module calls `.finalize()` on its
/// `ApiRouter` to mount the actual axum routes; this helper stitches
/// the OpenAPI metadata back together.
///
/// The `title` and `version` are taken from the first registered
/// entry — pass overrides via the `info` mutator if you need to
/// merge in `description`, `contact`, etc.
pub fn aggregated_openapi_value() -> Value {
    let registry = global_route_registry();
    let entries: Vec<GlobalRoutesEntry> = registry.lock()
        .map(|guard| guard.clone())
        .unwrap_or_default();

    let (title, version) = entries.first()
        .map(|e| (e.info.title.clone(), e.info.version.clone()))
        .unwrap_or_else(|| ("API".to_string(), "0.0.0".to_string()));

    let mut router = ApiRouter::<()>::new(&title, &version);
    for entry in entries {
        for r in entry.routes {
            router.routes.push(r);
        }
    }
    router.openapi_value()
}

// Macro to create API router. Defaults to `ApiRouter<()>` (no state); for
// stateful routers, use `ApiRouter::<MyState>::new(title, version)` directly.
#[macro_export]
macro_rules! api_router {
    ($title:expr, $version:expr) => {
        $crate::ApiRouter::<()>::new($title, $version)
    };
}

// Re-export inventory for macros
pub use inventory;

// Re-export serde_json for macros
pub use serde_json;

// Re-export proc macros
pub use stonehm_macros::{api_handler, StonehmSchema, api_error};

#[cfg(test)]
mod tests {
    use super::*;

    // Test schema registrations
    inventory::submit! {
        SchemaRegistration {
            type_name: "UserData",
            schema_json: r#"{"type": "object", "properties": {"name": {"type": "string"}, "email": {"type": "string"}}, "required": ["name", "email"]}"#,
        }
    }

    inventory::submit! {
        SchemaRegistration {
            type_name: "CreateUserRequest",
            schema_json: r#"{"type": "object", "properties": {"name": {"type": "string"}, "email": {"type": "string"}, "age": {"type": "number"}}, "required": ["name", "email", "age"]}"#,
        }
    }

    inventory::submit! {
        SchemaRegistration {
            type_name: "UpdateUserRequest", 
            schema_json: r#"{"type": "object", "properties": {"name": {"type": "string"}, "email": {"type": "string"}}, "required": ["name", "email"]}"#,
        }
    }

    inventory::submit! {
        SchemaRegistration {
            type_name: "GreetResponse",
            schema_json: r#"{"type": "object", "properties": {"message": {"type": "string"}, "style": {"type": "string"}}, "required": ["message", "style"]}"#,
        }
    }

    inventory::submit! {
        SchemaRegistration {
            type_name: "DeleteUserError",
            schema_json: r#"{"type": "object", "properties": {"error": {"type": "object"}}}"#,
        }
    }

    inventory::submit! {
        SchemaRegistration {
            type_name: "GreetError",
            schema_json: r#"{"type": "object", "properties": {"error": {"type": "object"}}}"#,
        }
    }

    inventory::submit! {
        SchemaRegistration {
            type_name: "UserResponse",
            schema_json: r#"{"type": "object", "properties": {"id": {"type": "integer"}, "name": {"type": "string"}, "email": {"type": "string"}}, "required": ["id", "name", "email"]}"#,
        }
    }

    inventory::submit! {
        SchemaRegistration {
            type_name: "GetUserError",
            schema_json: r#"{"type": "object", "properties": {"error": {"type": "object"}}}"#,
        }
    }

    inventory::submit! {
        SchemaRegistration {
            type_name: "CreateUserError",
            schema_json: r#"{"type": "object", "properties": {"error": {"type": "object"}}}"#,
        }
    }

    #[test]
    fn test_api_router_creation() {
        let router = ApiRouter::<()>::new("Test API", "1.0.0");
        let spec = router.openapi_spec();

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
    fn test_api_description() {
        let router = api_router!("Test API", "1.0.0")
            .description("Test API for testing");
            
        let spec = router.openapi_spec();
        assert_eq!(spec.info.description, Some("Test API for testing".to_string()));
    }

    #[test]
    fn test_terms_of_service() {
        let router = api_router!("Test API", "1.0.0")
            .terms_of_service("https://example.com/terms");
            
        let spec = router.openapi_spec();
        assert_eq!(spec.info.terms_of_service, Some("https://example.com/terms".to_string()));
    }

    #[test]
    fn test_contact_info() {
        let router = api_router!("Test API", "1.0.0")
            .contact(Some("Test Team"), Some("https://example.com"), Some("test@example.com"));
            
        let spec = router.openapi_spec();
        assert!(spec.info.contact.is_some());
        
        let contact = spec.info.contact.as_ref().unwrap();
        assert_eq!(contact.name, Some("Test Team".to_string()));
        assert_eq!(contact.url, Some("https://example.com".to_string()));
        assert_eq!(contact.email, Some("test@example.com".to_string()));
    }

    #[test]
    fn test_contact_email_only() {
        let router = api_router!("Test API", "1.0.0")
            .contact_email("test@example.com");
            
        let spec = router.openapi_spec();
        assert!(spec.info.contact.is_some());
        
        let contact = spec.info.contact.as_ref().unwrap();
        assert_eq!(contact.email, Some("test@example.com".to_string()));
        assert_eq!(contact.name, None);
        assert_eq!(contact.url, None);
    }

    #[test]
    fn test_license() {
        let router = api_router!("Test API", "1.0.0")
            .license("MIT", Some("https://opensource.org/licenses/MIT"));
            
        let spec = router.openapi_spec();
        assert!(spec.info.license.is_some());
        
        let license = spec.info.license.as_ref().unwrap();
        assert_eq!(license.name, "MIT");
        assert_eq!(license.url, Some("https://opensource.org/licenses/MIT".to_string()));
    }

    #[test]
    fn test_tag_addition() {
        let router = api_router!("Test API", "1.0.0")
            .tag("users", Some("User operations"))
            .tag("admin", None);
            
        let spec = router.openapi_spec();
        assert_eq!(spec.tags.len(), 2);
        
        assert_eq!(spec.tags[0].name, "users");
        assert_eq!(spec.tags[0].description, Some("User operations".to_string()));
        
        assert_eq!(spec.tags[1].name, "admin");
        assert_eq!(spec.tags[1].description, None);
    }

    #[test]
    fn test_tag_with_external_docs() {
        let router = api_router!("Test API", "1.0.0")
            .tag_with_docs(
                "users", 
                Some("User operations"), 
                Some("Learn more"), 
                "https://example.com/docs"
            );
            
        let spec = router.openapi_spec();
        assert_eq!(spec.tags.len(), 1);
        
        let tag = &spec.tags[0];
        assert_eq!(tag.name, "users");
        assert_eq!(tag.description, Some("User operations".to_string()));
        assert!(tag.external_docs.is_some());
        
        let docs = tag.external_docs.as_ref().unwrap();
        assert_eq!(docs.description, Some("Learn more".to_string()));
        assert_eq!(docs.url, "https://example.com/docs");
    }

    #[test]
    fn test_convert_path_to_openapi() {
        let router = api_router!("Test API", "1.0.0");
        
        assert_eq!(router.convert_path_to_openapi("/users/:id"), "/users/{id}");
        assert_eq!(router.convert_path_to_openapi("/users/:id/posts/:post_id"), "/users/{id}/posts/{post_id}");
        assert_eq!(router.convert_path_to_openapi("/static"), "/static");
        assert_eq!(router.convert_path_to_openapi("/"), "/");
    }

    #[test]
    fn test_parse_parameters_to_openapi() {
        let router = api_router!("Test API", "1.0.0");

        // Test empty parameters — returns an empty array.
        assert_eq!(router.parse_parameters_to_openapi("[]"), serde_json::json!([]));

        // Test path parameter
        let params = r#"["id (path): The user ID"]"#;
        let result = router.parse_parameters_to_openapi(params);
        assert_eq!(result[0]["name"], "id");
        assert_eq!(result[0]["in"], "path");
        assert_eq!(result[0]["required"], true);

        // Test query parameter
        let params = r#"["filter (query): Filter results"]"#;
        let result = router.parse_parameters_to_openapi(params);
        assert_eq!(result[0]["name"], "filter");
        assert_eq!(result[0]["in"], "query");
        assert_eq!(result[0]["required"], false);
    }

    #[test]
    fn test_parse_responses_to_openapi() {
        let mut router = api_router!("Test API", "1.0.0");

        // Test empty responses
        let result = router.parse_responses_to_openapi("[]");
        assert_eq!(result["200"]["description"], "Successful response");

        // Test simple responses
        let responses = r#"["200: Success", "404: Not found"]"#;
        let result = router.parse_responses_to_openapi(responses);
        assert_eq!(result["200"]["description"], "Success");
        // 2xx responses have a content section.
        assert!(result["200"]["content"]["application/json"].is_object());
        assert_eq!(result["404"]["description"], "Not found");
    }

    #[test]
    fn test_parse_tags_to_openapi() {
        let router = api_router!("Test API", "1.0.0");

        // Test empty tags
        assert_eq!(router.parse_tags_to_openapi("[]"), serde_json::json!([]));
        assert_eq!(router.parse_tags_to_openapi(""), serde_json::json!([]));

        // Test single tag
        let result = router.parse_tags_to_openapi(r#"["users"]"#);
        assert_eq!(result, serde_json::json!(["users"]));

        // Test multiple tags
        let result = router.parse_tags_to_openapi(r#"["users", "admin"]"#);
        assert_eq!(result, serde_json::json!(["users", "admin"]));
    }

    #[test]
    fn test_openapi_json_structure() {
        let mut router = api_router!("Test API", "1.0.0")
            .description("Test Description")
            .tag("test", Some("Test operations"));

        let value = router.openapi_value();

        assert_eq!(value["openapi"], "3.0.0");
        assert_eq!(value["info"]["title"], "Test API");
        assert_eq!(value["info"]["version"], "1.0.0");
        assert_eq!(value["info"]["description"], "Test Description");
        assert!(value["paths"].is_object());
        assert!(value["tags"].is_array());
    }

    #[test]
    fn test_response_schema_references() {
        let mut router = api_router!("Test", "1.0");
        let responses = r#"["200: Returns a personalized GreetResponse message"]"#;
        let result = router.parse_responses_to_openapi(responses);
        assert_eq!(
            result["200"]["content"]["application/json"]["schema"]["$ref"],
            "#/components/schemas/GreetResponse"
        );
    }

    #[test]
    fn test_error_response_schema_references() {
        let mut router = api_router!("Test", "1.0");
        let responses = r#"["404: User not found DeleteUserError", "403: Insufficient permissions DeleteUserError"]"#;
        let result = router.parse_responses_to_openapi(responses);
        assert_eq!(
            result["404"]["content"]["application/json"]["schema"]["$ref"],
            "#/components/schemas/DeleteUserError"
        );
        assert_eq!(
            result["403"]["content"]["application/json"]["schema"]["$ref"],
            "#/components/schemas/DeleteUserError"
        );
    }

    #[test]
    fn test_user_response_schema_references() {
        let mut router = api_router!("Test", "1.0");
        let responses = r#"["200: Successfully retrieved UserResponse information", "201: User successfully created UserResponse"]"#;
        let result = router.parse_responses_to_openapi(responses);
        assert_eq!(
            result["200"]["content"]["application/json"]["schema"]["$ref"],
            "#/components/schemas/UserResponse"
        );
        assert_eq!(
            result["201"]["content"]["application/json"]["schema"]["$ref"],
            "#/components/schemas/UserResponse"
        );
    }

    #[test]
    fn test_mixed_response_types() {
        let mut router = api_router!("Test", "1.0");
        let responses = r#"["200: Returns GreetResponse", "400: Invalid request GreetError"]"#;
        let result = router.parse_responses_to_openapi(responses);
        assert_eq!(
            result["200"]["content"]["application/json"]["schema"]["$ref"],
            "#/components/schemas/GreetResponse"
        );
        assert_eq!(
            result["400"]["content"]["application/json"]["schema"]["$ref"],
            "#/components/schemas/GreetError"
        );
    }

    #[test]
    fn test_get_user_error_schema_references() {
        let mut router = api_router!("Test", "1.0");
        let responses = r#"["404: User not found for the given ID GetUserError", "400: Invalid user ID format GetUserError"]"#;
        let result = router.parse_responses_to_openapi(responses);
        assert_eq!(
            result["404"]["content"]["application/json"]["schema"]["$ref"],
            "#/components/schemas/GetUserError"
        );
        assert_eq!(
            result["400"]["content"]["application/json"]["schema"]["$ref"],
            "#/components/schemas/GetUserError"
        );
    }

    #[test]
    fn test_create_user_error_schema_references() {
        let mut router = api_router!("Test", "1.0");
        let responses = r#"["400: Invalid input data provided CreateUserError", "500: Internal server error occurred CreateUserError"]"#;
        let result = router.parse_responses_to_openapi(responses);
        assert_eq!(
            result["400"]["content"]["application/json"]["schema"]["$ref"],
            "#/components/schemas/CreateUserError"
        );
        assert_eq!(
            result["500"]["content"]["application/json"]["schema"]["$ref"],
            "#/components/schemas/CreateUserError"
        );
    }

    #[test]
    fn test_all_error_types_coverage() {
        let mut router = api_router!("Test", "1.0");
        let responses = r#"["400: GetUserError response", "401: CreateUserError response", "403: DeleteUserError response", "422: GreetError response"]"#;
        let result = router.parse_responses_to_openapi(responses);
        assert_eq!(
            result["400"]["content"]["application/json"]["schema"]["$ref"],
            "#/components/schemas/GetUserError"
        );
        assert_eq!(
            result["401"]["content"]["application/json"]["schema"]["$ref"],
            "#/components/schemas/CreateUserError"
        );
        assert_eq!(
            result["403"]["content"]["application/json"]["schema"]["$ref"],
            "#/components/schemas/DeleteUserError"
        );
        assert_eq!(
            result["422"]["content"]["application/json"]["schema"]["$ref"],
            "#/components/schemas/GreetError"
        );
    }

    #[test]
    fn test_unused_schema_detection() {
        let mut router = api_router!("Test", "1.0");
        
        // Use some schemas first
        let _ = router.parse_responses_to_openapi(r#"["200: Successfully retrieved UserResponse information", "404: User not found GetUserError"]"#);
        
        // Now check what's used vs unused
        let all_schemas_count = inventory::iter::<SchemaRegistration>().count();
        let unused = router.get_unused_schemas();
        
        // Should have some unused schemas
        assert!(!unused.is_empty());
        assert!(unused.len() < all_schemas_count);
        
        // Should not include schemas we just used
        assert!(!unused.contains(&"UserResponse".to_string()));
        assert!(!unused.contains(&"GetUserError".to_string()));
        
        // Should include schemas we didn't use
        assert!(unused.contains(&"CreateUserRequest".to_string()) || 
                unused.contains(&"UpdateUserRequest".to_string()));
    }

    #[test]
    fn test_openapi_only_includes_used_schemas() {
        let mut router = api_router!("Test", "1.0");
        
        // The test doesn't need to manually track schemas - the openapi_json() method 
        // should track schemas from actual handler documentation. Since we don't have 
        // handlers registered in this test, we need to verify that the openapi_json 
        // method correctly excludes unused schemas.
        
        let openapi_json = router.openapi_json();
        
        // Since no handlers are registered, no schemas should be included
        assert!(!openapi_json.contains("GreetResponse"));
        assert!(!openapi_json.contains("GreetError"));
        assert!(!openapi_json.contains("DeleteUserError"));
        assert!(!openapi_json.contains("CreateUserError"));
        assert!(!openapi_json.contains("UserResponse"));
        
        // Should have empty paths since no routes registered
        assert!(openapi_json.contains(r#""paths":{}"#));
    }

    #[test]
    fn test_warn_unused_schemas_output() {
        let mut router = api_router!("Test", "1.0");
        
        // This should identify unused schemas (all test schemas since we don't use any)
        let unused = router.get_unused_schemas();
        assert!(!unused.is_empty());
        
        // Test passes if we can identify unused schemas
        assert!(unused.contains(&"CreateUserRequest".to_string()) || 
                unused.contains(&"UserData".to_string()) ||
                unused.contains(&"UpdateUserRequest".to_string()));
    }

    #[test]
    fn test_with_openapi_routes_prefix_normalization() {
        let test_cases = vec![
            ("", "/openapi.json"), // Empty prefix defaults to /openapi
            ("/openapi", "/openapi.json"),
            ("openapi", "/openapi.json"),
            ("/api/docs", "/api/docs.json"),
            ("/api/docs/", "/api/docs.json"),
            ("api/docs", "/api/docs.json"),
            ("api/docs/", "/api/docs.json"),
        ];
        
        for (prefix, _expected_json) in test_cases {
            let router = api_router!("Test API", "1.0.0");
            
            // The normalized prefix is used internally by with_openapi_routes_prefix
            // We can't directly test the result, but we can verify it doesn't panic
            let _router = router.with_openapi_routes_prefix(prefix);
            
            // If we could inspect the routes, we would verify:
            // assert!(router has route at expected_json);
            // assert!(router has route at expected_yaml);
        }
    }

    #[test]
    fn test_route_tracking() {
        let router = api_router!("Test API", "1.0.0");

        // Track initial state
        assert_eq!(router.routes.len(), 0);

        // Note: We can't fully test route tracking without proper handler types,
        // but we can verify the structure exists and basic operations work
    }

    /// Smoke test for the `ApiRouter<S>` state generic. Confirms the builder
    /// can carry a state type through `.get(...)`, that handlers using
    /// `axum::extract::State<S>` typecheck, and that `with_state` collapses
    /// the state to produce an `ApiRouter<()>` you can mount on a stateless
    /// router.
    #[test]
    fn test_stateful_api_router() {
        use axum::extract::State;

        #[derive(Clone)]
        struct AppState {
            #[allow(dead_code)]
            name: &'static str,
        }

        async fn ping(State(_s): State<AppState>) -> &'static str { "pong" }

        let router: ApiRouter<AppState> = ApiRouter::<AppState>::new("Stateful API", "1.0.0")
            .get("/ping", ping);
        assert_eq!(router.routes.len(), 1);
        assert_eq!(router.routes[0].method, "GET");
        assert_eq!(router.routes[0].path, "/ping");

        // Collapse the state — produces an ApiRouter<()> that's mountable.
        let stateless: ApiRouter<()> = router.with_state(AppState { name: "test" });
        assert_eq!(stateless.routes.len(), 1);
        let _final: axum::Router = stateless.into_router();
    }
}

#[cfg(test)]
mod handler_tests {
    use super::*;
    
    // Test helpers to simulate different handler documentation scenarios
    fn create_test_router() -> ApiRouter {
        api_router!("Handler Test API", "1.0.0")
    }
    
    fn simulate_handler_registration(
        _router: &ApiRouter,
        function_name: &'static str,
        summary: &'static str,
        description: &'static str,
        parameters: &'static str,
        responses: &'static str,
        request_body: &'static str,
        tags: &'static str,
    ) -> HandlerDocumentation {
        // Simulate what the api_handler macro would register
        HandlerDocumentation {
            function_name,
            summary,
            description,
            parameters,
            responses,
            request_body,
            tags,
        }
    }
    
    #[test]
    fn test_simple_get_handler_no_params() {
        let router = create_test_router();
        
        // Simulate a simple GET handler with no parameters
        let docs = simulate_handler_registration(
            &router,
            "list_items",
            "List all items",
            "Returns a list of all available items",
            "[]",
            r#"["200: Returns list of items"]"#,
            "[]",
            r#"["items"]"#,
        );
        
        assert_eq!(docs.function_name, "list_items");
        assert_eq!(docs.summary, "List all items");
        assert!(docs.parameters.contains("[]"));
        assert!(docs.request_body.contains("[]"));
    }
    
    #[test]
    fn test_get_handler_with_path_param() {
        let router = create_test_router();
        
        // Simulate GET /users/:id handler
        let docs = simulate_handler_registration(
            &router,
            "get_user",
            "Get user by ID",
            "Retrieves a specific user by their ID",
            r#"["id (path): The user's unique identifier"]"#,
            r#"["200: User found", "404: User not found"]"#,
            "[]",
            r#"["users"]"#,
        );
        
        assert!(docs.parameters.contains("id (path)"));
        assert!(docs.responses.contains("404: User not found"));
    }
    
    #[test]
    fn test_post_handler_with_json_body() {
        let router = create_test_router();
        
        // Simulate POST with JSON body
        let docs = simulate_handler_registration(
            &router,
            "create_user",
            "Create new user",
            "Creates a new user account",
            "[]",
            r#"["201: User created", "400: Invalid input"]"#,
            r#"["Type: CreateUserRequest", "Content-Type: application/json", "User creation data"]"#,
            r#"["users", "admin"]"#,
        );
        
        assert!(docs.request_body.contains("Type: CreateUserRequest"));
        assert!(docs.request_body.contains("application/json"));
        assert!(docs.tags.contains("admin"));
    }
    
    #[test]
    fn test_handler_with_query_params() {
        let router = create_test_router();
        
        // Simulate GET with query parameters
        let docs = simulate_handler_registration(
            &router,
            "search_users",
            "Search users",
            "Search for users with filters",
            r#"["q (query): Search query", "limit (query): Maximum results", "offset (query): Pagination offset"]"#,
            r#"["200: Search results"]"#,
            "[]",
            r#"["users", "search"]"#,
        );
        
        assert!(docs.parameters.contains("q (query)"));
        assert!(docs.parameters.contains("limit (query)"));
        assert!(docs.parameters.contains("offset (query)"));
    }
    
    #[test]
    fn test_handler_with_multiple_path_params() {
        let router = create_test_router();
        
        // Simulate /organizations/:org_id/users/:user_id
        let docs = simulate_handler_registration(
            &router,
            "get_org_user",
            "Get organization user",
            "Get a specific user within an organization",
            r#"["org_id (path): Organization ID", "user_id (path): User ID"]"#,
            r#"["200: User details", "404: Not found", "403: Access denied"]"#,
            "[]",
            r#"["organizations", "users"]"#,
        );
        
        assert!(docs.parameters.contains("org_id (path)"));
        assert!(docs.parameters.contains("user_id (path)"));
        assert!(docs.responses.contains("403: Access denied"));
    }
    
    #[test]
    fn test_handler_with_header_params() {
        let router = create_test_router();
        
        // Simulate handler with header parameters
        let docs = simulate_handler_registration(
            &router,
            "authenticated_endpoint",
            "Authenticated endpoint",
            "Requires authentication token",
            r#"["Authorization (header): Bearer token", "X-Request-ID (header): Request tracking ID"]"#,
            r#"["200: Success", "401: Unauthorized"]"#,
            "[]",
            r#"["auth"]"#,
        );
        
        assert!(docs.parameters.contains("Authorization (header)"));
        assert!(docs.parameters.contains("X-Request-ID (header)"));
        assert!(docs.responses.contains("401: Unauthorized"));
    }
    
    #[test]
    fn test_delete_handler_with_responses() {
        let router = create_test_router();
        
        // Simulate DELETE handler
        let docs = simulate_handler_registration(
            &router,
            "delete_user",
            "Delete user",
            "Permanently delete a user account",
            r#"["id (path): User ID to delete"]"#,
            r#"["204: User deleted", "404: User not found", "403: Cannot delete admin"]"#,
            "[]",
            r#"["users", "admin"]"#,
        );
        
        assert!(docs.responses.contains("204: User deleted"));
        assert!(!docs.responses.contains("200")); // Should not have 200 for DELETE
    }
    
    #[test]
    fn test_put_handler_with_body() {
        let router = create_test_router();
        
        // Simulate PUT handler
        let docs = simulate_handler_registration(
            &router,
            "update_user",
            "Update user",
            "Update an existing user",
            r#"["id (path): User ID"]"#,
            r#"["200: User updated", "404: User not found", "400: Invalid data"]"#,
            r#"["Type: UpdateUserRequest", "Content-Type: application/json", "Updated user data"]"#,
            r#"["users"]"#,
        );
        
        assert!(docs.request_body.contains("Type: UpdateUserRequest"));
        assert!(docs.responses.contains("200: User updated"));
    }
    
    #[test]
    fn test_patch_handler_partial_update() {
        let router = create_test_router();
        
        // Simulate PATCH handler
        let docs = simulate_handler_registration(
            &router,
            "patch_user",
            "Partially update user",
            "Update specific fields of a user",
            r#"["id (path): User ID"]"#,
            r#"["200: User updated", "404: User not found"]"#,
            r#"["Type: PatchUserRequest", "Content-Type: application/json", "Partial user data"]"#,
            r#"["users"]"#,
        );
        
        assert!(docs.request_body.contains("Partial user data"));
    }
    
    #[test]
    fn test_handler_with_complex_responses() {
        let router = create_test_router();
        
        // Simulate handler with detailed response documentation
        let docs = simulate_handler_registration(
            &router,
            "complex_endpoint",
            "Complex endpoint",
            "Endpoint with detailed responses",
            "[]",
            r#"["200: Success with data", "400: Bad request with validation errors", "401: Authentication required", "403: Insufficient permissions", "500: Internal server error"]"#,
            "[]",
            r#"["complex"]"#,
        );
        
        // Verify all response codes are captured
        assert!(docs.responses.contains("200:"));
        assert!(docs.responses.contains("400:"));
        assert!(docs.responses.contains("401:"));
        assert!(docs.responses.contains("403:"));
        assert!(docs.responses.contains("500:"));
    }
    
    #[test]
    fn test_handler_without_documentation() {
        let router = create_test_router();
        
        // Simulate handler with minimal/no documentation
        let docs = simulate_handler_registration(
            &router,
            "undocumented_handler",
            "No summary",
            "No description",
            "[]",
            "[]",
            "[]",
            "[]",
        );
        
        assert_eq!(docs.summary, "No summary");
        assert_eq!(docs.description, "No description");
        assert_eq!(docs.parameters, "[]");
        assert_eq!(docs.responses, "[]");
    }
    
    #[test]
    fn test_request_body_parsing() {
        let mut router = create_test_router();
        let json_body = r#"["Type: UserData", "Content-Type: application/json", "- name (string): User name", "- email (string): User email"]"#;
        let result = router.parse_request_body_to_openapi(json_body);
        // UserData is registered via inventory::submit! at module load,
        // so the "Type: UserData" hint produces a $ref to the registered schema.
        assert_eq!(result["required"], true);
        assert_eq!(
            result["content"]["application/json"]["schema"]["$ref"],
            "#/components/schemas/UserData"
        );
    }

    #[test]
    fn test_multiple_tags_parsing() {
        let router = create_test_router();
        let tags = r#"["users", "admin", "v2"]"#;
        let result = router.parse_tags_to_openapi(tags);
        assert_eq!(result, serde_json::json!(["users", "admin", "v2"]));
    }

    #[test]
    fn test_special_status_codes() {
        let mut router = create_test_router();
        let responses = r#"["204: No content", "201: Created with Location header", "202: Accepted for processing"]"#;
        let result = router.parse_responses_to_openapi(responses);
        // 204 has no content section.
        assert_eq!(result["204"]["description"], "No content");
        assert!(result["204"].get("content").is_none());
        // 2xx success responses have a content section.
        assert_eq!(result["201"]["description"], "Created with Location header");
        assert!(result["201"]["content"]["application/json"].is_object());
    }

    #[test]
    fn test_error_response_parsing() {
        let mut router = create_test_router();
        let responses = r#"["400: Validation failed", "409: Conflict with existing resource", "422: Unprocessable entity"]"#;
        let result = router.parse_responses_to_openapi(responses);
        // Error responses don't get a content section unless the description
        // matches a registered *Error schema name.
        assert_eq!(result["400"]["description"], "Validation failed");
        assert!(result["400"].get("content").is_none());
        assert_eq!(result["409"]["description"], "Conflict with existing resource");
        assert!(result["409"].get("content").is_none());
        assert_eq!(result["422"]["description"], "Unprocessable entity");
        assert!(result["422"].get("content").is_none());
    }
    
    #[test]
    fn test_handler_with_all_param_types() {
        let router = create_test_router();
        
        // Test handler with path, query, and header params
        let docs = simulate_handler_registration(
            &router,
            "complex_params",
            "Complex parameters",
            "Handler with all parameter types",
            r#"["id (path): Resource ID", "filter (query): Filter criteria", "sort (query): Sort order", "Authorization (header): Auth token"]"#,
            r#"["200: Success"]"#,
            r#"["Type: FilterRequest", "Content-Type: application/json"]"#,
            r#"["complex"]"#,
        );
        
        assert!(docs.parameters.contains("(path)"));
        assert!(docs.parameters.contains("(query)"));
        assert!(docs.parameters.contains("(header)"));
    }
    
    #[test]
    fn test_openapi_json_generation_with_handlers() {
        let mut router = create_test_router();
        
        // Simulate adding routes
        router.routes.push(RouteInfo {
            path: "/users".to_string(),
            method: "GET".to_string(),
            function_name: "list_users".to_string(),
            summary: Some("List users".to_string()),
            description: None,
        });
        
        router.routes.push(RouteInfo {
            path: "/users/:id".to_string(),
            method: "GET".to_string(),
            function_name: "get_user".to_string(),
            summary: Some("Get user".to_string()),
            description: None,
        });
        
        let json = router.openapi_json();
        
        // Verify paths are included
        assert!(json.contains(r#""/users""#));
        assert!(json.contains(r#""/users/{id}""#)); // Converted from :id
        assert!(json.contains(r#""get":"#));
    }
    
    #[test]
    fn test_schema_reference_in_responses() {
        let mut router = create_test_router();
        let responses = r#"["200: Successfully retrieved user information"]"#;
        let result = router.parse_responses_to_openapi(responses);
        assert_eq!(result["200"]["description"], "Successfully retrieved user information");
    }
    
    #[test]
    fn test_empty_prefix_handling() {
        let router = create_test_router();
        
        // Empty prefix should default to /openapi
        let router_with_routes = router.with_openapi_routes_prefix("");
        
        // This should not panic and should use /openapi as default
        let _final_router = router_with_routes.into_router();
    }
}

#[cfg(test)]
mod rustdoc_parsing_tests {
    use super::*;
    
    #[test]
    fn test_parse_parameters_from_rustdoc() {
        let router = api_router!("Test", "1.0");
        let params = r#"["id (path): The unique user identifier", "include_deleted (query): Include soft-deleted records"]"#;
        let result = router.parse_parameters_to_openapi(params);
        assert_eq!(result[0]["name"], "id");
        assert_eq!(result[0]["in"], "path");
        assert_eq!(result[1]["name"], "include_deleted");
        assert_eq!(result[1]["in"], "query");
    }

    #[test]
    fn test_parse_request_body_from_rustdoc() {
        let mut router = api_router!("Test", "1.0");
        let body = r#"["Type: CreateUserRequest", "Content-Type: application/json", "User information for account creation", "- name (string): The user's full name", "- email (string): Valid email address", "- age (number): User's age in years"]"#;
        let result = router.parse_request_body_to_openapi(body);
        // CreateUserRequest is registered via StonehmSchema in the test fixtures,
        // so the parser should emit a $ref instead of inlining.
        assert_eq!(result["required"], true);
        assert_eq!(
            result["content"]["application/json"]["schema"]["$ref"],
            "#/components/schemas/CreateUserRequest"
        );
    }

    #[test]
    fn test_parse_responses_with_status_codes() {
        let mut router = api_router!("Test", "1.0");
        let responses = r#"["200: User successfully created", "201: Resource created", "400: Invalid request data", "500: Internal server error"]"#;
        let result = router.parse_responses_to_openapi(responses);
        assert!(result["200"].is_object());
        assert!(result["201"].is_object());
        assert!(result["400"].is_object());
        assert!(result["500"].is_object());
    }

    #[test]
    fn test_malformed_parameter_handling() {
        let router = api_router!("Test", "1.0");
        let params = r#"["invalid param without type", "id: missing location", "valid (query): This one is good"]"#;
        let result = router.parse_parameters_to_openapi(params);
        // Find the parsed entry whose name is "valid".
        let arr = result.as_array().unwrap();
        assert!(arr.iter().any(|p| p["name"] == "valid" && p["in"] == "query"));
    }
}

#[cfg(test)]
mod schema_generation_tests {
    
    
    // Mock schema registration for testing
    fn mock_schema_registration(type_name: &str, schema_json: &str) {
        // In real usage, this would be done by the StonehmSchema derive macro
        // For testing, we just verify the structure
        assert!(!type_name.is_empty());
        assert!(schema_json.contains("type"));
    }
    
    #[test]
    fn test_simple_struct_schema() {
        let schema_json = r#"{"type":"object","properties":{"id":{"type":"integer"},"name":{"type":"string"}},"required":["id","name"]}"#;
        mock_schema_registration("UserResponse", schema_json);
        
        assert!(schema_json.contains(r#""type":"object""#));
        assert!(schema_json.contains(r#""properties""#));
        assert!(schema_json.contains(r#""required""#));
    }
    
    #[test]
    fn test_optional_fields_schema() {
        let schema_json = r#"{"type":"object","properties":{"id":{"type":"integer"},"nickname":{"type":"string"}},"required":["id"]}"#;
        mock_schema_registration("ProfileResponse", schema_json);
        
        // nickname is optional, so only id should be required
        assert!(schema_json.contains(r#""required":["id"]"#));
        assert!(!schema_json.contains("nickname") || !schema_json.contains(r#""required":["id","nickname"]"#));
    }
    
    #[test]
    fn test_nested_struct_schema() {
        let schema_json = r#"{"type":"object","properties":{"user":{"type":"object"},"preferences":{"type":"object"}},"required":["user","preferences"]}"#;
        mock_schema_registration("UserWithPreferences", schema_json);
        
        assert!(schema_json.contains(r#""user":{"type":"object"}"#));
        assert!(schema_json.contains(r#""preferences":{"type":"object"}"#));
    }
    
    #[test]
    fn test_array_field_schema() {
        let schema_json = r#"{"type":"object","properties":{"items":{"type":"array"}},"required":["items"]}"#;
        mock_schema_registration("ItemList", schema_json);
        
        assert!(schema_json.contains(r#""type":"array""#));
    }
    
    #[test]
    fn test_numeric_types_schema() {
        let schema_json = r#"{"type":"object","properties":{"age":{"type":"integer"},"height":{"type":"number"},"weight":{"type":"number"}},"required":["age","height","weight"]}"#;
        mock_schema_registration("PersonMetrics", schema_json);
        
        // Integer types
        assert!(schema_json.contains(r#""age":{"type":"integer"}"#));
        // Float types
        assert!(schema_json.contains(r#""height":{"type":"number"}"#));
    }
    
    #[test]
    fn test_boolean_field_schema() {
        let schema_json = r#"{"type":"object","properties":{"active":{"type":"boolean"},"verified":{"type":"boolean"}},"required":["active","verified"]}"#;
        mock_schema_registration("UserStatus", schema_json);

        assert!(schema_json.contains(r#""type":"boolean""#));
    }
}

/// Tests that exercise `#[derive(StonehmSchema)]` against real fixture
/// types. The earlier `schema_generation_tests` module asserts on
/// hardcoded JSON strings that the derive macro is supposed to emit, but
/// never actually runs the macro — so it can't catch macro regressions.
/// These tests do.
///
/// Note: imports are written with absolute `::serde` / `::serde_json`
/// paths because `crate::serde` is a stub trait module that would
/// otherwise shadow the real `serde` crate.
#[cfg(test)]
mod derive_macro_tests {
    use crate::StonehmSchema;
    use ::serde::{Deserialize, Serialize};
    use ::serde_json::Value;

    // ─── Option<T> coverage ──────────────────────────────────────────

    #[derive(Serialize, Deserialize, StonehmSchema)]
    #[allow(dead_code)]
    struct WithOptions {
        required_id: u64,
        optional_id: Option<u64>,
        optional_flag: Option<bool>,
        optional_name: Option<String>,
    }

    #[test]
    fn option_inner_type_is_preserved() {
        let raw = WithOptions::schema();
        let v: Value = serde_json::from_str(&raw).expect("schema is valid JSON");
        assert_eq!(v["properties"]["required_id"]["type"], "integer");
        // Option<T> should map to T's schema, not "string".
        assert_eq!(v["properties"]["optional_id"]["type"], "integer");
        assert_eq!(v["properties"]["optional_flag"]["type"], "boolean");
        assert_eq!(v["properties"]["optional_name"]["type"], "string");
    }

    #[test]
    fn option_fields_are_not_required() {
        let raw = WithOptions::schema();
        let v: Value = serde_json::from_str(&raw).expect("schema is valid JSON");
        let required: Vec<&str> = v["required"].as_array().unwrap()
            .iter().filter_map(|x| x.as_str()).collect();
        assert!(required.contains(&"required_id"));
        assert!(!required.contains(&"optional_id"));
        assert!(!required.contains(&"optional_flag"));
        assert!(!required.contains(&"optional_name"));
    }

    // ─── Vec<T> coverage ─────────────────────────────────────────────

    #[derive(Serialize, Deserialize, StonehmSchema)]
    #[allow(dead_code)]
    struct WithVec {
        ids: Vec<u64>,
        names: Vec<String>,
    }

    #[test]
    fn vec_inner_type_is_preserved() {
        let raw = WithVec::schema();
        let v: Value = serde_json::from_str(&raw).expect("schema is valid JSON");
        assert_eq!(v["properties"]["ids"]["type"], "array");
        assert_eq!(v["properties"]["ids"]["items"]["type"], "integer");
        assert_eq!(v["properties"]["names"]["type"], "array");
        assert_eq!(v["properties"]["names"]["items"]["type"], "string");
    }

    // ─── Fixed-size arrays + references ──────────────────────────────

    #[derive(Serialize, Deserialize, StonehmSchema)]
    #[allow(dead_code)]
    struct WithArrays {
        position: [f32; 3],
        rotation: [f32; 4],
        bytes: [u8; 32],
    }

    #[test]
    fn fixed_size_arrays_carry_length() {
        let raw = WithArrays::schema();
        let v: Value = serde_json::from_str(&raw).expect("schema is valid JSON");
        assert_eq!(v["properties"]["position"]["type"], "array");
        assert_eq!(v["properties"]["position"]["items"]["type"], "number");
        // The size is a hard constraint on the wire, so it should appear
        // as both minItems and maxItems.
        assert_eq!(v["properties"]["position"]["minItems"], 3);
        assert_eq!(v["properties"]["position"]["maxItems"], 3);
        assert_eq!(v["properties"]["rotation"]["maxItems"], 4);
        assert_eq!(v["properties"]["bytes"]["items"]["type"], "integer");
        assert_eq!(v["properties"]["bytes"]["maxItems"], 32);
    }

    #[derive(Serialize, StonehmSchema)]
    #[allow(dead_code)]
    struct WithRef {
        name: &'static str,
    }

    #[test]
    fn reference_types_strip_to_inner() {
        let raw = WithRef::schema();
        let v: Value = serde_json::from_str(&raw).expect("schema is valid JSON");
        // `&'static str` should be a plain string schema, not a $ref.
        assert_eq!(v["properties"]["name"]["type"], "string");
        assert!(v["properties"]["name"].get("$ref").is_none());
    }

    // ─── Well-known external types (Uuid, DateTime, serde_json::Value, etc.) ───

    /// Mock types named the same as the standard ones so we can derive
    /// `StonehmSchema` without taking a uuid/chrono dep on the test crate.
    /// The derive only sees the ident, not the fully-qualified path —
    /// so the special-case ident match in `type_to_schema` fires for
    /// any type called `Uuid`, regardless of where it actually came
    /// from. This is exactly the trade-off documented in the macro:
    /// consumers that want a *different* `Uuid` would derive
    /// `StonehmSchema` on their own type.
    mod stand_ins {
        // These types stand in for the real `uuid::Uuid`,
        // `chrono::DateTime<Utc>`, etc. — same idents, no actual deps.
        #[allow(dead_code)]
        pub struct Uuid;
        #[allow(dead_code)]
        pub struct DateTime<Tz>(pub std::marker::PhantomData<Tz>);
        #[allow(dead_code)]
        pub struct Utc;
    }

    // Note: no `Serialize` derive — these stand-ins don't implement it
    // and we don't actually serialize them in tests. StonehmSchema only
    // needs the type as a path segment.
    #[derive(StonehmSchema)]
    #[allow(dead_code)]
    struct WithExternals {
        id: stand_ins::Uuid,
        ts: stand_ins::DateTime<stand_ins::Utc>,
        // serde_json::Value is special-cased to `{}` (any JSON).
        any_json: ::serde_json::Value,
        opt_id: Option<stand_ins::Uuid>,
    }

    #[test]
    fn uuid_field_carries_format_uuid() {
        let raw = WithExternals::schema();
        let v: Value = serde_json::from_str(&raw).expect("schema is valid JSON");
        assert_eq!(v["properties"]["id"]["type"], "string");
        assert_eq!(v["properties"]["id"]["format"], "uuid");
        // Option<Uuid> should keep the format and just drop required-ness.
        assert_eq!(v["properties"]["opt_id"]["type"], "string");
        assert_eq!(v["properties"]["opt_id"]["format"], "uuid");
    }

    #[test]
    fn datetime_field_carries_format_date_time() {
        let raw = WithExternals::schema();
        let v: Value = serde_json::from_str(&raw).expect("schema is valid JSON");
        assert_eq!(v["properties"]["ts"]["type"], "string");
        assert_eq!(v["properties"]["ts"]["format"], "date-time");
    }

    #[test]
    fn serde_json_value_field_is_open_schema() {
        let raw = WithExternals::schema();
        let v: Value = serde_json::from_str(&raw).expect("schema is valid JSON");
        // serde_json::Value is "any JSON" — empty schema in OpenAPI.
        let any = &v["properties"]["any_json"];
        assert!(any.is_object());
        assert_eq!(any.as_object().unwrap().len(), 0,
            "expected empty schema (any-JSON), got: {any}");
    }

    // ─── Map types ──────────────────────────────────────────────────

    use std::collections::HashMap;

    #[derive(StonehmSchema)]
    #[allow(dead_code)]
    struct WithMap {
        flags: HashMap<String, bool>,
    }

    #[test]
    fn hashmap_emits_additional_properties() {
        let raw = WithMap::schema();
        let v: Value = serde_json::from_str(&raw).expect("schema is valid JSON");
        assert_eq!(v["properties"]["flags"]["type"], "object");
        assert_eq!(v["properties"]["flags"]["additionalProperties"]["type"], "boolean");
    }

    // ─── Tagged enum (`#[serde(tag = "type")]`) ─────────────────────

    #[derive(Serialize, Deserialize, StonehmSchema)]
    #[serde(tag = "type", rename_all = "snake_case")]
    #[allow(dead_code)]
    enum DemoCommand {
        AddBox { width: f64, height: f64 },
        Select { id: u64 },
        DeselectAll,
        WithOpt { value: Option<u64> },
    }

    #[test]
    fn tagged_enum_emits_oneof_with_discriminator() {
        let raw = DemoCommand::schema();
        let v: Value = serde_json::from_str(&raw).expect("schema is valid JSON");
        assert!(v["oneOf"].is_array(), "expected oneOf, got: {raw}");
        assert_eq!(v["discriminator"]["propertyName"], "type");
    }

    #[test]
    fn tagged_enum_variant_names_are_snake_cased() {
        let raw = DemoCommand::schema();
        let v: Value = serde_json::from_str(&raw).expect("schema is valid JSON");
        let variants = v["oneOf"].as_array().unwrap();
        let consts: Vec<String> = variants.iter()
            .map(|var| var["properties"]["type"]["const"].as_str().unwrap().to_string())
            .collect();
        assert!(consts.contains(&"add_box".to_string()), "consts: {consts:?}");
        assert!(consts.contains(&"select".to_string()));
        assert!(consts.contains(&"deselect_all".to_string()));
        assert!(consts.contains(&"with_opt".to_string()));
    }

    #[test]
    fn tagged_enum_variant_carries_field_schemas() {
        let raw = DemoCommand::schema();
        let v: Value = serde_json::from_str(&raw).expect("schema is valid JSON");
        // Find the AddBox variant.
        let add_box = v["oneOf"].as_array().unwrap().iter().find(|var| {
            var["properties"]["type"]["const"] == "add_box"
        }).expect("add_box variant present");
        assert_eq!(add_box["properties"]["width"]["type"], "number");
        assert_eq!(add_box["properties"]["height"]["type"], "number");
        // The discriminator field is required, plus all non-Option fields.
        let req: Vec<&str> = add_box["required"].as_array().unwrap()
            .iter().filter_map(|x| x.as_str()).collect();
        assert!(req.contains(&"type"));
        assert!(req.contains(&"width"));
        assert!(req.contains(&"height"));
    }

    #[test]
    fn tagged_enum_unit_variant_is_just_the_discriminator() {
        let raw = DemoCommand::schema();
        let v: Value = serde_json::from_str(&raw).expect("schema is valid JSON");
        let deselect = v["oneOf"].as_array().unwrap().iter().find(|var| {
            var["properties"]["type"]["const"] == "deselect_all"
        }).expect("deselect_all variant present");
        // Only the discriminator field.
        let props = deselect["properties"].as_object().unwrap();
        assert_eq!(props.len(), 1);
        assert!(props.contains_key("type"));
    }

    #[test]
    fn tagged_enum_variant_option_field_is_not_required() {
        let raw = DemoCommand::schema();
        let v: Value = serde_json::from_str(&raw).expect("schema is valid JSON");
        let with_opt = v["oneOf"].as_array().unwrap().iter().find(|var| {
            var["properties"]["type"]["const"] == "with_opt"
        }).expect("with_opt variant present");
        // The Option<u64> should be typed as integer (the inner type)
        // and absent from `required`.
        assert_eq!(with_opt["properties"]["value"]["type"], "integer");
        let req: Vec<&str> = with_opt["required"].as_array().unwrap()
            .iter().filter_map(|x| x.as_str()).collect();
        assert!(req.contains(&"type"));
        assert!(!req.contains(&"value"));
    }

    // ─── Per-variant rename ─────────────────────────────────────────

    #[derive(Serialize, Deserialize, StonehmSchema)]
    #[serde(tag = "kind")]
    #[allow(dead_code)]
    enum WithRename {
        #[serde(rename = "ai_generate_3d")]
        AiGenerate3d { name: String },
        Plain,
    }

    #[test]
    fn variant_level_rename_is_honoured() {
        let raw = WithRename::schema();
        let v: Value = serde_json::from_str(&raw).expect("schema is valid JSON");
        assert_eq!(v["discriminator"]["propertyName"], "kind");
        let consts: Vec<String> = v["oneOf"].as_array().unwrap().iter()
            .map(|var| var["properties"]["kind"]["const"].as_str().unwrap().to_string())
            .collect();
        // The renamed variant uses the override; the unrenamed uses
        // the bare ident verbatim (no rename_all here).
        assert!(consts.contains(&"ai_generate_3d".to_string()));
        assert!(consts.contains(&"Plain".to_string()));
    }

    // ─── Custom-type field becomes $ref ─────────────────────────────

    #[derive(Serialize, Deserialize, StonehmSchema)]
    #[allow(dead_code)]
    struct Inner { value: u64 }

    #[derive(Serialize, Deserialize, StonehmSchema)]
    #[allow(dead_code)]
    struct Outer { inner: Inner, optional_inner: Option<Inner> }

    #[test]
    fn custom_type_field_is_a_ref() {
        let raw = Outer::schema();
        let v: Value = serde_json::from_str(&raw).expect("schema is valid JSON");
        assert_eq!(
            v["properties"]["inner"]["$ref"],
            "#/components/schemas/Inner"
        );
        assert_eq!(
            v["properties"]["optional_inner"]["$ref"],
            "#/components/schemas/Inner"
        );
    }
}