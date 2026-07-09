use crate::theme::icons as vec_icons;
use crate::{state::UiState, theme as t};
use eframe::egui::{self, FontId, Frame, RichText, Stroke};
use vnox_client::net::{NetCommand, NetHandle};

mod icons;
const BAR_WIDTH: f32 = 56.0;

pub fn show(ctx: &egui::Context, s: &mut UiState, net: &NetHandle) {
    egui::SidePanel::left("guild_bar")
        .exact_width(BAR_WIDTH)
        .resizable(false)
        .frame(Frame::NONE.fill(t::BG_STRIP))
        .show_separator_line(false)
        .show(ctx, |ui| {
            let r = ui.max_rect();
            ui.painter()
                .vline(r.right(), r.y_range(), Stroke::new(1.0, t::BORDER_SUBTLE));

            ui.with_layout(
                egui::Layout::top_down_justified(egui::Align::Center),
                |ui| {
                    ui.add_space(8.0);

                    let home_active = s.active_guild_id.is_none();
                    icons::guild_icon_drawn(ui, vec_icons::home, "Home", home_active, || {
                        s.active_guild_id = None;
                        s.active_dm_id = s.dm_conversations.first().map(|c| c.dm_id.clone());
                    });
                    ui.add_space(4.0);

                    ui.painter().hline(
                        egui::Rangef::new(r.left() + 12.0, r.right() - 12.0),
                        ui.cursor().top() + 2.0,
                        Stroke::new(1.0, t::BORDER_SUBTLE),
                    );
                    ui.add_space(6.0);

                    for guild in &s.guilds {
                        let active = s.active_guild_id.as_deref() == Some(&guild.guild_id);
                        let initial = guild
                            .name
                            .chars()
                            .next()
                            .unwrap_or('?')
                            .to_uppercase()
                            .to_string();
                        icons::guild_icon(ui, &initial, &guild.name, active, || {
                            s.active_guild_id = Some(guild.guild_id.clone());
                            let net = net.clone();
                            let gid = guild.guild_id.clone();
                            tokio::spawn(async move {
                                net.send(vnox_client::net::NetCommand::GuildList).await;
                                let _ = gid;
                            });
                        });
                        ui.add_space(2.0);
                    }

                    ui.add_space(4.0);
                    if t::icon_btn_drawn(
                        ui,
                        vec_icons::plus,
                        "create a guild",
                        None,
                        icons::GUILD_ICON_SIZE,
                    )
                    .clicked()
                    {
                        s.create_guild_open = true;
                        s.create_guild_input.clear();
                    }

                    ui.add_space(4.0);
                    if t::icon_btn_drawn(
                        ui,
                        vec_icons::link,
                        "join with invite code",
                        None,
                        icons::GUILD_ICON_SIZE,
                    )
                    .clicked()
                    {
                        s.invite_accept_open = true;
                        s.invite_accept_code.clear();
                    }

                    ui.add_space(4.0);
                    let has_guild = s.active_guild_id.is_some();
                    let audit_resp = t::icon_btn_drawn(
                        ui,
                        vec_icons::clipboard,
                        "view audit log (admins only)",
                        None,
                        icons::GUILD_ICON_SIZE,
                    );
                    if has_guild
                        && audit_resp.clicked()
                        && let Some(gid) = s.active_guild_id.clone()
                    {
                        s.audit_log_open = true;
                        s.audit_log_guild_id = Some(gid.clone());
                        s.audit_log_entries.clear();
                        let net = net.clone();
                        tokio::spawn(async move {
                            net.send(NetCommand::GuildAuditLogFetch {
                                guild_id: gid,
                                limit: 50,
                            })
                            .await;
                        });
                    }

                    ui.add_space(4.0);
                    let members_resp = t::icon_btn_drawn(
                        ui,
                        vec_icons::people,
                        "guild members",
                        None,
                        icons::GUILD_ICON_SIZE,
                    );
                    let has_guild = s.active_guild_id.is_some();
                    if has_guild
                        && members_resp.clicked()
                        && let Some(gid) = s.active_guild_id.clone()
                    {
                        s.guild_members_open = true;
                        s.guild_member_list_guild_id = Some(gid.clone());
                        s.guild_member_list.clear();
                        s.guild_role_list.clear();
                        let net = net.clone();
                        tokio::spawn(async move {
                            net.send(NetCommand::GuildMemberListFetch {
                                guild_id: gid.clone(),
                            })
                            .await;
                            net.send(NetCommand::GuildRoleListFetch { guild_id: gid })
                                .await;
                        });
                    }
                },
            );

            if s.create_guild_open {
                show_create_guild_popup(ctx, s, net);
            }
            if s.invite_accept_open {
                show_invite_accept_popup(ctx, s, net);
            }
            if s.audit_log_open {
                show_audit_log_popup(ctx, s, net);
            }
            if s.guild_members_open {
                show_members_popup(ctx, s, net);
            }
        });
}

