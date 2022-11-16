use std::collections::HashMap;

use fastly::Request;
use serde::Serialize;
use time::OffsetDateTime;


#[derive(Serialize)]
pub struct EventPayload {
    pub event_id: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub timestamp: OffsetDateTime,
    pub platform: Platform,
    pub level: Level,
    pub transaction: Option<String>,
    pub server_name: Option<String>,
    pub release: Option<String>,
    pub environment: Option<String>,
    pub exception: Vec<Exception>,
    pub request: Option<RequestMetadata>,
}

#[derive(Serialize)]
pub struct Exception {
    #[serde(rename = "type")]
    name: String,
    value: String,
}

#[derive(Serialize)]
pub struct RequestMetadata {
    method: String,
    url: String,
    headers: HashMap<String, String>,
}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    C,
    Native,
    Other,
}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Fatal,
    Error,
    Warning,
    Info,
    Debug,
}

impl Default for EventPayload {
    fn default() -> Self {
        EventPayload {
            event_id: uuid::Uuid::new_v4().to_string(),
            event_type: "event".to_string(),
            timestamp: OffsetDateTime::now_utc(),
            platform: Platform::Other,
            level: Level::Fatal,
            transaction: None,
            server_name: Some(std::env::var("FASTLY_HOSTNAME").unwrap()),
            release: Some(std::env::var("FASTLY_SERVICE_VERSION").unwrap()),
            environment: Some(std::env::var("FASTLY_SERVICE_ID").unwrap()),
            exception: Vec::new(),
            request: None,
        }
    }
}

impl<T: std::error::Error> From<T> for EventPayload {
    fn from(error: T) -> Self {
        EventPayload {
            exception: vec![Exception {
                name: format!("{:?}", error)
                    .chars()
                    .take_while(|&ch| ch != '(')
                    .collect::<String>(),
                value: error.to_string(),
            }],
            ..Default::default()
        }
    }
}

impl From<&Request> for RequestMetadata {
    fn from(request: &Request) -> Self {
        let mut headers = HashMap::new();

        request.get_header_names().for_each(|k| {
            headers.insert(
                k.to_string(),
                request.get_header(k).unwrap().to_str().unwrap().to_string(),
            );
        });

        RequestMetadata {
            method: request.get_method().to_string(),
            url: request.get_url().to_string(),
            headers,
        }
    }
}
