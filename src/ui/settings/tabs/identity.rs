use crate::identity::{self, Identity};
use crate::theme as t;
use crate::ui::settings::widgets::{group_label, id_button, page_header, s_row, toggle};
use crate::ui::state::UiState;
use eframe::egui::{self, Color32, FontId, Frame, Margin, RichText, Stroke, Vec2};

pub fn show(ui: &mut egui::Ui, identity: &Identity, s: &mut UiState) {
    page_header(ui, "identity", "your local keypair · no account required");
    Frame::new()
        .fill(t::BG_PANEL)
        .stroke(Stroke::new(1.0, t::BORDER))
        .corner_radius(7u8)
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let init: String = identity
                    .nickname
                    .chars()
                    .take(2)
                    .collect::<String>()
                    .to_uppercase();
                let av = egui::Button::new(
                    RichText::new(&init)
                        .font(FontId::monospace(11.0))
                        .color(t::ACCENT),
                )
                .fill(Color32::from_rgba_premultiplied(255, 107, 53, 18))
                .stroke(Stroke::new(
                    1.0,
                    Color32::from_rgba_premultiplied(255, 107, 53, 38),
                ))
                .corner_radius(8u8)
                .min_size(Vec2::splat(36.0));
                ui.add_enabled(false, av);
                ui.add_space(6.0);
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(&identity.nickname)
                            .font(FontId::monospace(12.0))
                            .color(t::TEXT_SECONDARY)
                            .strong(),
                    );
                    ui.label(
                        RichText::new(format!("pub: {}…", &identity.pubkey_hex[..16]))
                            .font(FontId::monospace(9.0))
                            .color(t::TEXT_GHOST),
                    );
                });
            });
            ui.add_space(10.0);
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 5.0;
                if id_button(ui, "copy pubkey") {
                    ui.ctx().copy_text(identity.pubkey_hex.clone());
                }
                if id_button(ui, "set passphrase") {
                    s.vault_set_open = true;
                    s.vault_passphrase_input.clear();
                    s.vault_passphrase_confirm.clear();
                    s.vault_error = None;
                }
                if id_button(ui, "remove passphrase") {
                    s.vault_remove_open = true;
                    s.vault_passphrase_input.clear();
                    s.vault_error = None;
                }
                if id_button(ui, "export keyfile") {
                    s.export_keyfile_open = true;
                    s.export_keyfile_pass.clear();
                    s.export_keyfile_confirm.clear();
                    s.export_keyfile_output.clear();
                    s.export_keyfile_error = None;
                }
                if id_button(ui, "import keyfile") {
                    s.import_keyfile_open = true;
                    s.import_keyfile_input.clear();
                    s.import_keyfile_pass.clear();
                    s.import_keyfile_error = None;
                }
                id_button(ui, "change nickname");
                id_button(ui, "rotate keypair");
            });
        });

    // Vault status block.
    ui.add_space(10.0);
    group_label(ui, "vault status");
    let is_encrypted = identity::is_vault_encrypted();
    Frame::new()
        .fill(if is_encrypted { t::BG_PANEL } else { t::BG_ELEVATED })
        .stroke(Stroke::new(
            1.0,
            if is_encrypted { t::SUCCESS } else { t::BORDER },
        ))
        .corner_radius(7u8)
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let (label, color) = if is_encrypted {
                    ("encrypted (Argon2id + ChaCha20-Poly1305)", t::SUCCESS)
                } else {
                    ("plain JSON — passphrase not set", t::WARNING)
                };
                // Drawn lock/unlock icon.
                let (icon_rect, _) = ui.allocate_exact_size(
                    egui::vec2(18.0, 18.0),
                    egui::Sense::hover(),
                );
                let icon_fn = if is_encrypted {
                    crate::theme::icons::lock
                } else {
                    crate::theme::icons::unlock
                };
                icon_fn(ui.painter(), icon_rect, color);
                ui.label(
                    RichText::new(label)
                        .font(FontId::proportional(12.0))
                        .color(color)
                        .strong(),
                );
            });
            ui.add_space(4.0);
            ui.label(
                RichText::new(if is_encrypted {
                    "Your private key is encrypted at rest. The vault is unlocked in memory while VNOX is running."
                } else {
                    "Anyone with filesystem access can read your private key. Set a passphrase to encrypt it."
                })
                .font(FontId::proportional(11.0))
                .color(t::TEXT_MUTED),
            );
        });

    ui.add_space(10.0);
    group_label(ui, "backup");
    s_row(
        ui,
        "seed phrase backup",
        Some("24-word recovery · never share"),
        |ui| {
            let mut v = false;
            toggle(ui, &mut v);
        },
    );
    s_row(
        ui,
        "export to file",
        Some(".vnox keyfile · encrypted"),
        |ui| {
            let _ = id_button(ui, "export");
        },
    );

    ui.add_space(8.0);
    group_label(ui, "permissions");
    s_row(
        ui,
        "show identity to strangers",
        Some("pubkey visible on federated nodes"),
        |ui| {
            let mut v = true;
            toggle(ui, &mut v);
        },
    );

    // Set-passphrase modal.
    if s.vault_set_open {
        show_set_passphrase_modal(ui.ctx(), identity, s);
    }
    // Remove-passphrase modal.
    if s.vault_remove_open {
        show_remove_passphrase_modal(ui.ctx(), identity, s);
    }
    // Export keyfile modal.
    if s.export_keyfile_open {
        show_export_keyfile_modal(ui.ctx(), identity, s);
    }
    // Import keyfile modal.
    if s.import_keyfile_open {
        show_import_keyfile_modal(ui.ctx(), s);
    }
}

