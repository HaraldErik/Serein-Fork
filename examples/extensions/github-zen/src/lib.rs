use serein_extension_sdk::{Element, FetchRequest, NetworkInvocation, NetworkOutput, Output};

const URL: &str = "https://api.github.com/zen";
const MAX_SAYING_CHARS: usize = 200;

fn panel(text: &str) -> Vec<Element> {
	vec![
		Element::Heading {
			text: "GitHub Zen".into(),
		},
		Element::Text { text: text.into() },
		Element::Button {
			id: "fetch".into(),
			label: "Get a new saying".into(),
		},
	]
}

/// Pull-only example: the button asks Serein to download one short text from the listed
/// host; the reply arrives at `fetched`, which saves and shows it.
fn handle(input: NetworkInvocation) -> NetworkOutput {
	let saved = input.invocation.storage.clone();
	match input.invocation.action.as_str() {
		"fetch" => NetworkOutput {
			output: Output {
				panel: panel("Downloading…"),
				..Default::default()
			},
			fetch: Some(FetchRequest {
				id: "zen".into(),
				url: URL.into(),
			}),
		},
		"fetched" => {
			let Some(result) = input.fetch_result else {
				return NetworkOutput::default();
			};
			let output = match result.body {
				Some(body) => {
					let saying: String = body.trim().chars().take(MAX_SAYING_CHARS).collect();
					Output {
						panel: panel(&saying),
						storage: Some(saying),
						..Default::default()
					}
				}
				None => Output {
					panel: panel(&format!(
						"Download failed: {}",
						result.error.unwrap_or_default()
					)),
					..Default::default()
				},
			};
			NetworkOutput {
				output,
				fetch: None,
			}
		}
		"show" => NetworkOutput {
			output: Output {
				panel: panel(saved.as_deref().unwrap_or("No saying yet.")),
				..Default::default()
			},
			fetch: None,
		},
		_ => NetworkOutput::default(),
	}
}
serein_extension_sdk::export!(handle);

#[cfg(test)]
mod tests {
	use super::*;
	use serein_extension_sdk::{FetchResult, Invocation};

	fn run(action: &str, fetch_result: Option<FetchResult>) -> NetworkOutput {
		handle(NetworkInvocation {
			invocation: Invocation {
				action: action.into(),
				..Default::default()
			},
			fetch_result,
		})
	}

	#[test]
	fn requests_one_listed_url_and_saves_a_bounded_reply() {
		assert_eq!(run("fetch", None).fetch.unwrap().url, URL);
		let reply = run(
			"fetched",
			Some(FetchResult {
				id: "zen".into(),
				url: URL.into(),
				body: Some(format!(" {} \n", "z".repeat(500))),
				error: None,
			}),
		);
		assert_eq!(reply.output.storage.unwrap().len(), MAX_SAYING_CHARS);
		let failed = run(
			"fetched",
			Some(FetchResult {
				id: "zen".into(),
				url: URL.into(),
				body: None,
				error: Some("timed out".into()),
			}),
		);
		assert!(failed.output.storage.is_none() && failed.fetch.is_none());
	}
}
