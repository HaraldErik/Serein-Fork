use serde::{Deserialize, Serialize};

pub const MAX_DECORATION_BADGES: usize = 4;
pub const MAX_BADGE_TEXT_CHARS: usize = 16;
pub const MAX_BADGE_TOOLTIP_BYTES: usize = 128;
pub const MAX_AFFIX_BYTES: usize = 64;

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

/// Opt-in output carrying a decoration; keeps existing `Output` struct literals compiling.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecorationOutput {
	#[serde(flatten)]
	pub output: crate::Output,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub decoration: Option<Decoration>,
}