fn show_set_passphrase_modal(ctx: &egui::Context, identity: &Identity, s: &mut UiState) {
    egui::Window::new("Set Identity Passphrase")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .frame(
            Frame::window(&ctx.style())
                .fill(t::BG_ELEVATED)
                .stroke(egui::Stroke::new(1.0, t::BORDER_DEFAULT))
                .corner_radius(t::R_LG),
        )
        .show(ctx, |ui| {
            ui.set_width(360.0);
            ui.label(
                RichText::new("Pick a strong passphrase. It will be used to encrypt your identity keypair at rest (Argon2id + ChaCha20-Poly1305).")
                    .font(FontId::proportional(11.0))
                    .color(t::TEXT_MUTED),
            );
            ui.add_space(4.0);
            ui.label(
                RichText::new("! There is no recovery — if you forget the passphrase, the keypair is lost.")
                    .font(FontId::proportional(11.0))
                    .color(t::WARNING),
            );
            ui.add_space(10.0);
            ui.label(RichText::new("Passphrase").font(FontId::proportional(12.0)).color(t::TEXT_PRIMARY));
            let p1 = ui.add(
                egui::TextEdit::singleline(&mut s.vault_passphrase_input)
                    .password(true)
                    .font(FontId::monospace(12.0))
                    .desired_width(320.0),
            );
            ui.add_space(6.0);
            ui.label(RichText::new("Confirm").font(FontId::proportional(12.0)).color(t::TEXT_PRIMARY));
            let p2 = ui.add(
                egui::TextEdit::singleline(&mut s.vault_passphrase_confirm)
                    .password(true)
                    .font(FontId::monospace(12.0))
                    .desired_width(320.0),
            );
            ui.add_space(10.0);
            let mismatch = !s.vault_passphrase_input.is_empty()
                && s.vault_passphrase_input != s.vault_passphrase_confirm;
            if mismatch {
                ui.label(
                    RichText::new("passphrases do not match")
                        .font(FontId::proportional(11.0))
                        .color(t::ERROR),
                );
            }
            if s.vault_passphrase_input.len() < 8 && !s.vault_passphrase_input.is_empty() {
                ui.label(
                    RichText::new("! use at least 8 characters")
                        .font(FontId::proportional(11.0))
                        .color(t::WARNING),
                );
            }
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui
                    .add(
                        egui::Button::new(
                            RichText::new("Cancel")
                                .font(FontId::proportional(12.0))
                                .color(t::TEXT_MUTED),
                        )
                        .fill(egui::Color32::TRANSPARENT)
                        .frame(false),
                    )
                    .clicked()
                {
                    s.vault_set_open = false;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let can_apply = !s.vault_passphrase_input.is_empty()
                        && s.vault_passphrase_input == s.vault_passphrase_confirm
                        && s.vault_passphrase_input.len() >= 8;
                    let btn = egui::Button::new(
                        RichText::new("Encrypt & Save")
                            .font(FontId::proportional(12.0))
                            .color(egui::Color32::WHITE),
                    )
                    .fill(t::ACCENT)
                    .stroke(egui::Stroke::NONE)
                    .corner_radius(t::R_SM)
                    .min_size(egui::vec2(120.0, 28.0));
                    if ui.add_enabled(can_apply, btn).clicked() {
                        match identity::save(identity, Some(&s.vault_passphrase_input)) {
                            Ok(()) => {
                                s.vault_set_open = false;
                                s.vault_passphrase_input.clear();
                                s.vault_passphrase_confirm.clear();
                            }
                            Err(e) => {
                                s.vault_error = Some(format!("save failed: {e}"));
                            }
                        }
                    }
                });
            });
            if let Some(ref err) = s.vault_error {
                ui.add_space(4.0);
                ui.label(RichText::new(err).font(FontId::proportional(11.0)).color(t::ERROR));
            }
            if (p1.lost_focus() || p2.lost_focus()) && ui.input(|i| i.key_pressed(egui::Key::Escape))
            {
                s.vault_set_open = false;
            }
        });
}

