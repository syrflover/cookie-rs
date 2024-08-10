use std::collections::HashMap;

use http::{
    header::{self, HeaderName, HeaderValue, InvalidHeaderValue},
    HeaderMap,
};
use itertools::Itertools;

#[derive(Debug, Default, Clone)]
pub struct Cookie {
    inner: HashMap<String, String>,
}

impl Cookie {
    pub fn new() -> Self {
        Default::default()
    }

    /// returns None
    /// - if header_name is not cookie or set-cookie
    /// - if hasn't cookie or set-cookie in headers
    /// - if failed to `header_value.to_str()`
    /// - if failed to parse cookie
    pub fn from_headers(header_name: HeaderName, headers: &HeaderMap) -> Option<Self> {
        if let header::COOKIE = header_name {
            let x = headers.get(header_name)?;
            return Cookie::from_cookie(x);
        }

        if let header::SET_COOKIE = header_name {
            let xs = headers
                .iter()
                .filter_map(|(header_name, x)| (header_name == header::SET_COOKIE).then_some(x));
            return Cookie::from_set_cookie(xs);
        }

        None
    }

    fn from_cookie(x: &HeaderValue) -> Option<Self> {
        // key1=avchdef; key2=qwehkdfsjd
        // key1=afjkd

        let mut inner = HashMap::new();

        let x = x.to_str().ok()?;
        for key_value in x.split(';') {
            let (key, value) = key_value.split_once('=')?;

            inner.insert(key.trim().to_owned(), value.to_owned());
        }

        Self { inner }.into()
    }

    fn from_set_cookie<'a, I>(xs: I) -> Option<Self>
    where
        I: Iterator<Item = &'a HeaderValue>,
    {
        // Set-Cookie: key1=value; Max-Age=12345; Domain=eeeee.com; HttpOnly; Secure
        // Set-Cookie: key2=value

        let mut inner = HashMap::new();

        for x in xs {
            let key_value = x.to_str().ok()?.split(';').next()?;

            let (key, value) = key_value.split_once('=')?;

            inner.insert(key.to_owned(), value.to_owned());
        }

        Self { inner }.into()
    }

    pub fn add(&mut self, key: &str, value: &str) {
        self.inner.insert(key.to_owned(), value.to_owned());
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.inner.get(key).map(|st| st as &str)
    }

    pub fn get2(&self, key1: &str, key2: &str) -> Option<(&str, &str)> {
        self.get(key1).and_then(|x| Some((x, self.get(key2)?)))
    }

    pub fn take(&mut self, key: &str) -> Option<String> {
        self.inner.remove(key)
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// used `format!` macro
    pub fn to_str(&self) -> String {
        self.inner
            .iter()
            .map(|(key, value)| format!("{}={}", key, value))
            .join(";")
    }

    /// used `+` operator
    pub fn into_str(self) -> String {
        self.inner
            .into_iter()
            .map(|(key, value)| key + "=" + &value)
            .join(";")
    }
}

impl FromIterator<(String, String)> for Cookie {
    fn from_iter<T: IntoIterator<Item = (String, String)>>(iter: T) -> Self {
        Self {
            inner: HashMap::from_iter(iter),
        }
    }
}

impl<'a> FromIterator<(&'a str, &'a str)> for Cookie {
    fn from_iter<T: IntoIterator<Item = (&'a str, &'a str)>>(iter: T) -> Self {
        Self::from_iter(
            iter.into_iter()
                .map(|(key, value)| (key.to_owned(), value.to_owned())),
        )
    }
}

impl<'a> FromIterator<(&'a str, String)> for Cookie {
    fn from_iter<T: IntoIterator<Item = (&'a str, String)>>(iter: T) -> Self {
        Self::from_iter(iter.into_iter().map(|(key, value)| (key.to_owned(), value)))
    }
}

impl<'a> FromIterator<(String, &'a str)> for Cookie {
    fn from_iter<T: IntoIterator<Item = (String, &'a str)>>(iter: T) -> Self {
        Self::from_iter(iter.into_iter().map(|(key, value)| (key, value.to_owned())))
    }
}

impl TryInto<HeaderValue> for Cookie {
    type Error = InvalidHeaderValue;

    fn try_into(self) -> Result<HeaderValue, Self::Error> {
        (&self).try_into()
    }
}

impl<'a> TryInto<HeaderValue> for &'a Cookie {
    type Error = InvalidHeaderValue;

    fn try_into(self) -> Result<HeaderValue, Self::Error> {
        self.to_str().try_into()
    }
}

#[test]
fn test_from_cookie() {
    let x = "madome_access_token=avchdef; madome_refresh_token=qwehkdfsjd";
    let mut headers = HeaderMap::new();
    headers.insert(header::COOKIE, x.try_into().unwrap());

    let cookie = Cookie::from_headers(header::COOKIE, &headers).unwrap();

    assert_eq!(cookie.len(), 2);
    assert_eq!(cookie.get("madome_access_token"), Some("avchdef"));
    assert_eq!(cookie.get("madome_refresh_token"), Some("qwehkdfsjd"));
}

#[test]
fn test_from_cookie_without_semicolon() {
    let x = "madome_access_token=avchdef";
    let mut headers = HeaderMap::new();
    headers.insert(header::COOKIE, x.try_into().unwrap());

    let cookie = Cookie::from_headers(header::COOKIE, &headers).unwrap();

    assert_eq!(cookie.len(), 1);
    assert_eq!(cookie.get("madome_access_token"), Some("avchdef"));
}

#[test]
fn test_from_set_cookie() {
    let xs = [
        "madome_access_token=admjsher; Max-Age=12345; Domain=eeeee.com; HttpOnly; Secure",
        "madome_refresh_token=kfadbhe",
    ];
    let mut headers = HeaderMap::new();
    headers.insert(header::COOKIE, "any value".try_into().unwrap());
    for x in xs {
        headers.append(header::SET_COOKIE, x.try_into().unwrap());
    }

    let cookie = Cookie::from_headers(header::SET_COOKIE, &headers).unwrap();

    assert_eq!(cookie.len(), 2);
    assert_eq!(cookie.get("madome_access_token"), Some("admjsher"));
    assert_eq!(cookie.get("madome_refresh_token"), Some("kfadbhe"));
}
