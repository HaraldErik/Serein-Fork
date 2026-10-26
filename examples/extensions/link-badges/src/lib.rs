use serein_extension_sdk::{
	Badge, Decoration, DecorationOutput, EventInvocation, MessageEventKind,
};

/// Adds a LINK badge to messages containing a URL and a LONG badge to very long ones.
/// Reads the event text only for this call; nothing is stored.
fn handle(input: EventInvocation) -> DecorationOutput {
	let mut output = DecorationOutput::default();
	let Some(event) = input.message_event else {
		return output;
	};
	if input.invocation.action != "message-event" || event.kind == MessageEventKind::Delete {
		return output;
	}
	let content = event.content.unwrap_or_default();
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
	if !badges.is_empty() {
		output.decoration = Some(Decoration {
			badges,
			..Default::default()
		});
	}
	output
}
serein_extension_sdk::export!(handle);

#[cfg(test)]
mod tests {
	use super::*;
	use serein_extension_sdk::{Invocation, MessageEvent};

	fn run(kind: MessageEventKind, content: &str) -> DecorationOutput {
		handle(EventInvocation {
			invocation: Invocation {
				action: "message-event".into(),
				..Default::default()
			},
			message_event: Some(MessageEvent {
				kind,
				channel_id: "1".into(),
				message_id: "2".into(),
				author_id: None,
				content: Some(content.into()),
			}),
		})
	}

	#[test]
	fn badges_links_and_long_text_but_not_deletes_or_plain_text() {
		let badges = |output: DecorationOutput| {
			output
				.decoration
				.map(|d| d.badges.into_iter().map(|b| b.text).collect::<Vec<_>>())
				.unwrap_or_default()
		};
		assert_eq!(
			badges(run(MessageEventKind::Create, "see https://example.com")),
			["LINK"]
		);
		assert_eq!(
			badges(run(MessageEventKind::Update, &"x".repeat(501))),
			["LONG"]
		);
		assert!(badges(run(MessageEventKind::Create, "hello")).is_empty());
		assert!(badges(run(MessageEventKind::Delete, "https://example.com")).is_empty());
	}
}