fn show_create_guild_popup(ctx: &egui::Context, s: &mut UiState, net: &NetHandle) {
    egui::Window::new("Create a Guild")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .frame(
            Frame::window(&ctx.style())
                .fill(t::BG_ELEVATED)
                .stroke(Stroke::new(1.0, t::BORDER_DEFAULT))
                .corner_radius(t::R_LG),
        )
        .show(ctx, |ui| {
            ui.set_width(280.0);
            ui.label(
                RichText::new("Guild Name")
                    .font(FontId::proportional(12.0))
                    .color(t::TEXT_PRIMARY),
            );
            ui.add_space(8.0);
            let resp = ui.add(
                egui::TextEdit::singleline(&mut s.create_guild_input)
                    .font(FontId::proportional(14.0))
                    .text_color(t::TEXT_PRIMARY)
                    .hint_text(RichText::new("e.g. My Awesome Guild").color(t::TEXT_DISABLED))
                    .desired_width(240.0),
            );
            ui.add_space(12.0);
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
                    s.create_guild_open = false;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let create_btn = egui::Button::new(
                        RichText::new("Create")
                            .font(FontId::proportional(12.0))
                            .color(egui::Color32::WHITE),
                    )
                    .fill(t::ACCENT)
                    .stroke(Stroke::NONE)
                    .corner_radius(t::R_SM)
                    .min_size(egui::vec2(80.0, 28.0));
                    if ui.add(create_btn).clicked() && !s.create_guild_input.trim().is_empty() {
                        let net = net.clone();
                        let name = s.create_guild_input.trim().to_string();
                        tokio::spawn(async move {
                            net.send(vnox_client::net::NetCommand::GuildCreate { name })
                                .await;
                        });
                        s.create_guild_open = false;
                    }
                });
            });
            if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                s.create_guild_open = false;
            }
        });
}

fn show_invite_accept_popup(ctx: &egui::Context, s: &mut UiState, net: &NetHandle) {
    egui::Window::new("Join a Guild")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .frame(
            Frame::window(&ctx.style())
                .fill(t::BG_ELEVATED)
                .stroke(Stroke::new(1.0, t::BORDER_DEFAULT))
                .corner_radius(t::R_LG),
        )
        .show(ctx, |ui| {
            ui.set_width(280.0);
            ui.label(
                RichText::new("Invite Code")
                    .font(FontId::proportional(12.0))
                    .color(t::TEXT_PRIMARY),
            );
            ui.add_space(8.0);
            let resp = ui.add(
                egui::TextEdit::singleline(&mut s.invite_accept_code)
                    .font(FontId::monospace(14.0))
                    .text_color(t::TEXT_PRIMARY)
                    .hint_text(RichText::new("e.g. a1b2c3d4").color(t::TEXT_DISABLED))
                    .desired_width(240.0),
            );
            ui.add_space(12.0);
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
                    s.invite_accept_open = false;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let join_btn = egui::Button::new(
                        RichText::new("Join")
                            .font(FontId::proportional(12.0))
                            .color(egui::Color32::WHITE),
                    )
                    .fill(t::ACCENT)
                    .stroke(Stroke::NONE)
                    .corner_radius(t::R_SM)
                    .min_size(egui::vec2(80.0, 28.0));
                    if ui.add(join_btn).clicked() && !s.invite_accept_code.trim().is_empty() {
                        let net = net.clone();
                        let code = s.invite_accept_code.trim().to_string();
                        tokio::spawn(async move {
                            net.send(NetCommand::InviteAccept { code }).await;
                        });
                        s.invite_accept_open = false;
                    }
                });
            });
            if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                s.invite_accept_open = false;
            }
        });
}

