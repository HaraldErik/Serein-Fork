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

/// Least-recently-used cache bounded by item count and bytes, keyed by Discord ID (snowflake)
/// and plugin ID. Lookups are O(plugins on that target), so the UI can call `get` every frame.
#[derive(Debug, Default)]
pub struct DecorationCache {
	entries: HashMap<u64, BTreeMap<String, (Decoration, u64)>>,
	order: BTreeMap<u64, (u64, String)>,
	clock: u64,
	items: usize,
	bytes: usize,
}

fn cost(decoration: &Decoration, plugin: &str) -> usize {
	decoration.bytes() + plugin.len() + 16
}

impl DecorationCache {
	/// Validates then stores a decoration, evicting the oldest entries to stay within bounds.
	pub fn insert(
		&mut self,
		plugin: &str,
		target: u64,
		decoration: Decoration,
	) -> Result<(), Error> {
		decoration.validate()?;
		if plugin.is_empty() || plugin.len() > 128 || target == 0 {
			return Err(Error::Invalid);
		}
		let size = cost(&decoration, plugin);
		if size > MAX_CACHE_BYTES {
			return Err(Error::Limit);
		}
		self.remove(plugin, target);
		while self.items >= MAX_CACHE_ITEMS || self.bytes + size > MAX_CACHE_BYTES {
			let Some((_, (old_target, old_plugin))) = self.order.pop_first() else {
				break;
			};
			self.take(old_plugin.as_str(), old_target, false);
		}
		self.clock += 1;
		self.order.insert(self.clock, (target, plugin.to_owned()));
		self.entries
			.entry(target)
			.or_default()
			.insert(plugin.to_owned(), (decoration, self.clock));
		self.items += 1;
		self.bytes += size;
		Ok(())
	}

	/// Decorations for one target in ascending plugin-ID order. Does not refresh recency.
	pub fn get(&self, target: u64) -> impl Iterator<Item = (&str, &Decoration)> {
		self.entries
			.get(&target)
			.into_iter()
			.flatten()
			.map(|(plugin, (decoration, _))| (plugin.as_str(), decoration))
	}

	/// Drops everything from one plugin (disable, update or logout).
	pub fn remove_plugin(&mut self, plugin: &str) {
		let targets: Vec<u64> = self
			.entries
			.iter()
			.filter(|(_, plugins)| plugins.contains_key(plugin))
			.map(|(target, _)| *target)
			.collect();
		for target in targets {
			self.remove(plugin, target);
		}
	}

	pub fn clear(&mut self) {
		*self = Self::default();
	}

	pub fn len(&self) -> usize {
		self.items
	}

	pub fn is_empty(&self) -> bool {
		self.items == 0
	}

	fn remove(&mut self, plugin: &str, target: u64) {
		self.take(plugin, target, true);
	}

	fn take(&mut self, plugin: &str, target: u64, forget_order: bool) {
		let Some(plugins) = self.entries.get_mut(&target) else {
			return;
		};
		if let Some((old, stamp)) = plugins.remove(plugin) {
			if forget_order {
				self.order.remove(&stamp);
			}
			self.items -= 1;
			self.bytes -= cost(&old, plugin);
		}
		if plugins.is_empty() {
			self.entries.remove(&target);
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
				.insert("plugin", index as u64 + 1, badge("b"))
				.unwrap();
		}
		assert_eq!(cache.len(), MAX_CACHE_ITEMS);
		assert_eq!(cache.get(1).count(), 0);
		assert_eq!(cache.get(MAX_CACHE_ITEMS as u64 + 10).count(), 1);
	}

	#[test]
	fn cache_byte_accounting_survives_replace_and_plugin_removal() {
		let mut cache = DecorationCache::default();
		cache.insert("a", 1, badge("one")).unwrap();
		cache.insert("a", 1, badge("two")).unwrap();
		cache.insert("b", 1, badge("three")).unwrap();
		assert!(cache.insert("a", 0, badge("zero")).is_err());
		assert_eq!(cache.len(), 2);
		assert_eq!(cache.get(1).map(|(plugin, _)| plugin).next(), Some("a"));
		cache.remove_plugin("a");
		assert_eq!(cache.len(), 1);
		cache.remove_plugin("b");
		assert!(cache.is_empty() && cache.bytes == 0 && cache.order.is_empty());
		assert!(cache.entries.is_empty());
	}
}
