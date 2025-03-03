use std::fmt::Display;

use axum::{extract::FromRequestParts, response::IntoResponse};
use http::{header, request::Parts, StatusCode};

use crate::Cookie;

#[derive(Debug)]
pub struct CookieRejection;

impl Display for CookieRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CookieRejection")
    }
}

impl std::error::Error for CookieRejection {
    fn description(&self) -> &str {
        "CookieRejection"
    }
}

impl IntoResponse for CookieRejection {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}

impl<S> FromRequestParts<S> for Cookie
where
    S: Send + Sync,
{
    type Rejection = CookieRejection;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Cookie::from_headers(header::COOKIE, &parts.headers).ok_or(CookieRejection)
    }
}
