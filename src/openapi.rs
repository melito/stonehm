// Simple OpenAPI implementation without external dependencies
use std::collections::HashMap;

// Simple JSON-like value type using only standard library
#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

impl JsonValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            JsonValue::String(s) => Some(s),
            _ => None,
        }
    }
    
    pub fn as_object(&self) -> Option<&HashMap<String, JsonValue>> {
        match self {
            JsonValue::Object(obj) => Some(obj),
            _ => None,
        }
    }
}

// Simple OpenAPI types
#[derive(Debug, Clone)]
pub struct OpenAPI {
    pub openapi: String,
    pub info: Info,
    pub paths: HashMap<String, PathItem>,
    pub components: Option<Components>,
}

impl OpenAPI {
    pub fn new(title: &str, version: &str) -> Self {
        Self {
            openapi: "3.0.0".to_string(),
            info: Info {
                title: title.to_string(),
                version: version.to_string(),
                description: None,
            },
            paths: HashMap::new(),
            components: None,
        }
    }
    
    pub fn to_json(&self) -> String {
        format!(
            r#"{{
  "openapi": "{}",
  "info": {{
    "title": "{}",
    "version": "{}"
  }},
  "paths": {{}},
  "components": {{
    "schemas": {{}}
  }}
}}"#,
            self.openapi, self.info.title, self.info.version
        )
    }
    
    pub fn to_yaml(&self) -> String {
        format!(
            r#"openapi: {}
info:
  title: {}
  version: {}
paths: {{}}
components:
  schemas: {{}}
"#,
            self.openapi, self.info.title, self.info.version
        )
    }
}

#[derive(Debug, Clone)]
pub struct Info {
    pub title: String,
    pub version: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PathItem {
    pub get: Option<Operation>,
    pub post: Option<Operation>,
    pub put: Option<Operation>,
    pub delete: Option<Operation>,
    pub patch: Option<Operation>,
}

#[derive(Debug, Clone)]
pub struct Operation {
    pub summary: Option<String>,
    pub description: Option<String>,
    pub parameters: Vec<Parameter>,
    pub request_body: Option<RequestBody>,
    pub responses: HashMap<String, Response>,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub location: String,
    pub description: Option<String>,
    pub required: bool,
    pub schema: Schema,
}

#[derive(Debug, Clone)]
pub struct RequestBody {
    pub description: Option<String>,
    pub content: HashMap<String, MediaType>,
    pub required: bool,
}

#[derive(Debug, Clone)]
pub struct Response {
    pub description: String,
    pub content: Option<HashMap<String, MediaType>>,
}

#[derive(Debug, Clone)]
pub struct MediaType {
    pub schema: Option<Schema>,
}

#[derive(Debug, Clone)]
pub struct Components {
    pub schemas: HashMap<String, Schema>,
}

#[derive(Debug, Clone)]
pub struct Schema {
    pub schema_type: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub properties: Option<HashMap<String, Schema>>,
    pub required: Option<Vec<String>>,
}

impl Default for Schema {
    fn default() -> Self {
        Self {
            schema_type: Some("object".to_string()),
            title: None,
            description: None,
            properties: None,
            required: None,
        }
    }
}