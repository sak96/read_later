use anyhow::{Context, Error, bail};
use digest_auth::{AuthContext, parse as parse_digest};
use quick_xml::Reader;
use quick_xml::events::Event;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderName, HeaderValue, WWW_AUTHENTICATE};
use reqwest::{Client, Method, RequestBuilder, Response, StatusCode};
use std::time::UNIX_EPOCH;
use url::Url;

const DEPTH: HeaderName = HeaderName::from_static("depth");

#[derive(Debug, Clone)]
pub struct FileItem {
    path: String,
    is_file: bool,
    timestamp: i64,
    content: String,
}

impl FileItem {
    #[must_use]
    pub fn is_file(&self) -> bool {
        self.is_file
    }

    #[must_use]
    pub fn timestamp(&self) -> i64 {
        self.timestamp
    }

    #[must_use]
    pub fn content(&self) -> &str {
        &self.content
    }

    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AuthType {
    Anonymous,
    Basic,
    Digest,
}

impl AuthType {
    fn from_str(value: &str) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "basic" => Self::Basic,
            "digest" => Self::Digest,
            _ => Self::Anonymous,
        }
    }
}

pub struct WebDav {
    client: Client,
    base_url: Url,
    username: String,
    password: String,
    auth_type: AuthType,
}

impl WebDav {
    pub fn new(url: &str, username: &str, password: &str, auth_type: &str) -> Result<Self, Error> {
        let mut base_url = Url::parse(url).with_context(|| format!("invalid WebDAV URL: {url}"))?;

        if !base_url.path().ends_with('/') {
            let path = format!("{}/", base_url.path());
            base_url.set_path(&path);
        }

        let client = Client::builder()
            .build()
            .context("failed to create HTTP client")?;

        Ok(Self {
            client,
            base_url,
            username: username.to_owned(),
            password: password.to_owned(),
            auth_type: AuthType::from_str(auth_type),
        })
    }

    fn url(&self, path: &str) -> Result<Url, Error> {
        let path = path.trim_start_matches('/');

        self.base_url
            .join(path)
            .with_context(|| format!("invalid WebDAV path: {path}"))
    }

    async fn request(
        &self,
        method: Method,
        url: &Url,
        body: Option<Vec<u8>>,
    ) -> Result<Response, Error> {
        let mut request = self.client.request(method.clone(), url.clone());

        if let Some(ref body) = body {
            request = request.body(body.clone());
        }

        match self.auth_type {
            AuthType::Anonymous => self.send(request).await,

            AuthType::Basic => {
                self.send(request.basic_auth(&self.username, Some(&self.password)))
                    .await
            }

            AuthType::Digest => self.digest_request(method, url, body).await,
        }
    }

    async fn send(&self, request: RequestBuilder) -> Result<Response, Error> {
        request.send().await.context("WebDAV request failed")
    }

    async fn digest_request(
        &self,
        method: Method,
        url: &Url,
        body: Option<Vec<u8>>,
    ) -> Result<Response, Error> {
        // First request obtains the WWW-Authenticate challenge.
        let mut initial = self.client.request(method.clone(), url.clone());

        if let Some(ref body) = body {
            initial = initial.body(body.clone());
        }

        let response = initial
            .send()
            .await
            .context("WebDAV Digest challenge request failed")?;

        if response.status().is_success() {
            return Ok(response);
        }

        if response.status() != StatusCode::UNAUTHORIZED {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();

            bail!("WebDAV request failed: {status}: {text}");
        }

        let challenge = response
            .headers()
            .get(WWW_AUTHENTICATE)
            .context("Digest authentication challenge missing")?
            .to_str()
            .context("invalid WWW-Authenticate header")?;

        let mut prompt =
            parse_digest(challenge).context("invalid Digest authentication challenge")?;

        let uri = url.path().to_owned()
            + url
                .query()
                .map(|q| format!("?{q}"))
                .unwrap_or_default()
                .as_str();

        let context = match method {
            Method::GET => AuthContext::new(&self.username, &self.password, &uri),

            Method::POST => {
                AuthContext::new_post(&self.username, &self.password, &uri, body.as_deref())
            }

            _ => AuthContext::new_with_method(
                &self.username,
                &self.password,
                &uri,
                body.as_deref(),
                digest_auth::HttpMethod::from(method.as_str()),
            ),
        };

        let authorization = prompt
            .respond(&context)
            .context("failed to generate Digest authorization")?
            .to_string();

        let mut request = self.client.request(method, url.clone());

        request = request.header(
            AUTHORIZATION,
            HeaderValue::from_str(&authorization).context("invalid Digest Authorization header")?,
        );

        if let Some(body) = body {
            request = request.body(body);
        }

        self.send(request).await
    }

    pub async fn get(&self, path: &str) -> Result<String, Error> {
        let url = self.url(path)?;
        let response = self
            .request(Method::GET, &url, None)
            .await
            .context("WebDAV GET request failed")?;

        let status = response.status();
        let body = response
            .text()
            .await
            .context("failed to read WebDAV response body")?;

        if !status.is_success() {
            anyhow::bail!("WebDAV GET failed with {status}: {body}");
        }

        Ok(body)
    }