fn show_remove_passphrase_modal(ctx: &egui::Context, identity: &Identity, s: &mut UiState) {
    egui::Window::new("Remove Passphrase")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .frame(
            Frame::window(&ctx.style())
                .fill(t::BG_ELEVATED)
                .stroke(egui::Stroke::new(1.0, t::BORDER_DEFAULT))
                .corner_radius(t::R_LG),
        )
        .show(ctx, |ui| {
            ui.set_width(360.0);
            ui.label(
                RichText::new("! This will store your identity as plain JSON on disk. Anyone with filesystem access will be able to read your private key.")
                    .font(FontId::proportional(11.0))
                    .color(t::WARNING),
            );
            ui.add_space(8.0);
            ui.label(
                RichText::new("Type REMOVE to confirm.")
                    .font(FontId::proportional(12.0))
                    .color(t::TEXT_PRIMARY),
            );
            ui.add_space(4.0);
            let resp = ui.add(
                egui::TextEdit::singleline(&mut s.vault_passphrase_input)
                    .font(FontId::monospace(12.0))
                    .desired_width(320.0),
            );
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if ui
                    .add(
                        egui::Button::new(
                            RichText::new("Cancel")
                                .font(FontId::proportional(12.0))
                                .color(t::TEXT_MUTED),
                        )
                        .fill(egui::Color32::TRANSPARENT)
                        .frame(false),
                    )
                    .clicked()
                {
                    s.vault_remove_open = false;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let can_apply = s.vault_passphrase_input == "REMOVE";
                    let btn = egui::Button::new(
                        RichText::new("Remove Passphrase")
                            .font(FontId::proportional(12.0))
                            .color(egui::Color32::WHITE),
                    )
                    .fill(t::ERROR)
                    .stroke(egui::Stroke::NONE)
                    .corner_radius(t::R_SM)
                    .min_size(egui::vec2(140.0, 28.0));
                    if ui.add_enabled(can_apply, btn).clicked() {
                        match identity::save(identity, None) {
                            Ok(()) => {
                                s.vault_remove_open = false;
                                s.vault_passphrase_input.clear();
                            }
                            Err(e) => {
                                s.vault_error = Some(format!("save failed: {e}"));
                            }
                        }
                    }
                });
            });
            if let Some(ref err) = s.vault_error {
                ui.add_space(4.0);
                ui.label(RichText::new(err).font(FontId::proportional(11.0)).color(t::ERROR));
            }
            if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                s.vault_remove_open = false;
            }
        });
}

