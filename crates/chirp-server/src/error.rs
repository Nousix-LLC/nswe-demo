//! The server-side error type and its mapping onto the `chirp-types` wire error contract.
//!
//! [`ServerError`] is the single error type every [`ChirpRepository`](crate::repository::ChirpRepository)
//! operation (and, later, every service/handler) returns. It is a typed enum — not a stringly-typed
//! or opaque error — so callers can branch on the failure mode, and so the transport layer can map
//! each variant to the correct HTTP status deterministically.
//!
//! # Ownership of the error seam
//!
//! This module owns the **enum definition** and its translation into the wire types
//! [`ApiError`] / [`ErrorCode`] — both of
//! which are *imported from `chirp-types`, never re-declared*. It deliberately does **not** own the
//! `axum::response::IntoResponse` implementation or any HTTP-status mapping: that is the api-layer
//! spoke's responsibility (it depends on `axum`; this layer stays HTTP-free). The bridge this module
//! provides for that spoke is [`ServerError::code`] (the machine-readable category) and
//! [`ServerError::into_api_error`] (the ready-to-serialize body); the api-layer maps
//! [`ErrorCode`] → `axum::http::StatusCode` in its `IntoResponse`.
//!
//! # The error-mapping contract
//!
//! The variant ↔ [`ErrorCode`] ↔ HTTP-status correspondence is frozen
//! in the subtree contract (`contracts/_MANIFEST.yaml`, `error_mapping`). For reference, the
//! intended end-to-end mapping (HTTP status applied by the api-layer) is:
//!
//! | [`ServerError`] variant | [`ErrorCode`] | HTTP status |
//! |-------------------------|--------------------------------------------|-------------|
//! | [`Validation`](ServerError::Validation)   | `VALIDATION_ERROR` | 400 |
//! | [`Unauthorized`](ServerError::Unauthorized) | `UNAUTHORIZED`    | 401 |
//! | [`Forbidden`](ServerError::Forbidden)      | `FORBIDDEN`        | 403 |
//! | [`NotFound`](ServerError::NotFound)        | `NOT_FOUND`        | 404 |
//! | [`Conflict`](ServerError::Conflict)        | `CONFLICT`         | 409 |
//! | [`Internal`](ServerError::Internal)        | `INTERNAL`         | 500 |

use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use chirp_types::api::{ApiError, ErrorCode, FieldError};
use chirp_types::domain::ValidationError;

/// The one error type returned across the chirp server: repository, service, and (via the
/// api-layer's `IntoResponse`) the transport edge.
///
/// Each variant corresponds to exactly one [`ErrorCode`] category and,
/// downstream, one HTTP status (see the [module docs](self)). The enum is `#[non_exhaustive]` so new
/// failure categories can be added without breaking downstream matches, and it carries a
/// human-readable message on every variant for logging and for the wire
/// [`message`](chirp_types::api::ApiError::message) field.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ServerError {
    /// Input or DTO validation failed (HTTP 400 / `VALIDATION_ERROR`).
    ///
    /// Carries the originating [`ValidationError`] as its source so the field-level detail survives
    /// into [`ApiError::details`](chirp_types::api::ApiError::details). Produced automatically from a
    /// domain [`ValidationError`] via `?` (see the [`From`] impl).
    #[error("validation failed: {0}")]
    Validation(#[from] ValidationError),

    /// A request body could not be deserialized/validated at the transport boundary
    /// (HTTP 400 / `VALIDATION_ERROR`).
    ///
    /// Raised by the [`ValidatedJson`](crate::api::ValidatedJson) extractor when axum's JSON
    /// decoding rejects the body — including a `chirp-types` value object's
    /// `#[serde(try_from = "String")]` [`ValidationError`] surfacing as a serde data error, as well
    /// as malformed JSON or a missing/incorrect content type. These all represent invalid *input*,
    /// so they map to the same `VALIDATION_ERROR`/400 as a domain [`Validation`](ServerError::Validation)
    /// failure, ensuring the body is rendered as the contract's [`ApiError`] rather than axum's
    /// default plain-text `422`. Carries the rejection's human-readable message. See the
    /// [`From<JsonRejection>`](#impl-From<JsonRejection>-for-ServerError) impl.
    #[error("{0}")]
    InvalidBody(String),

    /// Authentication is required but missing or invalid (HTTP 401 / `UNAUTHORIZED`).
    ///
    /// Raised when a request presents no session token, or a token that resolves to no user.
    #[error("unauthorized: {0}")]
    Unauthorized(String),

    /// The caller is authenticated but not permitted to perform the action (HTTP 403 / `FORBIDDEN`).
    ///
    /// Example: attempting to follow oneself, which the follow invariant forbids.
    #[error("forbidden: {0}")]
    Forbidden(String),

    /// An addressed resource (user or chirp) does not exist (HTTP 404 / `NOT_FOUND`).
    #[error("not found: {0}")]
    NotFound(String),

    /// The request conflicts with current state (HTTP 409 / `CONFLICT`).
    ///
    /// Example: registering a username that is already taken.
    #[error("conflict: {0}")]
    Conflict(String),

    /// An unexpected server-side failure (HTTP 500 / `INTERNAL`).
    ///
    /// Reserved for conditions that are not the caller's fault — for example a poisoned lock. The
    /// human-readable message is for logs; the wire response deliberately stays generic.
    #[error("internal error: {0}")]
    Internal(String),
}

