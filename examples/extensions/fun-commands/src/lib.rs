use serein_extension_sdk::{Element, Invocation, Output};

/// `/mock`, `/reverse` and `/upper` put their result in the draft; you still press Send.
fn handle(input: Invocation) -> Output {
	let text = input.composer.unwrap_or_default();
	if text.trim().is_empty() {
		return Output {
			panel: vec![Element::Text {
				text: format!("Usage: /{} some text", input.action),
			}],
			..Default::default()
		};
	}
	let result = match input.action.as_str() {
		"mock" => {
			let mut upper = false;
			text.chars()
				.map(|c| {
					if !c.is_alphabetic() {
						return c.to_string();
					}
					upper = !upper;
					if upper {
						c.to_uppercase().to_string()
					} else {
						c.to_lowercase().to_string()
					}
				})
				.collect()
		}
		"reverse" => text.chars().rev().collect(),
		"upper" => text.to_uppercase(),
		_ => return Output::default(),
	};
	Output {
		replacement: Some(result),
		..Default::default()
	}
}
serein_extension_sdk::export!(handle);

#[cfg(test)]
mod tests {
	use super::*;

	fn run(action: &str, text: &str) -> Output {
		handle(Invocation {
			action: action.into(),
			composer: Some(text.into()),
			..Default::default()
		})
	}

	#[test]
	fn commands_transform_text_and_explain_missing_arguments() {
		assert_eq!(run("mock", "hi you").replacement.as_deref(), Some("Hi YoU"));
		assert_eq!(run("reverse", "abc").replacement.as_deref(), Some("cba"));
		assert_eq!(run("upper", "ok").replacement.as_deref(), Some("OK"));
		let empty = run("upper", "  ");
		assert!(empty.replacement.is_none() && !empty.panel.is_empty());
	}
}