fn show_export_keyfile_modal(ctx: &egui::Context, identity: &Identity, s: &mut UiState) {
    egui::Window::new("Export Identity Keyfile")
        .collapsible(false)
        .resizable(true)
        .default_size([440.0, 380.0])
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .frame(
            Frame::window(&ctx.style())
                .fill(t::BG_ELEVATED)
                .stroke(egui::Stroke::new(1.0, t::BORDER_DEFAULT))
                .corner_radius(t::R_LG),
        )
        .show(ctx, |ui| {
            ui.set_width(420.0);
            ui.label(
                RichText::new("Pick a passphrase to encrypt the keyfile. Anyone with the keyfile and the passphrase can impersonate you — share carefully.")
                    .font(FontId::proportional(11.0))
                    .color(t::TEXT_MUTED),
            );
            ui.add_space(8.0);
            ui.label(RichText::new("Passphrase (optional — empty = plain JSON)").font(FontId::proportional(12.0)).color(t::TEXT_PRIMARY));
            ui.add(
                egui::TextEdit::singleline(&mut s.export_keyfile_pass)
                    .password(true)
                    .font(FontId::monospace(12.0))
                    .desired_width(380.0),
            );
            ui.add_space(6.0);
            ui.label(RichText::new("Confirm").font(FontId::proportional(12.0)).color(t::TEXT_PRIMARY));
            ui.add(
                egui::TextEdit::singleline(&mut s.export_keyfile_confirm)
                    .password(true)
                    .font(FontId::monospace(12.0))
                    .desired_width(380.0),
            );
            let mismatch = !s.export_keyfile_pass.is_empty()
                && s.export_keyfile_pass != s.export_keyfile_confirm;
            if mismatch {
                ui.label(
                    RichText::new("passphrases do not match")
                        .font(FontId::proportional(11.0))
                        .color(t::ERROR),
                );
            }
            ui.add_space(8.0);
            let can_generate = !mismatch;
            let gen_btn = egui::Button::new(
                RichText::new("Generate keyfile")
                    .font(FontId::proportional(12.0))
                    .color(egui::Color32::WHITE),
            )
            .fill(t::ACCENT)
            .stroke(egui::Stroke::NONE)
            .corner_radius(t::R_SM)
            .min_size(egui::vec2(120.0, 28.0));
            if ui.add_enabled(can_generate, gen_btn).clicked() {
                let pass = if s.export_keyfile_pass.is_empty() {
                    None
                } else {
                    Some(s.export_keyfile_pass.as_str())
                };
                match identity::export_keyfile(identity, pass) {
                    Ok(text) => s.export_keyfile_output = text,
                    Err(e) => s.export_keyfile_error = Some(format!("export failed: {e}")),
                }
            }
            if let Some(ref err) = s.export_keyfile_error {
                ui.label(RichText::new(err).font(FontId::proportional(11.0)).color(t::ERROR));
            }
            ui.add_space(6.0);
            if !s.export_keyfile_output.is_empty() {
                ui.label(
                    RichText::new("Keyfile (copy and save to a .vnoxkey file):")
                        .font(FontId::proportional(11.0))
                        .color(t::TEXT_MUTED),
                );
                egui::ScrollArea::vertical()
                    .max_height(160.0)
                    .show(ui, |ui| {
                        let mut buf = s.export_keyfile_output.clone();
                        ui.add(
                            egui::TextEdit::multiline(&mut buf)
                                .font(FontId::monospace(10.0))
                                .desired_width(380.0)
                                .interactive(true),
                        );
                    });
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new("Copy")
                                    .font(FontId::proportional(11.0))
                                    .color(t::ACCENT),
                            )
                            .fill(egui::Color32::TRANSPARENT)
                            .frame(false),
                        )
                        .clicked()
                    {
                        ui.ctx().copy_text(s.export_keyfile_output.clone());
                    }
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new("Save to disk")
                                    .font(FontId::proportional(11.0))
                                    .color(t::ACCENT),
                            )
                            .fill(egui::Color32::TRANSPARENT)
                            .frame(false),
                        )
                        .clicked()
                    {
                        let path = dirs::document_dir()
                            .unwrap_or_else(|| std::path::PathBuf::from("."))
                            .join(format!("vnox-{}-{}.vnoxkey", identity.nickname, &identity.pubkey_hex[..8]));
                        match std::fs::write(&path, &s.export_keyfile_output) {
                            Ok(()) => {
                                s.export_keyfile_error = Some(format!("saved to: {}", path.display()));
                            }
                            Err(e) => {
                                s.export_keyfile_error = Some(format!("save failed: {e}"));
                            }
                        }
                    }
                });
            }
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui
                    .add(
                        egui::Button::new(
                            RichText::new("Close")
                                .font(FontId::proportional(12.0))
                                .color(t::TEXT_MUTED),
                        )
                        .fill(egui::Color32::TRANSPARENT)
                        .frame(false),
                    )
                    .clicked()
                {
                    s.export_keyfile_open = false;
                }
            });
        });
}

