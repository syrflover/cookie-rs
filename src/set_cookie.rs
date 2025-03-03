use std::{
    collections::{HashMap, hash_map},
    str::FromStr,
};

use http::{
    HeaderMap, HeaderValue,
    header::{self, HeaderName},
};

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone, Copy)]
pub enum SameSite {
    Strict,
    Lax,
    None,
}

impl SameSite {
    fn as_str(&self) -> &'static str {
        match self {
            SameSite::Strict => "Strict",
            SameSite::Lax => "Lax",
            SameSite::None => "None",
        }
    }
}

impl FromStr for SameSite {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let same_site = match s {
            "strict" => Self::Strict,
            "lax" => Self::Lax,
            "none" => Self::None,
            _ => return Err(()),
        };

        Ok(same_site)
    }
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone, Default)]
pub struct SetCookieOptions {
    pub http_only: bool,
    pub secure: bool,
    // expires: ,
    /// Seconds
    pub max_age: Option<i64>,
    pub domain: Option<String>,
    pub path: Option<String>,
    pub same_site: Option<SameSite>,
}

impl SetCookieOptions {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn is_set_cookie_option(st: &str) -> bool {
        let st = st.to_lowercase();

        st.starts_with("max-age=")
            || st.starts_with("domain=")
            || st.starts_with("path=")
            // || st.starts_with("expires=")
            || st.eq("httponly")
            || st.eq("secure")
    }

    pub fn http_only(mut self, http_only: bool) -> Self {
        self.http_only = http_only;

        self
    }

    #[allow(dead_code)]
    pub fn secure(mut self, secure: bool) -> Self {
        self.secure = secure;

        self
    }

    pub fn max_age(mut self, max_age: i64) -> Self {
        self.max_age.replace(max_age);

        self
    }

    pub fn domain(mut self, domain: impl Into<String>) -> Self {
        self.domain.replace(domain.into());

        self
    }

    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.path.replace(path.into());

        self
    }

    pub fn same_site(mut self, same_site: SameSite) -> Self {
        self.same_site.replace(same_site);

        self
    }
}

impl<'a> From<Vec<&'a str>> for SetCookieOptions {
    fn from(xs: Vec<&'a str>) -> Self {
        let mut options = SetCookieOptions {
            domain: None,
            max_age: None,
            path: None,
            http_only: false,
            secure: false,
            same_site: None,
        };

        for st in xs.iter().map(|st| st.to_lowercase()) {
            if st.starts_with("domain=") {
                let domain = st.split('=').nth(1).unwrap_or_default();
                options.domain.replace(domain.to_string());
            } else if st.starts_with("max-age=") {
                let max_age = st
                    .split('=')
                    .nth(1)
                    .and_then(|x| x.parse().ok())
                    .unwrap_or_default();
                options.max_age.replace(max_age);
            } else if st.starts_with("path=") {
                let path = st.split('=').nth(1).unwrap_or("/");
                options.path.replace(path.to_string());
            } else if st.starts_with("httponly") {
                options.http_only = true;
            } else if st.starts_with("secure") {
                options.secure = true;
            } else if st.starts_with("samesite") {
                let same_site = st.split('=').nth(1).and_then(|s| s.parse().ok());
                if let Some(same_site) = same_site {
                    options.same_site.replace(same_site);
                }
            }
        }

        options
    }
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone, Default)]
pub struct SetCookie {
    inner: HashMap<String, (String, SetCookieOptions)>,
}

impl SetCookie {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn from_headers(headers: &HeaderMap) -> Self {
        headers
            .iter()
            .filter(|(k, _)| k == &header::SET_COOKIE)
            .filter_map(|(_, v)| v.to_str().ok())
            .into()
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.inner.get(key).map(|(v, _)| v.as_str())
    }

    pub fn take(&mut self, key: &str) -> Option<String> {
        self.inner.remove(key).map(|(x, _)| x)
    }

    pub fn set(
        mut self,
        key: impl Into<String>,
        value: impl Into<String>,
        options: SetCookieOptions,
    ) -> Self {
        self.inner.insert(key.into(), (value.into(), options));

        self
    }

    #[allow(dead_code)]
    pub fn remove(mut self, key: impl Into<String>) -> Self {
        self.inner.remove(&key.into());

        self
    }

    /// SetHeaders::headers(set_cookie.iter());
    pub fn iter(&self) -> impl Iterator<Item = (HeaderName, HeaderValue)> + '_ {
        self.inner
            .iter()
            .map(|(key, (value, options))| fmt(key, value, options))
            .map(|st| (header::SET_COOKIE, st.parse().unwrap()))
    }
}

pub struct IntoIter {
    inner: hash_map::IntoIter<String, (String, SetCookieOptions)>,
}

impl Iterator for IntoIter {
    type Item = (HeaderName, HeaderValue);