impl ServerError {
    /// Returns the machine-readable [`ErrorCode`] category for this
    /// error — the stable value a client branches on and the key the api-layer maps to an HTTP
    /// status.
    #[must_use]
    pub fn code(&self) -> ErrorCode {
        match self {
            ServerError::Validation(_) | ServerError::InvalidBody(_) => ErrorCode::ValidationError,
            ServerError::Unauthorized(_) => ErrorCode::Unauthorized,
            ServerError::Forbidden(_) => ErrorCode::Forbidden,
            ServerError::NotFound(_) => ErrorCode::NotFound,
            ServerError::Conflict(_) => ErrorCode::Conflict,
            ServerError::Internal(_) => ErrorCode::Internal,
        }
    }

    /// Builds the wire-ready [`ApiError`] for this error.
    ///
    /// The [`code`](ServerError::code) sets the category and the [`Display`](std::fmt::Display)
    /// text becomes the human-readable `message`. The two validation-category variants carry field
    /// details: for a [`Validation`](ServerError::Validation) error the originating
    /// [`ValidationError`]'s field is lifted into a single
    /// [`FieldError`]; for an [`InvalidBody`](ServerError::InvalidBody)
    /// error (a transport-boundary deserialization rejection, where the structured field is not
    /// recoverable from axum's `JsonRejection`) a single generic `body` [`FieldError`] carries the
    /// rejection message. Other variants carry no field details. HTTP status is intentionally *not*
    /// set here — the api-layer applies it when turning this into a response.
    #[must_use]
    pub fn into_api_error(&self) -> ApiError {
        let error = ApiError::new(self.code(), self.to_string());
        match self {
            ServerError::Validation(source) => error.with_details(vec![FieldError {
                field: validation_field(source).to_owned(),
                message: source.to_string(),
            }]),
            ServerError::InvalidBody(detail) => error.with_details(vec![FieldError {
                field: "body".to_owned(),
                message: detail.clone(),
            }]),
            _ => error,
        }
    }
}

/// Converts an axum [`JsonRejection`] into a [`ServerError::InvalidBody`].
///
/// This is the seam the [`ValidatedJson`](crate::api::ValidatedJson) extractor relies on: a failed
/// JSON body extraction (a `chirp-types` `#[serde(try_from)]` [`ValidationError`] surfacing as a
/// serde data error, malformed JSON, or a missing/incorrect content type) becomes a
/// `VALIDATION_ERROR`/400 that renders as the contract's [`ApiError`], instead of axum's default
/// plain-text `422`. The rejection's [`body_text`](JsonRejection::body_text) is preserved as the
/// human-readable message.
impl From<JsonRejection> for ServerError {
    fn from(rejection: JsonRejection) -> Self {
        ServerError::InvalidBody(rejection.body_text())
    }
}

/// Extracts the offending field name from a domain [`ValidationError`] so it can surface as a
/// wire-level [`FieldError`].
fn validation_field(error: &ValidationError) -> &'static str {
    match error {
        ValidationError::Empty { field }
        | ValidationError::TooLong { field, .. }
        | ValidationError::InvalidCharacter { field, .. } => field,
    }
}

