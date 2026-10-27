//! Paints plugin badges from the bounded decoration cache; never calls a plugin.
use egui::{Color32, RichText};
use extensions::{DecorationCache, Presentation};
use std::sync::Mutex;

pub(crate) type SharedCache = std::sync::Arc<Mutex<DecorationCache>>;

/// Badges, then the optional text before/after, for one message. Costs one map lookup.
pub(crate) fn paint(ui: &mut egui::Ui, cache: &SharedCache, message: u64) {
	let Ok(cache) = cache.lock() else { return };
	for (_, decoration) in cache.get(message) {
		if let Some(prefix) = &decoration.prefix {
			ui.label(RichText::new(prefix).small().weak());
		}
		for badge in &decoration.badges {
			let color = badge.color.map_or(ui.visuals().weak_text_color(), |rgb| {
				Color32::from_rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
			});
			let response = ui.label(RichText::new(&badge.text).small().strong().color(color));
			if let Some(tooltip) = &badge.tooltip {
				response.on_hover_text(tooltip);
			}
		}
		if let Some(suffix) = &decoration.suffix {
			ui.label(RichText::new(suffix).small().weak());
		}
	}
}

/// Placeholder text when any plugin conceals this message; `None` shows it normally.
pub(crate) fn concealed(cache: &SharedCache, message: u64) -> Option<&'static str> {
	let cache = cache.lock().ok()?;
	let strongest = cache
		.get(message)
		.filter_map(|(_, decoration)| decoration.presentation)
		.max_by_key(|presentation| match presentation {
			Presentation::Blur => 0,
			Presentation::Collapse => 1,
			Presentation::Hide => 2,
		})?;
	Some(match strongest {
		Presentation::Blur => "Blurred by a plugin. Click to show.",
		Presentation::Collapse => "Collapsed by a plugin. Click to show.",
		Presentation::Hide => "Hidden by a plugin. Click to show.",
	})
}
