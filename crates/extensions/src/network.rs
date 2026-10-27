//! Pull-only plugin network access. Plugins name the hosts they may read in their manifest;
//! the host performs one bounded HTTPS GET per request and delivers the body to a
//! `fetch_result` action. Nothing the plugin chooses is sent except the URL itself.
use crate::{Error, Manifest};
use serde::{Deserialize, Serialize};

pub const MAX_NETWORK_HOSTS: usize = 8;
pub const MAX_FETCH_URL_BYTES: usize = 2048;
pub const MAX_FETCH_BYTES: usize = 64 * 1024;
pub const MAX_FETCH_ERROR_BYTES: usize = 256;

/// One HTTPS GET a plugin asks the host to perform.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FetchRequest {
	/// Author-chosen label returned with the result.
	pub id: String,
	pub url: String,
}

/// The body of a completed request, or why it failed.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FetchResult {
	pub id: String,
	pub url: String,
	/// UTF-8 body (invalid bytes replaced), at most 64 KiB; `None` when the request failed.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub body: Option<String>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub error: Option<String>,
}

/// A lowercase public DNS name such as `api.example.com`: no scheme, port, path or wildcard.
pub fn valid_host(host: &str) -> bool {
	host.len() <= 253
		&& host.contains('.')
		&& host.split('.').all(|label| {
			!label.is_empty()
				&& label.len() <= 63
				&& !label.starts_with('-')
				&& !label.ends_with('-')
				&& label
					.bytes()
					.all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
		}) && !host
		.rsplit('.')
		.next()
		.is_some_and(|tld| tld.bytes().all(|b| b.is_ascii_digit()))
}

/// Whether `url` is a credential-free HTTPS URL on port 443 for one of the manifest's hosts.
pub fn allowed_url(url: &str, hosts: &[String]) -> bool {
	url.len() <= MAX_FETCH_URL_BYTES
		&& url::Url::parse(url).is_ok_and(|url| {
			url.scheme() == "https"
				&& url.username().is_empty()
				&& url.password().is_none()
				&& url.port_or_known_default() == Some(443)
				&& url
					.host_str()
					.is_some_and(|host| hosts.iter().any(|allowed| allowed == host))
		})
}

impl FetchRequest {
	pub fn validate(&self, manifest: &Manifest) -> Result<(), Error> {
		if !crate::valid_id(&self.id) || !allowed_url(&self.url, &manifest.network_hosts) {
			return Err(Error::Invalid);
		}
		Ok(())
	}
}

impl FetchResult {
	pub fn validate(&self) -> Result<(), Error> {
		if self.body.is_some() == self.error.is_some()
			|| self.url.len() > MAX_FETCH_URL_BYTES
			|| self
				.body
				.as_ref()
				.is_some_and(|body| body.len() > MAX_FETCH_BYTES)
			|| self
				.error
				.as_ref()
				.is_some_and(|error| error.len() > MAX_FETCH_ERROR_BYTES)
			|| !crate::valid_id(&self.id)
		{
			return Err(Error::Invalid);
		}
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn hosts_and_urls_are_exact_public_https() {
		for good in ["api.example.com", "a-b.example.co"] {
			assert!(valid_host(good), "{good}");
		}
		for bad in [
			"localhost",
			"*.example.com",
			"Example.com",
			"example.com:8443",
			"https://example.com",
			"127.0.0.1",
			"-x.example.com",
			"",
		] {
			assert!(!valid_host(bad), "{bad}");
		}
		let hosts = vec!["api.example.com".to_owned()];
		assert!(allowed_url("https://api.example.com/v1?q=1", &hosts));
		for bad in [
			"http://api.example.com/",
			"https://evil.example.com/",
			"https://api.example.com.evil.net/",
			"https://user:pw@api.example.com/",
			"https://api.example.com:8443/",
		] {
			assert!(!allowed_url(bad, &hosts), "{bad}");
		}
		assert!(!allowed_url(
			&format!(
				"https://api.example.com/{}",
				"a".repeat(MAX_FETCH_URL_BYTES)
			),
			&hosts
		));
	}
}
