use fastly::{http::Method, Error, Request};
use log::debug;
use url::Url;

mod types;

pub use types::*;

pub struct Raven {
    dsn: Url,
    backend: String,
}

impl Raven {
    pub fn from_dsn_and_backend(dsn: impl Into<Url>, backend: impl Into<String>) -> Self {
        Raven {
            dsn: dsn.into(),
            backend: backend.into(),
        }
    }

    pub fn report_error(
        &self,
        error: impl std::error::Error,
        request: &Request,
    ) -> Result<(), Error> {
        let event_payload = EventPayload {
            request: Some(request.into()),
            ..error.into()
        };

        let auth_string = format!(
            "Sentry sentry_version=7,sentry_key={},sentry_secret={},sentry_client={}/{}",
            self.dsn.username(),
            self.dsn.password().unwrap_or(""),
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        );

        let req = Request::new(
            Method::POST,
            format!(
                "{}://{}/api{}/store/",
                self.dsn.scheme(),
                self.dsn.host_str().unwrap(),
                self.dsn.path()
            ),
        )
        .with_header("X-Sentry-Auth", auth_string)
        .with_body_json(&event_payload)?;

        debug!("-> Submitting report to Sentry");

        let resp = req.send(self.backend.clone())?;

        debug!("-> {} from Sentry", resp.get_status());

        Ok(())
    }
}
