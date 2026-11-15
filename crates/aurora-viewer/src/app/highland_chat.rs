//! Highland: Highland Chat, the chat-only way to start Highland (`--chat`,
//! or the program named `highland-chat`, `crate::cli::highland_chat`), in
//! the spirit of Radegast and Speedlight. Not in the Legacy Viewer.
//!
//! It is the same program and the same world, so every chat feature comes
//! with it; what changes in this mode:
//! - the window is small, with its own size and place
//!   (`Settings::highland_chat_window`) and its own window layout file
//!   (`ui/layout.rs`), never full screen or maximized;
//! - the 3D is never drawn: empty draw lists and the smallest targets, the
//!   pause of a busy card (`App::drawing_paused`), hidden under an opaque
//!   backdrop (the windows' see-through looks show nothing behind them),
//!   and the frame rate held low (`frame_cap`);
//! - the simulator is asked for the shortest view (`agent_far`), so little of
//!   the region is sent;
//! - nothing moves the avatar, picks, orbits or zooms in the world, and the
//!   world's own sounds and the parcel music stay off;
//! - in the world, no bars, name tags, beacons or HUDs: a strip with the
//!   status, People and Log out, and the Conversations window filling the
//!   rest (`ui::widgets::Floater::highland_fill`).

use super::*;

/// Highland: the view distance asked of the simulator in Highland Chat, in
/// metres: well under the least the Legacy Viewer offers (RenderFarClip,
/// 32), so the region sends little more than what is right around us.
/// Chat, IMs and the avatars' list do not depend on it.
pub(super) const HIGHLAND_CHAT_FAR: f32 = 8.0;
/// Highland: Highland Chat's frame rate, in front and behind other windows
/// (a chat window needs no more; less work for the processor).
const HIGHLAND_CHAT_FPS: (f32, f32) = (30.0, 10.0);
/// Highland: Highland Chat's window the first time (points).
const HIGHLAND_CHAT_SIZE: (u32, u32) = (480, 760);
/// Highland: Highland Chat's smallest window (points).
pub(super) const HIGHLAND_CHAT_MIN: (u32, u32) = (360, 480);
/// Highland: the height of the strip over Conversations (points).
const HIGHLAND_CHAT_STRIP: f32 = 34.0;

/// Highland: Highland Chat's frame rate cap, under the viewer's own (`cap`,
/// 0 for none).
pub(super) fn frame_cap(cap: f32, focused: bool) -> f32 {
    let limit = if focused { HIGHLAND_CHAT_FPS.0 } else { HIGHLAND_CHAT_FPS.1 };
    if cap > 0.0 { cap.min(limit) } else { limit }
}

/// Highland: the world's own sounds, kept off in Highland Chat (the
/// interface sounds and voice stay).
pub(super) fn mutes(channel: aurora_audio::Channel) -> bool {
    use aurora_audio::Channel;
    crate::cli::highland_chat() && matches!(channel, Channel::Ambient | Channel::Sfx | Channel::Music | Channel::Media)
}

impl App {
    /// Highland: started as Highland Chat.
    pub(super) fn highland_chat(&self) -> bool {
        crate::cli::highland_chat()
    }

    /// Highland: the view distance asked of the simulator and kept by the
    /// scene: the user's draw distance, Highland Chat's short one.
    pub(super) fn agent_far(&self) -> f32 {
        if self.highland_chat() {
            HIGHLAND_CHAT_FAR.min(self.settings.draw_distance)
        } else {
            self.settings.draw_distance
        }
    }

    /// Highland: Highland Chat's window as it opens (width, height, last
    /// position); `None` for the viewer's own window.
    pub(super) fn highland_chat_geometry(&self) -> Option<(u32, u32, Option<(i32, i32)>)> {
        if !self.highland_chat() {
            return None;
        }
        let (w, h, at) = self
            .settings
            .highland_chat_window
            .unwrap_or((HIGHLAND_CHAT_SIZE.0, HIGHLAND_CHAT_SIZE.1, None));
        Some((w.max(HIGHLAND_CHAT_MIN.0), h.max(HIGHLAND_CHAT_MIN.1), at))
    }

    /// Highland: Highland Chat's window was moved or resized (saved with the
    /// settings' next save, as the viewer's own window is).
    pub(super) fn highland_chat_record(&mut self, width: u32, height: u32, at: Option<(i32, i32)>) {
        let now = Some((width, height, at));
        if self.settings.highland_chat_window != now {
            self.settings.highland_chat_window = now;
        }
    }

    /// Highland: in the world, Highland Chat's strip at the top of the
    /// window (status, own name, People, Log out); returns the room left for
    /// the windows under it. Conversations stays open there.
    pub(super) fn highland_chat_strip(&mut self, ctx: &egui::Context, p: &Palette, a: &mut UiActions) -> egui::Rect {
        let screen = ctx.content_rect();
        let strip = egui::Rect::from_min_size(screen.min, egui::vec2(screen.width(), HIGHLAND_CHAT_STRIP));
        let status = self.world.status.modes();
        let name = self.world.own_name();
        let mut action = ui::bars::BarAction::None;
        let mut people = false;
        // an opaque backdrop under every window: the paused world's sky
        // never shows through a window's see-through look nor around it
        ctx.layer_painter(egui::LayerId::background()).rect_filled(screen, 0.0, p.navy);
        egui::Area::new(egui::Id::unique("highland-chat-strip"))
            .fixed_pos(strip.min)
            // over the backdrop (the background layer), with the windows
            .order(egui::Order::Middle)
            .show(ctx, |ui| {
                ui.set_min_size(strip.size());
                ui.set_max_size(strip.size());
                ui.painter().rect_filled(strip, 0.0, p.bar);
                ui.painter().hline(
                    strip.x_range(),
                    strip.bottom() - 0.5,
                    egui::Stroke::new(1.0, p.muted_dim.gamma_multiply(0.6)),
                );
                let inner = strip.shrink2(egui::vec2(8.0, 4.0));
                ui.scope_builder(egui::UiBuilder::new().max_rect(inner), |ui| {
                    ui.horizontal_centered(|ui| {
                        ui::bars::highland_status_dot(ui, p, status, &mut action);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui::widgets::flat_button(ui, p, aurora_i18n::tr!("ui-bars-log-out")).clicked() {
                                action = ui::bars::BarAction::Logout;
                            }
                            if ui::widgets::flat_button(ui, p, aurora_i18n::tr!("common-people")).clicked() {
                                people = true;
                            }
                            // the own name in the room the buttons leave
                            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                ui.add(egui::Label::new(egui::RichText::new(name).color(p.ink)).truncate());
                            });
                        });
                    });
                });
            });
        if people {
            ui::widgets::toggle_floater(ctx, "people", &mut self.panels.people);
        }
        if !matches!(action, ui::bars::BarAction::None) {
            a.bar = action;
        }
        // Conversations is the window: always open
        self.panels.chat = true;
        egui::Rect::from_min_max(egui::pos2(screen.left(), strip.bottom()), screen.max)
    }
}
