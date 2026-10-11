use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ApiRequest {
    pub name: String,
    pub method: HttpMethod,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

impl Default for ApiRequest {
    fn default() -> Self {
        Self {
            name: "New Request".to_string(),
            method: HttpMethod::Get,
            url: "https://httpbin.org/get".to_string(),
            headers: HashMap::new(),
            body: None,
        }
    }
}