/// Renders a [`ServerError`] as an HTTP response — the transport edge of the error seam.
///
/// The variant's [`code`](ServerError::code) selects the HTTP status fixed by the subtree
/// contract's `error_mapping` table, and [`into_api_error`](ServerError::into_api_error) supplies
/// the JSON body, so every failure surfaces on the wire as an [`ApiError`] with the matching
/// [`ErrorCode`]:
///
/// | [`code`](ServerError::code) | HTTP status |
/// |-----------------------------|-------------|
/// | `VALIDATION_ERROR` | `400 Bad Request` |
/// | `UNAUTHORIZED`     | `401 Unauthorized` |
/// | `FORBIDDEN`        | `403 Forbidden` |
/// | `NOT_FOUND`        | `404 Not Found` |
/// | `CONFLICT`         | `409 Conflict` |
/// | `INTERNAL`         | `500 Internal Server Error` |
///
/// An [`Internal`](ServerError::Internal) error logs its message at `error` level for the
/// operator but returns a deliberately generic body, so server-side detail never leaks to the
/// client.
impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let status = match self.code() {
            ErrorCode::ValidationError => StatusCode::BAD_REQUEST,
            ErrorCode::Unauthorized => StatusCode::UNAUTHORIZED,
            ErrorCode::Forbidden => StatusCode::FORBIDDEN,
            ErrorCode::NotFound => StatusCode::NOT_FOUND,
            ErrorCode::Conflict => StatusCode::CONFLICT,
            ErrorCode::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = match &self {
            ServerError::Internal(message) => {
                tracing::error!(error = %message, "internal server error");
                ApiError::new(ErrorCode::Internal, "an internal server error occurred")
            }
            other => other.into_api_error(),
        };

        (status, Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_variant_maps_to_its_error_code() {
        assert_eq!(
            ServerError::Unauthorized("no token".into()).code(),
            ErrorCode::Unauthorized
        );
        assert_eq!(
            ServerError::Forbidden("self-follow".into()).code(),
            ErrorCode::Forbidden
        );
        assert_eq!(
            ServerError::NotFound("user".into()).code(),
            ErrorCode::NotFound
        );
        assert_eq!(
            ServerError::Conflict("dup".into()).code(),
            ErrorCode::Conflict
        );
        assert_eq!(
            ServerError::Internal("boom".into()).code(),
            ErrorCode::Internal
        );
    }

    #[test]
    fn validation_error_converts_via_from_and_maps_to_validation_code() {
        // `?` on a chirp-types ValidationError yields ServerError::Validation.
        let ve = ValidationError::Empty { field: "username" };
        let err: ServerError = ve.into();
        assert_eq!(err.code(), ErrorCode::ValidationError);
    }

    #[test]
    fn validation_error_lifts_field_into_api_error_details() {
        let err = ServerError::Validation(ValidationError::TooLong {
            field: "text",
            max: 280,
            actual: 999,
        });
        let api = err.into_api_error();
        assert_eq!(api.code, ErrorCode::ValidationError);
        assert_eq!(api.details.len(), 1);
        assert_eq!(api.details[0].field, "text");
        assert!(!api.message.is_empty());
    }

    #[test]
    fn non_validation_errors_have_no_field_details() {
        let api = ServerError::NotFound("chirp chirp-9".into()).into_api_error();
        assert_eq!(api.code, ErrorCode::NotFound);
        assert!(api.details.is_empty());
    }

    #[test]
    fn invalid_body_maps_to_validation_code_with_a_generic_field_detail() {
        // A transport-boundary deserialization rejection surfaces as VALIDATION_ERROR with a
        // single generic `body` FieldError carrying the rejection message.
        let err = ServerError::InvalidBody(
            "Failed to deserialize the JSON body into the target type".to_owned(),
        );
        assert_eq!(err.code(), ErrorCode::ValidationError);
        let api = err.into_api_error();
        assert_eq!(api.code, ErrorCode::ValidationError);
        assert_eq!(api.details.len(), 1);
        assert_eq!(api.details[0].field, "body");
        assert!(!api.details[0].message.is_empty());
    }
}
