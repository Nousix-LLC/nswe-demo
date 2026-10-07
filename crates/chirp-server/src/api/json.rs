//! A JSON request-body extractor that routes decoding/validation failures through [`ServerError`].
//!
//! [`ValidatedJson`] is a thin wrapper around axum's [`Json`](axum::Json) extractor with one
//! behavioral difference: instead of letting a [`JsonRejection`](axum::extract::rejection::JsonRejection)
//! render axum's **default plain-text `422 Unprocessable Entity`**, it converts the rejection into a
//! [`ServerError::InvalidBody`] (via the `From<JsonRejection>` impl in [`crate::error`]), which the
//! committed `impl IntoResponse for ServerError` renders as the contract's `ApiError` with
//! `VALIDATION_ERROR`/**400**.
//!
//! # Why this exists
//!
//! The `chirp-types` value objects (`Username`, `ChirpText`) validate on deserialization via
//! `#[serde(try_from = "String")]`, so an invalid field value fails at the serde boundary of the
//! body extractor — *before* any handler body runs. With the plain [`Json`](axum::Json) extractor
//! that failure is axum's `JsonRejection::JsonDataError` → `422` plain text, which never routes
//! through [`ServerError`] and so bypasses the wire error contract. Extracting the body with
//! `ValidatedJson` instead funnels every body-decoding failure through the one error seam, so the
//! `error_mapping` contract (`input/DTO validation failure → VALIDATION_ERROR → 400`) holds on the
//! HTTP surface.
//!
//! # Usage
//!
//! A handler takes `ValidatedJson<T>` exactly where it would have taken `Json<T>` for the request
//! body; the inner `T` is accessed by destructuring the tuple struct:
//!
//! ```ignore
//! async fn create_user(
//!     State(state): State<AppState>,
//!     ValidatedJson(request): ValidatedJson<CreateUserRequest>,
//! ) -> Result<(StatusCode, Json<User>), ServerError> { /* ... */ }
//! ```

use axum::extract::{FromRequest, Request};
use axum::Json;
use serde::de::DeserializeOwned;

use crate::error::ServerError;

/// A request-body JSON extractor whose rejection is a [`ServerError`] (→ the contract's `ApiError`),
/// not axum's default plain-text response.
///
/// Behaves like [`Json<T>`](axum::Json) on success, yielding the decoded `T`; on any JSON decoding
/// or value-object validation failure it produces [`ServerError::InvalidBody`], which renders as a
/// `VALIDATION_ERROR`/`400` [`ApiError`](chirp_types::api::ApiError).
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedJson<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = ServerError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        // Delegate the actual decoding to axum's `Json`; the `?` maps a `JsonRejection` to
        // `ServerError::InvalidBody` through the `From<JsonRejection> for ServerError` impl.
        let Json(value) = Json::<T>::from_request(req, state).await?;
        Ok(ValidatedJson(value))
    }
}