fn show_audit_log_popup(ctx: &egui::Context, s: &mut UiState, _net: &NetHandle) {
    egui::Window::new("Audit Log")
        .collapsible(false)
        .resizable(true)
        .default_size([520.0, 420.0])
        .min_size([400.0, 300.0])
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .frame(
            Frame::window(&ctx.style())
                .fill(t::BG_ELEVATED)
                .stroke(egui::Stroke::new(1.0, t::BORDER_DEFAULT))
                .corner_radius(t::R_LG),
        )
        .show(ctx, |ui| {
            ui.set_width(500.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Last 50 admin actions for this guild")
                        .font(FontId::proportional(11.0))
                        .color(t::TEXT_MUTED),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if t::icon_btn_drawn(ui, vec_icons::close, "close", None, 18.0).clicked() {
                        s.audit_log_open = false;
                    }
                });
            });
            ui.add_space(6.0);
            ui.separator();
            ui.add_space(4.0);

            if s.audit_log_entries.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(40.0);
                    ui.label(
                        RichText::new("no audit log entries")
                            .font(FontId::proportional(12.0))
                            .color(t::TEXT_DISABLED),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("(loading or empty — admin actions like kick, role change, invite create will appear here)")
                            .font(FontId::proportional(10.0))
                            .color(t::TEXT_DISABLED),
                    );
                });
                return;
            }

            egui::ScrollArea::vertical()
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    for entry in s.audit_log_entries.clone() {
                        Frame::NONE
                            .fill(t::BG_SURFACE)
                            .corner_radius(t::R_SM)
                            .stroke(egui::Stroke::new(1.0, t::BORDER_SUBTLE))
                            .inner_margin(egui::Margin::symmetric(10, 6))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    let (icon, color) = action_icon(&entry.action);
                                    ui.label(
                                        RichText::new(icon)
                                            .font(FontId::proportional(14.0))
                                            .color(color),
                                    );
                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(
                                                RichText::new(&entry.action)
                                                    .font(FontId::proportional(12.0))
                                                    .color(t::TEXT_PRIMARY)
                                                    .strong(),
                                            );
                                            ui.label(
                                                RichText::new(format!("· by {}…", &entry.actor_id[..8.min(entry.actor_id.len())]))
                                                    .font(FontId::monospace(10.0))
                                                    .color(t::TEXT_DISABLED),
                                            );
                                        });
                                        if let Some(t_id) = &entry.target_id {
                                            ui.label(
                                                RichText::new(format!("target: {}…", &t_id[..8.min(t_id.len())]))
                                                    .font(FontId::monospace(10.0))
                                                    .color(t::TEXT_MUTED),
                                            );
                                        }
                                        if let Some(reason) = &entry.reason {
                                            ui.label(
                                                RichText::new(format!("reason: {reason}"))
                                                    .font(FontId::proportional(11.0))
                                                    .color(t::TEXT_MUTED)
                                                    .italics(),
                                            );
                                        }
                                    });
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.label(
                                            RichText::new(fmt_audit_ts(entry.created_at))
                                                .font(FontId::monospace(10.0))
                                                .color(t::TEXT_DISABLED),
                                        );
                                    });
                                });
                            });
                        ui.add_space(2.0);
                    }
                });
        });
}

fn action_icon(action: &str) -> (&'static str, egui::Color32) {
    match action {
        "guild_create" | "GUILD_CREATE" => ("[+]", t::SUCCESS),
        "guild_delete" | "GUILD_DELETE" => ("[-]", t::ERROR),
        "member_kick" | "MEMBER_KICK" => ("[kick]", t::ERROR),
        "member_leave" | "MEMBER_LEAVE" => ("[leave]", t::WARNING),
        "role_create" | "ROLE_CREATE" => ("[role+]", t::INFO),
        "role_delete" | "ROLE_DELETE" => ("[role-]", t::ERROR),
        "role_assign" | "ROLE_ASSIGN" => ("[role=]", t::INFO),
        "role_unassign" | "ROLE_UNASSIGN" => ("[role!=]", t::WARNING),
        "invite_create" | "INVITE_CREATE" => ("[invite+]", t::SUCCESS),
        "invite_delete" | "INVITE_DELETE" => ("[invite-]", t::ERROR),
        "invite_accept" | "INVITE_ACCEPT" => ("[invite=]", t::SUCCESS),
        _ => ("[*]", t::TEXT_MUTED),
    }
}

