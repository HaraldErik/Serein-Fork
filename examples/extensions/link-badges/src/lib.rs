use serein_extension_sdk::{
	Badge, DecorateInvocation, Decoration, DecorationOutput, MessageDecoration, MessageEventKind,
};

/// LINK for messages containing a URL, LONG for very long ones. Reads message text only
/// for this call; nothing is stored.
fn badges(content: &str) -> Option<Decoration> {
	let mut badges = Vec::new();
	if content.contains("http://") || content.contains("https://") {
		badges.push(Badge {
			text: "LINK".into(),
			tooltip: Some("This message contains a link".into()),
			color: Some(0x5865F2),
		});
	}
	if content.chars().count() > 500 {
		badges.push(Badge {
			text: "LONG".into(),
			tooltip: Some("More than 500 characters".into()),
			color: Some(0xFAA61A),
		});
	}
	(!badges.is_empty()).then(|| Decoration {
		badges,
		..Default::default()
	})
}

/// `message-event` badges new and edited messages; `decorate` badges messages that were
/// already loaded (history, or everything on screen when the plugin is enabled).
fn handle(input: DecorateInvocation) -> DecorationOutput {
	let mut output = DecorationOutput::default();
	match input.invocation.action.as_str() {
		"message-event" => {
			if let Some(event) = input.message_event
				&& event.kind != MessageEventKind::Delete
			{
				output.decoration = badges(event.content.as_deref().unwrap_or_default());
			}
		}
		"decorate" => {
			output.decorations = input
				.messages
				.into_iter()
				.filter_map(|message| {
					let decoration = badges(message.content.as_deref().unwrap_or_default())?;
					Some(MessageDecoration {
						message_id: message.message_id,
						decoration,
					})
				})
				.collect();
		}
		_ => {}
	}
	output
}
serein_extension_sdk::export!(handle);

#[cfg(test)]
mod tests {
	use super::*;
	use serein_extension_sdk::{Invocation, MessageEvent};

	fn message(id: &str, kind: MessageEventKind, content: &str) -> MessageEvent {
		MessageEvent {
			kind,
			channel_id: "1".into(),
			message_id: id.into(),
			author_id: None,
			content: Some(content.into()),
		}
	}

	fn run(kind: MessageEventKind, content: &str) -> Vec<String> {
		handle(DecorateInvocation {
			invocation: Invocation {
				action: "message-event".into(),
				..Default::default()
			},
			message_event: Some(message("2", kind, content)),
			..Default::default()
		})
		.decoration
		.map(|d| d.badges.into_iter().map(|b| b.text).collect())
		.unwrap_or_default()
	}

	#[test]
	fn badges_links_and_long_text_but_not_deletes_or_plain_text() {
		assert_eq!(
			run(MessageEventKind::Create, "see https://example.com"),
			["LINK"]
		);
		assert_eq!(run(MessageEventKind::Update, &"x".repeat(501)), ["LONG"]);
		assert!(run(MessageEventKind::Create, "hello").is_empty());
		assert!(run(MessageEventKind::Delete, "https://example.com").is_empty());
	}

	#[test]
	fn batches_decorate_only_matching_loaded_messages() {
		let output = handle(DecorateInvocation {
			invocation: Invocation {
				action: "decorate".into(),
				..Default::default()
			},
			messages: vec![
				message("10", MessageEventKind::Create, "plain"),
				message("11", MessageEventKind::Create, "https://example.com"),
			],
			..Default::default()
		});
		let ids: Vec<_> = output
			.decorations
			.iter()
			.map(|d| d.message_id.as_str())
			.collect();
		assert_eq!(ids, ["11"]);
	}
}