    fn next(&mut self) -> Option<Self::Item> {
        let (key, (value, options)) = self.inner.next()?;
        let set_cookie = fmt(&key, &value, &options).parse().ok()?;

        Some((header::SET_COOKIE, set_cookie))
    }
}

impl IntoIterator for SetCookie {
    type IntoIter = IntoIter;
    type Item = (HeaderName, HeaderValue);

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            inner: self.inner.into_iter(),
        }
    }
}

fn fmt(
    key: &str,
    value: &str,
    SetCookieOptions {
        domain,
        max_age,
        http_only,
        secure,
        path,
        same_site,
    }: &SetCookieOptions,
) -> String {
    let max_age = max_age.map(|x| x.to_string());
    let same_site = same_site.map(|x| x.as_str());

    let capacity = 1
        + key.len()
        + value.len()
        + domain.as_deref().map(|x| 2 + 6 + x.len()).unwrap_or(0)
        + max_age.as_deref().map(|x| 2 + 7 + x.len()).unwrap_or(0)
        + path.as_deref().map(|x| 2 + 4 + x.len()).unwrap_or(0)
        + same_site.map(|x| 2 + 8 + x.len()).unwrap_or(0)
        + if *http_only { 1 + 8 } else { 0 }
        + if *secure { 1 + 6 } else { 0 };

    let mut base = String::with_capacity(capacity);

    base.push_str(key);
    base.push('=');
    base.push_str(value);

    // let mut base = format!("{}={}", key, value);

    if let Some(domain) = domain {
        // base = format!("{}; Domain={}", base, domain);
        base.push(';');

        base.push_str("Domain=");
        base.push_str(domain);
    }

    if let Some(max_age) = max_age {
        // base = format!("{}; Max-Age={}", base, max_age);
        base.push(';');

        base.push_str("Max-Age=");
        base.push_str(&max_age);
    }

    if let Some(path) = path {
        // base = format!("{}; Path={}", base, path);
        base.push(';');

        base.push_str("Path=");
        base.push_str(path);
    }

    if let Some(same_site) = same_site {
        // base = format!("{}; SameSite={}", base, same_site.as_str())
        base.push(';');

        base.push_str("SameSite=");
        base.push_str(same_site);
    }

    if *http_only {
        // base = format!("{}; HttpOnly", base);
        base.push(';');

        base.push_str("HttpOnly");
    }

    if *secure {
        // base = format!("{}; Secure", base);
        base.push(';');

        base.push_str("Secure");
    }

    #[cfg(debug_assertions)]
    {
        // assert_eq!(capacity, base.len());
        if capacity != base.len() {
            tracing::warn!("capacity != base.len() : {} != {}", capacity, base.len());
        }
    }

    base
}

impl<A, I> From<I> for SetCookie
where
    A: AsRef<str>,
    I: Iterator<Item = A>,
{
    fn from(it: I) -> Self {
        // Set-Cookie: key=value; Max-Age=12345; Domain=eeeee.com; HttpOnly; Secure

        let mut set_cookie = Self::new();

        for header_value in it {
            let (options, key_value): (Vec<_>, Vec<_>) = header_value
                .as_ref()
                .split(';')
                .map(|st| st.trim())
                .partition(|st| SetCookieOptions::is_set_cookie_option(st));

            // println!("options = {:?}", options);
            // println!("key_value = {:?}", key_value);

            let mut key_value = key_value.first().map(|st| st.split('='));

            let key = key_value.as_mut().and_then(|st| st.next());
            let value = key_value.as_mut().and_then(|st| st.next());

            let (key, value) = match (key, value) {
                (Some(key), Some(value)) => (key, value),
                _ => continue,
            };

            // println!("key = {}", key);
            // println!("value = {}", value);

            set_cookie
                .inner
                .insert(key.to_string(), (value.to_string(), options.into()));
        }

        set_cookie
    }
}

#[test]
fn set_cookie_from_header_values() {
    let header_value = "key=value; Max-Age=12345; Domain=eeee.com; HttpOnly; Secure; Path=/abcd/e";

    let it = [header_value];

    let set_cookie = SetCookie::from(it.iter());

    // println!("{:?}", set_cookie);

    let expected = SetCookie::new().set(
        "key",
        "value",
        SetCookieOptions::new()
            .http_only(true)
            .secure(true)
            .max_age(12345)
            .domain("eeee.com")
            .path("/abcd/e"),
    );

    assert_eq!(set_cookie, expected);
}

#[test]
fn to_headers() {
    let set_cookie = SetCookie::new().set(
        "key1",
        "key2",
        SetCookieOptions::new()
            .domain("example.com")
            .path("/dasdj/ed")
            .max_age(4432483)
            .secure(true)
            .http_only(true)
            .same_site(SameSite::Strict),
    );

    let _r = set_cookie.into_iter().collect::<Vec<_>>();
}
