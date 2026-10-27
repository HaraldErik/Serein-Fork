//! Settings → Extensions → Built-in: the fork's native extensions, each with its own
//! on/off switch and Config. New ported features belong here (or in a Wasm plugin),
//! never on ordinary settings pages.

use crate::design;
use model::builtin_extensions::{SendTimestamps, VolumeBooster};

/// Draws every built-in extension; `open` remembers which Config is expanded.
pub(crate) fn section(
	ui: &mut egui::Ui,
	value: &mut model::BuiltinExtensions,
	open: &mut Option<&'static str>,
) {
	design::group(ui, "builtin-extensions-title", |ui| {
		entry(
			ui,
			open,
			"volume-booster",
			"builtin-volume-booster",
			"builtin-volume-booster-description",
			&mut value.volume_booster.enabled,
			|ui| {
				let ceilings = [200, 300, VolumeBooster::MAX_CEILING];
				let labels = ["200%", "300%", "400%"];
				let current = ceilings
					.iter()
					.position(|c| *c == value.volume_booster.max_percent)
					.unwrap_or(2);
				ui.label(design::eyebrow(
					ui,
					crate::i18n::translate("builtin-volume-booster-max"),
					design::palette(ui).muted,
				));
				if let Some(index) = design::segmented(ui, &labels, current) {
					value.volume_booster.max_percent = ceilings[index];
				}
				design::hint(
					ui,
					&crate::i18n::translate("builtin-volume-booster-max-hint"),
				);
			},
		);
		design::card_divider(ui);
		entry(
			ui,
			open,
			"clear-urls",
			"builtin-clear-urls",
			"builtin-clear-urls-description",
			&mut value.clear_urls.enabled,
			|ui| {
				design::switch(
					ui,
					"builtin-clear-urls-site-rules",
					Some("builtin-clear-urls-site-rules-description"),
					&mut value.clear_urls.site_rules,
				);
				design::switch(
					ui,
					"builtin-also-on-edits",
					None,
					&mut value.clear_urls.on_edit,
				);
			},
		);
		design::card_divider(ui);
		entry(
			ui,
			open,
			"send-timestamps",
			"builtin-send-timestamps",
			"builtin-send-timestamps-description",
			&mut value.send_timestamps.enabled,
			|ui| {
				let labels = ["18:30", "18:30:00", "Date + time", "Full date", "Relative"];
				let current = SendTimestamps::STYLES
					.iter()
					.position(|s| *s == value.send_timestamps.style)
					.unwrap_or(0);
				ui.label(design::eyebrow(
					ui,
					crate::i18n::translate("builtin-send-timestamps-style"),
					design::palette(ui).muted,
				));
				if let Some(index) = design::segmented(ui, &labels, current) {
					value.send_timestamps.style = SendTimestamps::STYLES[index];
				}
				design::switch(
					ui,
					"builtin-also-on-edits",
					None,
					&mut value.send_timestamps.on_edit,
				);
			},
		);
		design::card_divider(ui);
		entry(
			ui,
			open,
			"silent-toggle",
			"builtin-silent-toggle",
			"builtin-silent-toggle-description",
			&mut value.silent_toggle.enabled,
			|ui| {
				design::switch(
					ui,
					"builtin-silent-toggle-auto-disable",
					Some("builtin-silent-toggle-auto-disable-description"),
					&mut value.silent_toggle.auto_disable,
				);
			},
		);
	});
	design::hint(ui, &crate::i18n::translate("builtin-extensions-hint"));
}

/// One extension row: its switch, then a Config disclosure that expands the options.
fn entry(
	ui: &mut egui::Ui,
	open: &mut Option<&'static str>,
	id: &'static str,
	name: &str,
	description: &str,
	enabled: &mut bool,
	config: impl FnOnce(&mut egui::Ui),
) {
	ui.push_id(id, |ui| {
		design::switch(ui, name, Some(description), enabled);
		let expanded = *open == Some(id);
		if design::disclosure(ui, "builtin-extensions-config", expanded).clicked() {
			*open = if expanded { None } else { Some(id) };
		}
		if expanded {
			ui.add_enabled_ui(*enabled, |ui| {
				ui.indent(id, config);
			});
		}
	});
}
