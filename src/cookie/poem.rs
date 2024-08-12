use std::fmt::Display;

use http::header;
use poem::{error::ResponseError, http::StatusCode, FromRequest, Request, RequestBody};

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

impl ResponseError for CookieRejection {
    fn status(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

impl<'a> FromRequest<'a> for Cookie {
    async fn from_request(request: &'a Request, _body: &mut RequestBody) -> poem::Result<Self> {
        Cookie::from_headers(header::COOKIE, request.headers())
            .ok_or(CookieRejection)
            .map_err(Into::into)
    }
}
