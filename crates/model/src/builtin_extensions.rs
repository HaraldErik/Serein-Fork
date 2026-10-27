//! On/off state and options of the fork's built-in extensions.
//!
//! All of them are off until the user enables them. Built-in extensions are native features that the Wasm sandbox cannot provide (audio, composer
//! buttons, send rewrites). They are listed under Settings → Extensions → Built-in with their own
//! switch and Config, never on ordinary settings pages. Each one owns a field here.

/// Every built-in extension's saved state; older preferences fill in the defaults.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct BuiltinExtensions {
	pub volume_booster: VolumeBooster,
	pub clear_urls: ClearUrls,
	pub send_timestamps: SendTimestamps,
	pub silent_toggle: SilentToggle,
}

impl BuiltinExtensions {
	pub fn is_valid(&self) -> bool {
		(VolumeBooster::MIN_CEILING..=VolumeBooster::MAX_CEILING)
			.contains(&self.volume_booster.max_percent)
			&& SendTimestamps::STYLES.contains(&self.send_timestamps.style)
	}
}

/// Equicord VolumeBooster: participant and stream volume above 200%.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct VolumeBooster {
	pub enabled: bool,
	/// Highest value of the per-user and stream volume sliders.
	pub max_percent: u16,
}

impl VolumeBooster {
	/// Slider ceiling without the extension (Discord's own limit).
	pub const MIN_CEILING: u16 = 200;
	/// Must match `discord_voice::MAX_PLAYBACK_PERCENT`, which the mixer enforces.
	pub const MAX_CEILING: u16 = 400;

	/// The effective slider and mixer ceiling.
	pub fn ceiling(&self) -> u16 {
		if self.enabled {
			self.max_percent.clamp(Self::MIN_CEILING, Self::MAX_CEILING)
		} else {
			Self::MIN_CEILING
		}
	}
}

impl Default for VolumeBooster {
	fn default() -> Self {
		Self {
			enabled: false,
			max_percent: Self::MAX_CEILING,
		}
	}
}

/// Equicord ClearURLs: strip tracking parameters from links.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct ClearUrls {
	pub enabled: bool,
	/// Also apply site-specific rules (YouTube/Spotify `si`, X `s`/`t`, TikTok, Reddit).
	pub site_rules: bool,
	/// Clean links in edited messages too, not just new ones.
	pub on_edit: bool,
}

impl Default for ClearUrls {
	fn default() -> Self {
		Self {
			enabled: false,
			site_rules: true,
			on_edit: true,
		}
	}
}

/// Equicord SendTimestamps: `` `18:30` `` becomes `<t:…>`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct SendTimestamps {
	pub enabled: bool,
	/// Discord timestamp style letter.
	pub style: char,
	pub on_edit: bool,
}

impl SendTimestamps {
	/// Short time, long time, short date/time, long date/time, relative.
	pub const STYLES: [char; 5] = ['t', 'T', 'f', 'F', 'R'];
}

impl Default for SendTimestamps {
	fn default() -> Self {
		Self {
			enabled: false,
			style: 't',
			on_edit: true,
		}
	}
}

/// Equicord SilentMessageToggle: a bell beside Send for `@silent` messages (red while armed),
/// and a bell-with-z beside the time of every silent message.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct SilentToggle {
	pub enabled: bool,
	/// Switch the bell off again after one silent message.
	pub auto_disable: bool,
}

impl Default for SilentToggle {
	fn default() -> Self {
		Self {
			enabled: false,
			auto_disable: true,
		}
	}
}
