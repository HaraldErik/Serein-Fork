use serein_extension_sdk::{Element, Invocation, Output};

const MAX_RULES: usize = 50;
const MAX_WORD_BYTES: usize = 64;
const EXAMPLE: &str = "teh=the; recieve=receive";

/// Parses `from=to; from=to`. Empty or oversized pairs are skipped, at most `MAX_RULES` kept.
fn rules(text: &str) -> Vec<(String, String)> {
	text.split(';')
		.filter_map(|pair| {
			let (from, to) = pair.split_once('=')?;
			let (from, to) = (from.trim(), to.trim());
			(!from.is_empty() && from.len() <= MAX_WORD_BYTES && to.len() <= MAX_WORD_BYTES)
				.then(|| (from.to_owned(), to.to_owned()))
		})
		.take(MAX_RULES)
		.collect()
}

/// Replaces whole words only, so `teh` changes but `tehran` does not.
fn replace(draft: &str, rules: &[(String, String)]) -> String {
	let mut out = String::with_capacity(draft.len());
	let mut word = String::new();
	let flush = |word: &mut String, out: &mut String| {
		match rules.iter().find(|(from, _)| from == word) {
			Some((_, to)) => out.push_str(to),
			None => out.push_str(word),
		}
		word.clear();
	};
	for c in draft.chars() {
		if c.is_alphanumeric() || c == '\'' {
			word.push(c);
		} else {
			flush(&mut word, &mut out);
			out.push(c);
		}
	}
	flush(&mut word, &mut out);
	out
}

fn settings(saved: &str, note: &str) -> Vec<Element> {
	vec![
		Element::Heading {
			text: "Text Replace".into(),
		},
		Element::Text {
			text: format!("Whole words are replaced when you press Send. {note}"),
		},
		Element::TextInput {
			id: "rules".into(),
			label: "Rules (from=to; from=to)".into(),
			value: saved.into(),
		},
		Element::Button {
			id: "save".into(),
			label: "Save rules".into(),
		},
	]
}

fn handle(input: Invocation) -> Output {
	let saved = input.storage.clone().unwrap_or_else(|| EXAMPLE.into());
	match input.action.as_str() {
		"send" => {
			let Some(draft) = input.composer else {
				return Output::default();
			};
			let rewritten = replace(&draft, &rules(&saved));
			Output {
				replacement: (rewritten != draft).then_some(rewritten),
				..Default::default()
			}
		}
		"save" => {
			let text = input.value("rules").unwrap_or_default().to_owned();
			let count = rules(&text).len();
			Output {
				panel: settings(&text, &format!("Saved {count} rule(s).")),
				storage: Some(text),
				..Default::default()
			}
		}
		"settings" => Output {
			panel: settings(&saved, ""),
			..Default::default()
		},
		_ => Output::default(),
	}
}
serein_extension_sdk::export!(handle);

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn whole_words_change_and_bad_rules_are_skipped() {
		let rules = rules(
			"teh=the; =x; tehran; recieve = receive; long="
				.to_owned()
				.as_str(),
		);
		assert_eq!(rules.len(), 3);
		assert_eq!(
			replace("teh cat, recieve tehran. teh", &rules),
			"the cat, receive tehran. the"
		);
		assert_eq!(replace("long word", &rules), " word");
		assert!(super::rules(&format!("{}=y", "x".repeat(MAX_WORD_BYTES + 1))).is_empty());
		assert_eq!(super::rules(&"a=b;".repeat(MAX_RULES + 5)).len(), MAX_RULES);
	}

	#[test]
	fn unchanged_drafts_return_no_replacement() {
		let output = handle(Invocation {
			action: "send".into(),
			composer: Some("nothing to fix".into()),
			..Default::default()
		});
		assert!(output.replacement.is_none());
		let output = handle(Invocation {
			action: "send".into(),
			composer: Some("teh end".into()),
			..Default::default()
		});
		assert_eq!(output.replacement.as_deref(), Some("the end"));
	}
}
