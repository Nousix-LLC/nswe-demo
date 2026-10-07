//! The single UI error type every failure funnels through.
//!
//! `AppError` is the one error the views branch on. Both a [`ferric_http::Error`] (transport /
//! network / non-2xx status) and a decodable server `chirp_types::ApiError` map into it through
//! `AppError::from_http`, so every view renders a coherent loading / error state instead of each
//! module inventing its own error shape. This module is owned by `SUBTASK_scaffold` and is a real
//! implementation (not a stub): it is the seam the feature spokes map their failures onto.
//!
//! Nothing here panics: `from_http` is total over `ferric_http::Error` (including its
//! `#[non_exhaustive]` growth), and a non-decodable body degrades to `AppError::Transport` rather
//! than unwrapping.

use chirp_types::prelude::{ApiError, ErrorCode};
use ferric_http::Error as HttpError;

/// The single error type the UI branches on. Every [`ferric_http::Error`] and every decodable
/// server [`ApiError`] funnels through here so views render coherent states.
#[derive(Debug, Clone)]
pub enum AppError {
    /// A non-2xx response whose JSON body decoded to the server's wire error contract.
    Api(ApiError),
    /// `fetch` failed before a response arrived (offline/DNS/CORS). Human-readable text.
    Network(String),
    /// Any other transport/encoding failure (builder/body/(de)serialize, or a non-2xx whose body
    /// was not a decodable [`ApiError`]). Human-readable text.
    Transport(String),
}

impl AppError {
    /// Map a [`ferric_http::Error`] into an `AppError`, decoding a `chirp_types::ApiError` out
    /// of a non-2xx `Error::Status { body }` when the body is a valid error document.
    ///
    /// * A [`ferric_http::Error::Network`] becomes [`AppError::Network`].
    /// * A non-2xx [`ferric_http::Error::Status`] whose body decodes to an [`ApiError`] becomes
    ///   [`AppError::Api`]; otherwise it becomes `AppError::Transport` carrying a status summary.
    /// * Every other variant (builder / (de)serialize / body read, and any future `#[non_exhaustive]`
    ///   variant) becomes `AppError::Transport` carrying its display text.
    #[must_use]
    pub fn from_http(err: HttpError) -> Self {
        match err {
            HttpError::Network(message) => AppError::Network(message),
            HttpError::Status {
                status,
                status_text,
                url,
                body,
            } => {
                if let Some(body) = &body {
                    if let Ok(api) = serde_json::from_str::<ApiError>(body) {
                        return AppError::Api(api);
                    }
                }
                let summary = if status_text.is_empty() {
                    format!("HTTP status {status} for {url}")
                } else {
                    format!("HTTP status {status} {status_text} for {url}")
                };
                AppError::Transport(summary)
            }
            other => AppError::Transport(other.to_string()),
        }
    }

    /// A short, user-facing message suitable for an error banner.
    #[must_use]
    pub fn user_message(&self) -> String {
        match self {
            AppError::Api(api) => api.message.clone(),
            AppError::Network(message) => format!("Network error: {message}"),
            AppError::Transport(message) => format!("Something went wrong: {message}"),
        }
    }

    /// The machine-readable category when this is a decoded server error, else `None`.
    #[must_use]
    pub fn code(&self) -> Option<ErrorCode> {
        match self {
            AppError::Api(api) => Some(api.code),
            AppError::Network(_) | AppError::Transport(_) => None,
        }
    }
}

/// Convenience conversion so `?` on a `ferric_http` call yields an `AppError` directly.
impl From<HttpError> for AppError {
    fn from(err: HttpError) -> Self {
        AppError::from_http(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chirp_types::prelude::ApiError;

    #[test]
    fn network_error_maps_to_network_variant() {
        let err = AppError::from_http(HttpError::Network("offline".to_string()));
        assert!(matches!(err, AppError::Network(ref msg) if msg == "offline"));
        assert_eq!(err.code(), None);
    }

    #[test]
    fn status_with_decodable_body_maps_to_api_variant() {
        let body = serde_json::to_string(&ApiError::new(ErrorCode::NotFound, "no such user"))
            .expect("ApiError serializes");
        let err = AppError::from_http(HttpError::Status {
            status: 404,
            status_text: "Not Found".to_string(),
            url: "/api/users/9".to_string(),
            body: Some(body),
        });
        assert_eq!(err.code(), Some(ErrorCode::NotFound));
        assert_eq!(err.user_message(), "no such user");
    }

    #[test]
    fn status_with_undecodable_body_degrades_to_transport() {
        let err = AppError::from_http(HttpError::Status {
            status: 500,
            status_text: String::new(),
            url: "/api/timeline".to_string(),
            body: Some("<html>oops</html>".to_string()),
        });
        assert!(matches!(err, AppError::Transport(_)));
        assert_eq!(err.code(), None);
    }

    #[test]
    fn builder_error_degrades_to_transport() {
        let err: AppError = HttpError::Builder("bad header".to_string()).into();
        assert!(matches!(err, AppError::Transport(_)));
    }
}
