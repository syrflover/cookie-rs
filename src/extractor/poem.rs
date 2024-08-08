use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

use http::header;
use poem::{error::ResponseError, http::StatusCode, FromRequest, Request, RequestBody};

use crate::Cookie as CookieMap;

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

impl ResponseError for CookieRejection {
    fn status(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

#[derive(Debug, Clone)]
pub struct Cookie(CookieMap);

impl Deref for Cookie {
    type Target = CookieMap;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Cookie {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Cookie {
    async fn internal_from_request(request: &Request) -> Result<Self, CookieRejection> {
        CookieMap::from_headers(header::COOKIE, request.headers())
            .ok_or(CookieRejection)
            .map(Self)
    }
}

impl<'a> FromRequest<'a> for Cookie {
    async fn from_request(request: &'a Request, _body: &mut RequestBody) -> poem::Result<Self> {
        Self::internal_from_request(request)
            .await
            .map_err(Into::into)
    }
}