    pub async fn list(&self, path: &str) -> Result<Vec<FileItem>, Error> {
        let url = self.url(path)?;

        let body = r#"<?xml version="1.0" encoding="utf-8" ?>
<D:propfind xmlns:D="DAV:">
  <D:prop>
    <D:resourcetype/>
    <D:getlastmodified/>
    <D:getcontentlength/>
    <D:getcontenttype/>
  </D:prop>
</D:propfind>"#;

        let request_body = body.as_bytes().to_vec();

        let method = Method::from_bytes(b"PROPFIND").context("invalid PROPFIND method")?;

        let mut request = self
            .client
            .request(method.clone(), url.clone())
            .header(CONTENT_TYPE, "application/xml")
            .header(DEPTH, "1")
            .body(request_body.clone());

        let response = match self.auth_type {
            AuthType::Anonymous => self.send(request).await?,

            AuthType::Basic => {
                request = self
                    .client
                    .request(method.clone(), url.clone())
                    .header(CONTENT_TYPE, "application/xml")
                    .header(DEPTH, "1")
                    .basic_auth(&self.username, Some(&self.password))
                    .body(request_body);

                self.send(request).await?
            }

            AuthType::Digest => {
                self.digest_request(method, &url, Some(request_body))
                    .await?
            }
        };

        let xml = response
            .text()
            .await
            .context("failed to read PROPFIND response")?;

        parse_multistatus(&xml)
    }

    pub async fn put(&self, path: &str, content: &str) -> Result<(), Error> {
        let url = self.url(path)?;

        self.request(Method::PUT, &url, Some(content.as_bytes().to_vec()))
            .await?;

        Ok(())
    }

    pub async fn mkcol(&self, path: &str) -> Result<(), Error> {
        let url = self.url(path)?;

        let method = Method::from_bytes(b"MKCOL").context("invalid MKCOL method")?;

        self.request(method, &url, None).await?;

        Ok(())
    }
}

/*
 * Parsed representation of one <response> element.
 */
#[derive(Debug, Default)]
struct DavEntry {
    href: String,
    propstats: Vec<DavPropStat>,
}

/*
 * Parsed representation of one <propstat> element.
 */
#[derive(Debug, Default)]
struct DavPropStat {
    status: Option<String>,
    is_collection: bool,
    last_modified: Option<String>,
}

#[allow(clippy::too_many_lines)]
fn parse_multistatus(xml: &str) -> Result<Vec<FileItem>, Error> {
    let mut reader = Reader::from_str(xml);

    reader.config_mut().trim_text(true);

    let mut entries = Vec::new();

    let mut current_entry: Option<DavEntry> = None;
    let mut current_propstat: Option<DavPropStat> = None;

    let mut current_element: Option<Vec<u8>> = None;

    loop {
        match reader
            .read_event()
            .context("failed to read WebDAV XML event")?
        {
            Event::Start(event) => {
                let name = event.local_name();

                match name.as_ref() {
                    b"response" => {
                        current_entry = Some(DavEntry::default());
                    }

                    b"propstat" => {
                        current_propstat = Some(DavPropStat::default());
                    }

                    b"href" | b"status" | b"getlastmodified" => {
                        current_element = Some(name.as_ref().to_vec());
                    }

                    _ => {}
                }
            }

            Event::Empty(event) => {
                let name = event.local_name();

                if name.as_ref() == b"collection"
                    && let Some(propstat) = current_propstat.as_mut()
                {
                    propstat.is_collection = true;
                }
            }

            Event::Text(event) => {
                let text = event
                    .unescape()
                    .context("failed to unescape XML text")?
                    .into_owned();

                match current_element.as_deref() {
                    Some(b"href") => {
                        if let Some(entry) = current_entry.as_mut() {
                            entry.href = text;
                        }
                    }

                    Some(b"status") => {
                        if let Some(propstat) = current_propstat.as_mut() {
                            propstat.status = Some(text);
                        }
                    }

                    Some(b"getlastmodified") => {
                        if let Some(propstat) = current_propstat.as_mut() {
                            propstat.last_modified = Some(text);
                        }
                    }

                    _ => {}
                }
            }

            Event::End(event) => {
                let name = event.local_name();

                match name.as_ref() {
                    b"href" | b"status" | b"getlastmodified" => {
                        current_element = None;
                    }

                    b"propstat" => {
                        if let Some(propstat) = current_propstat.take()
                            && let Some(entry) = current_entry.as_mut()
                        {
                            entry.propstats.push(propstat);
                        }
                    }

                    b"response" => {
                        if let Some(entry) = current_entry.take() {
                            entries.push(entry);
                        }
                    }

                    _ => {}
                }
            }

            Event::Eof => break,

            _ => {}
        }
    }

    let mut files = Vec::with_capacity(entries.len());

    for entry in entries {
        let propstat = entry.propstats.iter().find(|propstat| {
            propstat
                .status
                .as_deref()
                .is_none_or(|status| status.contains("200"))
        });

        let is_file = propstat.is_none_or(|propstat| !propstat.is_collection);

        let timestamp = propstat
            .and_then(|propstat| propstat.last_modified.as_deref())
            .and_then(parse_http_timestamp)
            .unwrap_or(0);

        files.push(FileItem {
            path: entry.href,
            is_file,
            timestamp,
            content: String::new(),
        });
    }

    Ok(files)
}

fn parse_http_timestamp(value: &str) -> Option<i64> {
    httpdate::parse_http_date(value).ok().and_then(|time| {
        time.duration_since(UNIX_EPOCH)
            .ok()
            .and_then(|duration| i64::try_from(duration.as_secs()).ok())
    })
}