fn show_import_keyfile_modal(ctx: &egui::Context, s: &mut UiState) {
    egui::Window::new("Import Identity Keyfile")
        .collapsible(false)
        .resizable(true)
        .default_size([440.0, 360.0])
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .frame(
            Frame::window(&ctx.style())
                .fill(t::BG_ELEVATED)
                .stroke(egui::Stroke::new(1.0, t::BORDER_DEFAULT))
                .corner_radius(t::R_LG),
        )
        .show(ctx, |ui| {
            ui.set_width(420.0);
            ui.label(
                RichText::new("! Importing a keyfile will replace your current identity. Back up your current keypair first if you want to keep it.")
                    .font(FontId::proportional(11.0))
                    .color(t::WARNING),
            );
            ui.add_space(8.0);
            ui.label(
                RichText::new("Paste keyfile JSON here:")
                    .font(FontId::proportional(12.0))
                    .color(t::TEXT_PRIMARY),
            );
            egui::ScrollArea::vertical()
                .max_height(140.0)
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut s.import_keyfile_input)
                            .font(FontId::monospace(10.0))
                            .desired_width(380.0)
                            .hint_text(RichText::new(r#"{"version":1,"scheme":"..."}"#).color(t::TEXT_DISABLED)),
                    );
                });
            ui.add_space(6.0);
            ui.label(
                RichText::new("Passphrase (leave empty for plain keyfiles)")
                    .font(FontId::proportional(12.0))
                    .color(t::TEXT_PRIMARY),
            );
            ui.add(
                egui::TextEdit::singleline(&mut s.import_keyfile_pass)
                    .password(true)
                    .font(FontId::monospace(12.0))
                    .desired_width(380.0),
            );
            ui.add_space(8.0);
            let can_import = !s.import_keyfile_input.trim().is_empty();
            let import_btn = egui::Button::new(
                RichText::new("Import & Replace Identity")
                    .font(FontId::proportional(12.0))
                    .color(egui::Color32::WHITE),
            )
            .fill(t::ERROR)
            .stroke(egui::Stroke::NONE)
            .corner_radius(t::R_SM)
            .min_size(egui::vec2(180.0, 28.0));
            if ui.add_enabled(can_import, import_btn).clicked() {
                let pass = if s.import_keyfile_pass.is_empty() {
                    None
                } else {
                    Some(s.import_keyfile_pass.as_str())
                };
                match identity::import_keyfile(s.import_keyfile_input.trim(), pass) {
                    Ok(identity) => {
                        // Persist imported identity (without passphrase for now —
                        // the user can set one separately).
                        match identity::save(&identity, None) {
                            Ok(()) => {
                                s.import_keyfile_error = Some(
                                    "imported — restart VNOX to use the new identity".into(),
                                );
                            }
                            Err(e) => {
                                s.import_keyfile_error = Some(format!("save failed: {e}"));
                            }
                        }
                    }
                    Err(e) => {
                        s.import_keyfile_error = Some(format!("import failed: {e}"));
                    }
                }
            }
            if let Some(ref err) = s.import_keyfile_error {
                ui.label(RichText::new(err).font(FontId::proportional(11.0)).color(t::TEXT_MUTED));
            }
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui
                    .add(
                        egui::Button::new(
                            RichText::new("Close")
                                .font(FontId::proportional(12.0))
                                .color(t::TEXT_MUTED),
                        )
                        .fill(egui::Color32::TRANSPARENT)
                        .frame(false),
                    )
                    .clicked()
                {
                    s.import_keyfile_open = false;
                }
            });
        });
}
