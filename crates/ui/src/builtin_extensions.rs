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
			Some(&mut |ui: &mut egui::Ui| {
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
			}),
		);
		design::card_divider(ui);
		entry(
			ui,
			open,
			"clear-urls",
			"builtin-clear-urls",
			"builtin-clear-urls-description",
			&mut value.clear_urls.enabled,
			Some(&mut |ui: &mut egui::Ui| {
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
			}),
		);
		design::card_divider(ui);
		entry(
			ui,
			open,
			"send-timestamps",
			"builtin-send-timestamps",
			"builtin-send-timestamps-description",
			&mut value.send_timestamps.enabled,
			Some(&mut |ui: &mut egui::Ui| {
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
			}),
		);
		design::card_divider(ui);
		entry(
			ui,
			open,
			"silent-toggle",
			"builtin-silent-toggle",
			"builtin-silent-toggle-description",
			&mut value.silent_toggle.enabled,
			Some(&mut |ui: &mut egui::Ui| {
				design::switch(
					ui,
					"builtin-silent-toggle-auto-disable",
					Some("builtin-silent-toggle-auto-disable-description"),
					&mut value.silent_toggle.auto_disable,
				);
			}),
		);
		design::card_divider(ui);
		entry(
			ui,
			open,
			"always-expand-profiles",
			"builtin-always-expand-profiles",
			"builtin-always-expand-profiles-description",
			&mut value.always_expand_profiles.enabled,
			None,
		);
		design::card_divider(ui);
		entry(
			ui,
			open,
			"always-expand-roles",
			"builtin-always-expand-roles",
			"builtin-always-expand-roles-description",
			&mut value.always_expand_roles.enabled,
			None,
		);
		design::card_divider(ui);
		let folders = &mut value.better_folders;
		entry(
			ui,
			open,
			"better-folders",
			"builtin-better-folders",
			"builtin-better-folders-description",
			&mut folders.enabled,
			Some(&mut |ui: &mut egui::Ui| {
				for (label, detail, value) in [
					(
						"builtin-better-folders-sidebar",
						"builtin-better-folders-sidebar-description",
						&mut folders.sidebar,
					),
					(
						"builtin-better-folders-sidebar-anim",
						"builtin-better-folders-sidebar-anim-description",
						&mut folders.sidebar_anim,
					),
					(
						"builtin-better-folders-close-all",
						"builtin-better-folders-close-all-description",
						&mut folders.close_all_folders,
					),
					(
						"builtin-better-folders-close-home",
						"builtin-better-folders-close-home-description",
						&mut folders.close_all_home,
					),
					(
						"builtin-better-folders-close-others",
						"builtin-better-folders-close-others-description",
						&mut folders.close_others,
					),
					(
						"builtin-better-folders-close-server-folder",
						"builtin-better-folders-close-server-folder-description",
						&mut folders.close_server_folder,
					),
					(
						"builtin-better-folders-force-open",
						"builtin-better-folders-force-open-description",
						&mut folders.force_open,
					),
					(
						"builtin-better-folders-keep-icons",
						"builtin-better-folders-keep-icons-description",
						&mut folders.keep_icons,
					),
				] {
					design::switch(ui, label, Some(detail), value);
				}
				ui.label(design::eyebrow(
					ui,
					crate::i18n::translate("builtin-better-folders-show-icon"),
					design::palette(ui).muted,
				));
				let labels = [
					crate::i18n::translate("builtin-better-folders-icon-never"),
					crate::i18n::translate("builtin-better-folders-icon-always"),
					crate::i18n::translate("builtin-better-folders-icon-several"),
				];
				let labels: Vec<&str> = labels.iter().map(String::as_str).collect();
				if let Some(index) =
					design::segmented(ui, &labels, usize::from(folders.show_folder_icon.min(2)))
				{
					folders.show_folder_icon = index as u8;
				}
				design::hint(
					ui,
					&crate::i18n::translate("builtin-better-folders-show-icon-hint"),
				);
			}),
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
	config: Option<&mut dyn FnMut(&mut egui::Ui)>,
) {
	ui.push_id(id, |ui| {
		design::switch(ui, name, Some(description), enabled);
		let Some(config) = config else {
			return;
		};
		let expanded = *open == Some(id);
		if design::disclosure(ui, "builtin-extensions-config", expanded).clicked() {
			*open = if expanded { None } else { Some(id) };
		}
		if expanded {
			ui.add_enabled_ui(*enabled, |ui| {
				ui.indent(id, |ui| config(ui));
			});
		}
	});
}
