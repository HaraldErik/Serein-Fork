use serde::{Deserialize, Serialize};

pub const MAX_DECORATION_BADGES: usize = 4;
pub const MAX_BADGE_TEXT_CHARS: usize = 16;
pub const MAX_BADGE_TOOLTIP_BYTES: usize = 128;
pub const MAX_AFFIX_BYTES: usize = 64;
pub const MAX_DECORATE_BATCH: usize = 32;

/// Short label shown beside a message author.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Badge {
	pub text: String,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub tooltip: Option<String>,
	/// 0xRRGGBB
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub color: Option<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Presentation {
	Blur,
	Collapse,
	Hide,
}

/// Return from a `message_event` handler with the `decorations` capability.
/// Limits match the host: 4 badges, 16-character badge text, 64-byte prefix/suffix.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Decoration {
	#[serde(default)]
	pub badges: Vec<Badge>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub prefix: Option<String>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub suffix: Option<String>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub presentation: Option<Presentation>,
}

/// One message's decoration in a `decorate` result.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageDecoration {
	pub message_id: String,
	pub decoration: Decoration,
}

/// Opt-in output carrying decorations; keeps existing `Output` struct literals compiling.
/// `decoration` answers a `message_event`; `decorations` answers a `decorate` batch.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecorationOutput {
	#[serde(flatten)]
	pub output: crate::Output,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub decoration: Option<Decoration>,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub decorations: Vec<MessageDecoration>,
}

/// Input for decoration plugins. A `decorate` action receives up to 32 already-loaded
/// `messages`, newest first; a `message_event` action receives one live `message_event`.
/// One handler can serve both.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecorateInvocation {
	#[serde(flatten)]
	pub invocation: crate::Invocation,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub message_event: Option<crate::MessageEvent>,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub messages: Vec<crate::MessageEvent>,
}