fn fmt_audit_ts(ts: i64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let diff = (now - ts / 1000).max(0);
    if diff < 60 {
        format!("{diff}s ago")
    } else if diff < 3600 {
        format!("{}m ago", diff / 60)
    } else if diff < 86400 {
        format!("{}h ago", diff / 3600)
    } else {
        format!("{}d ago", diff / 86400)
    }
}

fn show_members_popup(ctx: &egui::Context, s: &mut UiState, net: &NetHandle) {
    egui::Window::new("Guild Members")
        .collapsible(false)
        .resizable(true)
        .default_size([520.0, 460.0])
        .min_size([400.0, 320.0])
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .frame(
            Frame::window(&ctx.style())
                .fill(t::BG_ELEVATED)
                .stroke(egui::Stroke::new(1.0, t::BORDER_DEFAULT))
                .corner_radius(t::R_LG),
        )
        .show(ctx, |ui| {
            ui.set_width(500.0);
            ui.horizontal(|ui| {
                let count = s.guild_member_list.len();
                ui.label(
                    RichText::new(format!("{count} members"))
                        .font(FontId::proportional(11.0))
                        .color(t::TEXT_MUTED),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if t::icon_btn_drawn(ui, vec_icons::refresh, "refresh", None, 18.0).clicked()
                        && let Some(gid) = s.guild_member_list_guild_id.clone()
                    {
                        let net = net.clone();
                        tokio::spawn(async move {
                            net.send(NetCommand::GuildMemberListFetch {
                                guild_id: gid.clone(),
                            })
                            .await;
                            net.send(NetCommand::GuildRoleListFetch { guild_id: gid })
                                .await;
                        });
                    }
                    if t::icon_btn_drawn(ui, vec_icons::close, "close", None, 18.0).clicked() {
                        s.guild_members_open = false;
                    }
                });
            });
            ui.add_space(6.0);
            ui.separator();
            ui.add_space(4.0);

            if s.guild_member_list.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(40.0);
                    ui.label(
                        RichText::new("no members")
                            .font(FontId::proportional(12.0))
                            .color(t::TEXT_DISABLED),
                    );
                });
                return;
            }

            // Determine if current user is owner (kick allowed).
            // For simplicity, we allow kick UI for everyone; gateway enforces permission.
            let roles_clone = s.guild_role_list.clone();
            let guild_id = s.guild_member_list_guild_id.clone().unwrap_or_default();

            egui::ScrollArea::vertical()
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    for member in s.guild_member_list.clone() {
                        let role_color =
                            parse_hex_color(&member.role_color).unwrap_or(t::TEXT_PRIMARY);
                        Frame::NONE
                            .fill(t::BG_SURFACE)
                            .corner_radius(t::R_SM)
                            .stroke(egui::Stroke::new(1.0, t::BORDER_SUBTLE))
                            .inner_margin(egui::Margin::symmetric(10, 6))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    // Avatar with role-colored ring
                                    let init = member
                                        .nickname
                                        .chars()
                                        .next()
                                        .unwrap_or('?')
                                        .to_uppercase()
                                        .to_string();
                                    let (r, _) = ui.allocate_exact_size(
                                        egui::vec2(28.0, 28.0),
                                        egui::Sense::hover(),
                                    );
                                    ui.painter().circle_filled(r.center(), 14.0, role_color);
                                    ui.painter().text(
                                        r.center(),
                                        egui::Align2::CENTER_CENTER,
                                        &init,
                                        egui::FontId::proportional(12.0),
                                        egui::Color32::from_rgb(15, 15, 15),
                                    );

                                    ui.add_space(8.0);
                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(
                                                RichText::new(&member.nickname)
                                                    .font(FontId::proportional(13.0))
                                                    .color(t::TEXT_PRIMARY)
                                                    .strong(),
                                            );
                                            if member.is_owner {
                                                let (cr, _) = ui.allocate_exact_size(
                                                    egui::vec2(14.0, 14.0),
                                                    egui::Sense::hover(),
                                                );
                                                vec_icons::crown(ui.painter(), cr, t::WARNING);
                                            }
                                        });
                                        ui.horizontal(|ui| {
                                            let (dot_r, _) = ui.allocate_exact_size(
                                                egui::vec2(8.0, 8.0),
                                                egui::Sense::hover(),
                                            );
                                            ui.painter().circle_filled(
                                                dot_r.center(),
                                                4.0,
                                                role_color,
                                            );
                                            ui.label(
                                                RichText::new(&member.role_name)
                                                    .font(FontId::proportional(10.0))
                                                    .color(t::TEXT_MUTED),
                                            );
                                            ui.label(
                                                RichText::new(format!(
                                                    "· {}…",
                                                    &member.user_id[..8.min(member.user_id.len())]
                                                ))
                                                .font(FontId::monospace(10.0))
                                                .color(t::TEXT_DISABLED),
                                            );
                                        });
                                    });

                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            // Role-assign combo (only shows if user has roles defined).
                                            if !roles_clone.is_empty() && !member.is_owner {
                                                let combo_text = "+ role".to_string();
                                                egui::ComboBox::from_id_salt(format!(
                                                    "role_combo_{}",
                                                    member.user_id
                                                ))
                                                .selected_text(combo_text)
                                                .width(120.0)
                                                .show_ui(ui, |ui| {
                                                    for role in &roles_clone {
                                                        if ui
                                                            .add(
                                                                egui::Button::new(
                                                                    RichText::new(&role.name)
                                                                        .font(FontId::proportional(
                                                                            11.0,
                                                                        ))
                                                                        .color(
                                                                            parse_hex_color(
                                                                                &role.color,
                                                                            )
                                                                            .unwrap_or(
                                                                                t::TEXT_PRIMARY,
                                                                            ),
                                                                        ),
                                                                )
                                                                .fill(egui::Color32::TRANSPARENT)
                                                                .frame(false),
                                                            )
                                                            .clicked()
                                                        {
                                                            let net = net.clone();
                                                            let gid = guild_id.clone();
                                                            let uid = member.user_id.clone();
                                                            let rid = role.id.clone();
                                                            tokio::spawn(async move {
                                                                net.send(
                                                                    NetCommand::GuildRoleAssign {
                                                                        guild_id: gid,
                                                                        user_id: uid,
                                                                        role_id: rid,
                                                                    },
                                                                )
                                                                .await;
                                                            });
                                                        }
                                                    }
                                                });
                                            }
                                            // Kick button (hidden for owner).
                                            if !member.is_owner
                                                && t::icon_btn_drawn(
                                                    ui,
                                                    vec_icons::boot,
                                                    "kick (admins only)",
                                                    Some(t::ERROR),
                                                    18.0,
                                                )
                                                .clicked()
                                            {
                                                let net_kick = net.clone();
                                                let net_refresh = net.clone();
                                                let gid = guild_id.clone();
                                                let uid = member.user_id.clone();
                                                tokio::spawn(async move {
                                                    net_kick
                                                        .send(NetCommand::GuildMemberKick {
                                                            guild_id: gid.clone(),
                                                            user_id: uid,
                                                        })
                                                        .await;
                                                    tokio::time::sleep(
                                                        std::time::Duration::from_millis(200),
                                                    )
                                                    .await;
                                                    net_refresh
                                                        .send(NetCommand::GuildMemberListFetch {
                                                            guild_id: gid,
                                                        })
                                                        .await;
                                                });
                                            }
                                        },
                                    );
                                });
                            });
                        ui.add_space(2.0);
                    }
                });
        });
}

fn parse_hex_color(hex: &str) -> Option<egui::Color32> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(egui::Color32::from_rgb(r, g, b))
}
