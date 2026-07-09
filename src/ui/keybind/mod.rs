//! Global hotkey detection (egui input snapshot).

mod bindings;

use eframe::egui::{self, Key, PointerButton};

use bindings::{ALL_KEYS, key_from_name, name_from_key};

/// True while the binding is held (push-to-talk).
pub fn binding_held(ctx: &egui::Context, binding: &str) -> bool {
    let b = binding.trim().to_lowercase();
    ctx.input(|i| match b.as_str() {
        "mouse4" | "mb4" | "x1" => i.pointer.button_down(PointerButton::Extra1),
        "mouse5" | "mb5" | "x2" => i.pointer.button_down(PointerButton::Extra2),
        "lmb" | "mouse1" => i.pointer.button_down(PointerButton::Primary),
        "rmb" | "mouse2" => i.pointer.button_down(PointerButton::Secondary),
        "mmb" | "mouse3" => i.pointer.button_down(PointerButton::Middle),
        "space" => i.key_down(Key::Space),
        _ => combo_down(&b, i),
    })
}

/// Rising edge (toggle mute/deafen, etc.).
pub fn binding_pressed(ctx: &egui::Context, binding: &str) -> bool {
    let b = binding.trim().to_lowercase();
    ctx.input(|i| match b.as_str() {
        "mouse4" | "mb4" | "x1" => i.pointer.button_pressed(PointerButton::Extra1),
        "mouse5" | "mb5" | "x2" => i.pointer.button_pressed(PointerButton::Extra2),
        "ctrl+m" => i.modifiers.ctrl && i.key_pressed(Key::M),
        "ctrl+d" => i.modifiers.ctrl && i.key_pressed(Key::D),
        "ctrl+shift+o" => i.modifiers.ctrl && i.modifiers.shift && i.key_pressed(Key::O),
        _ => combo_pressed(&b, i),
    })
}

/// Listen for any key/mouse press and return a binding string like "ctrl+m" or "mouse4".
pub fn listen_pressed(ctx: &egui::Context) -> Option<String> {
    ctx.input(|i| {
        for &key in ALL_KEYS {
            if i.key_pressed(key) {
                let mut parts: Vec<&str> = Vec::new();
                if i.modifiers.ctrl {
                    parts.push("ctrl");
                }
                if i.modifiers.shift {
                    parts.push("shift");
                }
                if i.modifiers.alt {
                    parts.push("alt");
                }
                if let Some(name) = name_from_key(key) {
                    parts.push(name);
                    return Some(parts.join("+"));
                }
            }
        }
        if i.pointer.button_pressed(PointerButton::Extra1) {
            return Some("mouse4".into());
        }
        if i.pointer.button_pressed(PointerButton::Extra2) {
            return Some("mouse5".into());
        }
        if i.pointer.button_pressed(PointerButton::Middle) {
            return Some("mouse3".into());
        }
        None
    })
}

fn combo_down(binding: &str, i: &egui::InputState) -> bool {
    let Some((mods, key)) = parse_combo(binding) else {
        return false;
    };
    mods.ctrl == i.modifiers.ctrl
        && mods.shift == i.modifiers.shift
        && mods.alt == i.modifiers.alt
        && i.key_down(key)
}

fn combo_pressed(binding: &str, i: &egui::InputState) -> bool {
    let Some((mods, key)) = parse_combo(binding) else {
        return false;
    };
    mods.ctrl == i.modifiers.ctrl
        && mods.shift == i.modifiers.shift
        && mods.alt == i.modifiers.alt
        && i.key_pressed(key)
}

#[derive(Default)]
struct Mods {
    ctrl: bool,
    shift: bool,
    alt: bool,
}

fn parse_combo(binding: &str) -> Option<(Mods, Key)> {
    let parts: Vec<&str> = binding.split('+').map(str::trim).collect();
    if parts.is_empty() {
        return None;
    }
    let mut mods = Mods::default();
    let key_name = *parts.last()?;
    for p in &parts[..parts.len().saturating_sub(1)] {
        match *p {
            "ctrl" | "control" => mods.ctrl = true,
            "shift" => mods.shift = true,
            "alt" => mods.alt = true,
            _ => {}
        }
    }
    let key = key_from_name(key_name)?;
    Some((mods, key))
}
