use serde::{Deserialize, Serialize};

pub const MAX_NETWORK_HOSTS: usize = 8;
pub const MAX_FETCH_URL_BYTES: usize = 2048;
pub const MAX_FETCH_BYTES: usize = 64 * 1024;

/// One HTTPS GET for the host to perform. The URL's host must be listed in the manifest's
/// `network_hosts`; the result arrives later at your `fetch_result` action.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FetchRequest {
	/// Your label for this request (lowercase letters, digits and `-`), returned with the result.
	pub id: String,
	pub url: String,
}

/// The body of a completed request (at most 64 KiB of UTF-8), or why it failed.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FetchResult {
	pub id: String,
	pub url: String,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub body: Option<String>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub error: Option<String>,
}

/// Opt-in output that can ask for one `fetch`; keeps `Output` struct literals compiling.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkOutput {
	#[serde(flatten)]
	pub output: crate::Output,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub fetch: Option<FetchRequest>,
}

/// Input for a plugin that uses `fetch`: `fetch_result` is set for its `fetch_result` action.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkInvocation {
	#[serde(flatten)]
	pub invocation: crate::Invocation,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub fetch_result: Option<FetchResult>,
}
