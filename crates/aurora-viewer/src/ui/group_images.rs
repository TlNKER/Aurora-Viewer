//! Highland: a picture of your own for a group, chosen from a file on the
//! computer (the "Custom image" button of the group's window). It shows in
//! place of the group's insignia in the groups list, the group's window
//! and Conversations, for you only. Each is kept, cropped square, in the
//! account's `group_images` folder as `<group id>.png`.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use uuid::Uuid;

/// The folder of the account's pictures, under its settings.
pub const FOLDER: &str = "group_images";
/// The side the pictures are kept at.
const SIDE: u32 = 256;

#[derive(Default)]
struct State {
    dir: Option<PathBuf>,
    /// Loaded pictures (None: the group has none of its own).
    loaded: HashMap<Uuid, Option<egui::TextureHandle>>,
}

static STATE: Mutex<Option<State>> = Mutex::new(None);

fn with<R>(f: impl FnOnce(&mut State) -> R) -> R {
    let mut guard = STATE.lock().unwrap_or_else(|e| e.into_inner());
    f(guard.get_or_insert_with(State::default))
}

/// The account's folder (at login).
pub fn set_dir(dir: PathBuf) {
    with(|s| {
        s.dir = Some(dir);
        s.loaded.clear();
    });
}

fn file(dir: &std::path::Path, group: &Uuid) -> PathBuf {
    dir.join(format!("{group}.png"))
}

/// The group's own picture, if one was chosen.
pub fn texture(ctx: &egui::Context, group: &Uuid) -> Option<egui::TextureHandle> {
    with(|s| {
        if let Some(t) = s.loaded.get(group) {
            return t.clone();
        }
        let t = s.dir.as_ref().and_then(|dir| load(ctx, &file(dir, group), group));
        s.loaded.insert(*group, t.clone());
        t
    })
}

/// Whether the group has a picture of its own.
pub fn has(group: &Uuid) -> bool {
    with(|s| s.dir.as_ref().is_some_and(|dir| file(dir, group).exists()))
}

fn load(ctx: &egui::Context, path: &std::path::Path, group: &Uuid) -> Option<egui::TextureHandle> {
    let bytes = std::fs::read(path).ok()?;
    let img = image::load_from_memory(&bytes).ok()?.to_rgba8();
    let size = [img.width() as usize, img.height() as usize];
    let color = egui::ColorImage::from_rgba_unmultiplied(size, img.as_raw());
    Some(ctx.load_texture(format!("highland-group-image-{group}"), color, egui::TextureOptions::LINEAR))
}

/// Asks for a picture file and keeps it, cropped square, for the group.
/// Returns an error to show when the file cannot be used.
pub fn choose(group: Uuid) -> Result<(), String> {
    let Some(dir) = with(|s| s.dir.clone()) else {
        return Err("Not logged in".to_owned());
    };
    let Some(path) = rfd::FileDialog::new()
        .set_title("Choose a picture for the group")
        .add_filter("Pictures", &["png", "jpg", "jpeg", "bmp", "tga", "gif", "webp"])
        .pick_file()
    else {
        return Ok(());
    };
    let img = image::open(&path).map_err(|e| format!("Cannot open the picture: {e}"))?;
    let side = img.width().min(img.height());
    let square = img.crop_imm((img.width() - side) / 2, (img.height() - side) / 2, side, side);
    let small = square.resize_exact(SIDE, SIDE, image::imageops::FilterType::Lanczos3);
    std::fs::create_dir_all(&dir).map_err(|e| format!("Cannot save the picture: {e}"))?;
    small
        .to_rgba8()
        .save_with_format(file(&dir, &group), image::ImageFormat::Png)
        .map_err(|e| format!("Cannot save the picture: {e}"))?;
    with(|s| {
        s.loaded.remove(&group);
    });
    Ok(())
}

/// Goes back to the group's own insignia.
pub fn remove(group: Uuid) {
    with(|s| {
        if let Some(dir) = &s.dir {
            let _ = std::fs::remove_file(file(dir, &group));
        }
        s.loaded.remove(&group);
    });
}
