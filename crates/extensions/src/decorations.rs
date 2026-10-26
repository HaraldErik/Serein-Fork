//! Bounded plugin decorations for messages, users and servers (plugin API v2 core).
//!
//! The UI only reads `DecorationCache`; plugins are never called from rendering.
use crate::Error;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

pub const MAX_DECORATION_BADGES: usize = 4;
pub const MAX_BADGE_TEXT_CHARS: usize = 16;
pub const MAX_BADGE_TOOLTIP_BYTES: usize = 128;
pub const MAX_AFFIX_BYTES: usize = 64;
pub const MAX_CACHE_ITEMS: usize = 2048;
pub const MAX_CACHE_BYTES: usize = 4 * 1024 * 1024;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Badge {
	pub text: String,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub tooltip: Option<String>,
	/// 0xRRGGBB
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub color: Option<u32>,
}

/// How the host presents the decorated item's content.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Presentation {
	Blur,
	Collapse,
	Hide,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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

fn plain(text: &str, max_bytes: usize) -> bool {
	!text.is_empty() && text.len() <= max_bytes && !text.chars().any(char::is_control)
}

impl Decoration {
	pub fn validate(&self) -> Result<(), Error> {
		if self.badges.len() > MAX_DECORATION_BADGES {
			return Err(Error::Limit);
		}
		for badge in &self.badges {
			if !plain(&badge.text, MAX_BADGE_TEXT_CHARS * 4)
				|| badge.text.chars().count() > MAX_BADGE_TEXT_CHARS
				|| badge.color.is_some_and(|color| color > 0xFF_FFFF)
				|| badge
					.tooltip
					.as_deref()
					.is_some_and(|tip| !plain(tip, MAX_BADGE_TOOLTIP_BYTES))
			{
				return Err(Error::Invalid);
			}
		}
		for affix in [&self.prefix, &self.suffix].into_iter().flatten() {
			if !plain(affix, MAX_AFFIX_BYTES) {
				return Err(Error::Invalid);
			}
		}
		Ok(())
	}

	fn bytes(&self) -> usize {
		let badges: usize = self
			.badges
			.iter()
			.map(|badge| badge.text.len() + badge.tooltip.as_ref().map_or(0, String::len) + 8)
			.sum();
		48 + badges
			+ self.prefix.as_ref().map_or(0, String::len)
			+ self.suffix.as_ref().map_or(0, String::len)
	}
}

/// Least-recently-used cache bounded by item count and bytes, keyed by plugin and target ID.
#[derive(Debug, Default)]
pub struct DecorationCache {
	entries: HashMap<(String, String), (Decoration, u64)>,
	order: BTreeMap<u64, (String, String)>,
	clock: u64,
	bytes: usize,
}

impl DecorationCache {
	/// Validates then stores a decoration, evicting the oldest entries to stay within bounds.
	pub fn insert(
		&mut self,
		plugin: &str,
		target: &str,
		decoration: Decoration,
	) -> Result<(), Error> {
		decoration.validate()?;
		if plugin.is_empty() || plugin.len() > 128 || target.is_empty() || target.len() > 32 {
			return Err(Error::Invalid);
		}
		let key = (plugin.to_owned(), target.to_owned());
		self.remove(&key);
		let size = decoration.bytes() + key.0.len() + key.1.len();
		if size > MAX_CACHE_BYTES {
			return Err(Error::Limit);
		}
		while self.entries.len() >= MAX_CACHE_ITEMS || self.bytes + size > MAX_CACHE_BYTES {
			let Some((_, oldest)) = self.order.pop_first() else {
				break;
			};
			if let Some((old, _)) = self.entries.remove(&oldest) {
				self.bytes -= old.bytes() + oldest.0.len() + oldest.1.len();
			}
		}
		self.clock += 1;
		self.order.insert(self.clock, key.clone());
		self.bytes += size;
		self.entries.insert(key, (decoration, self.clock));
		Ok(())
	}

	/// All decorations for one target, in ascending plugin-ID order. Does not refresh recency.
	pub fn get<'a>(&'a self, target: &'a str) -> impl Iterator<Item = (&'a str, &'a Decoration)> {
		self.entries
			.iter()
			.filter(move |((_, id), _)| id == target)
			.map(|((plugin, _), (decoration, _))| (plugin.as_str(), decoration))
	}

	/// Drops everything from one plugin (disable, update or logout).
	pub fn remove_plugin(&mut self, plugin: &str) {
		let keys: Vec<_> = self
			.entries
			.keys()
			.filter(|key| key.0 == plugin)
			.cloned()
			.collect();
		for key in keys {
			self.remove(&key);
		}
	}

	pub fn clear(&mut self) {
		*self = Self::default();
	}

	pub fn len(&self) -> usize {
		self.entries.len()
	}

	pub fn is_empty(&self) -> bool {
		self.entries.is_empty()
	}

	fn remove(&mut self, key: &(String, String)) {
		if let Some((old, stamp)) = self.entries.remove(key) {
			self.order.remove(&stamp);
			self.bytes -= old.bytes() + key.0.len() + key.1.len();
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn badge(text: &str) -> Decoration {
		Decoration {
			badges: vec![Badge {
				text: text.into(),
				..Default::default()
			}],
			..Default::default()
		}
	}

	#[test]
	fn malformed_decorations_are_rejected() {
		assert!(badge("ok").validate().is_ok());
		assert!(badge("").validate().is_err());
		assert!(badge("bad\ncontrol").validate().is_err());
		assert!(
			badge(&"x".repeat(MAX_BADGE_TEXT_CHARS + 1))
				.validate()
				.is_err()
		);
		let mut many = badge("a");
		many.badges = vec![many.badges[0].clone(); MAX_DECORATION_BADGES + 1];
		assert!(many.validate().is_err());
		let mut color = badge("a");
		color.badges[0].color = Some(0x100_0000);
		assert!(color.validate().is_err());
		let affix = Decoration {
			prefix: Some("x".repeat(MAX_AFFIX_BYTES + 1)),
			..Default::default()
		};
		assert!(affix.validate().is_err());
		assert!(
			serde_json::from_str::<Decoration>(r#"{"badges":[],"script":"x"}"#).is_err(),
			"unknown fields must be refused"
		);
	}

	#[test]
	fn cache_stays_within_item_bound_and_evicts_oldest() {
		let mut cache = DecorationCache::default();
		for index in 0..MAX_CACHE_ITEMS + 10 {
			cache
				.insert("plugin", &index.to_string(), badge("b"))
				.unwrap();
		}
		assert_eq!(cache.len(), MAX_CACHE_ITEMS);
		assert_eq!(cache.get("0").count(), 0);
		assert_eq!(cache.get(&(MAX_CACHE_ITEMS + 9).to_string()).count(), 1);
	}

	#[test]
	fn cache_byte_accounting_survives_replace_and_plugin_removal() {
		let mut cache = DecorationCache::default();
		cache.insert("a", "1", badge("one")).unwrap();
		cache.insert("a", "1", badge("two")).unwrap();
		cache.insert("b", "1", badge("three")).unwrap();
		assert_eq!(cache.len(), 2);
		assert_eq!(cache.get("1").map(|(plugin, _)| plugin).min(), Some("a"));
		cache.remove_plugin("a");
		assert_eq!(cache.len(), 1);
		cache.remove_plugin("b");
		assert!(cache.is_empty() && cache.bytes == 0 && cache.order.is_empty());
	}
}
