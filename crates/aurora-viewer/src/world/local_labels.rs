//! Highland: local labels, display names for inventory items (Oasis's
//! "Display Names for Objects"). Shift+F2 in the inventory gives an item a
//! name of your own; it shows in the inventory and the Appearance window
//! in place of the item's name, which itself stays as it is on the server.
//! The labels are kept for the account in `local_labels.json`, by the id of
//! the item (a link shows its original's label).

use super::inventory::{Inventory, actions as rules};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use uuid::Uuid;

pub const FILE: &str = "local_labels.json";

#[derive(Default)]
struct Labels {
    path: Option<PathBuf>,
    map: HashMap<Uuid, String>,
}

static LABELS: Mutex<Option<Labels>> = Mutex::new(None);
static REVISION: AtomicU64 = AtomicU64::new(0);

/// Bumped by every change of the labels (cached inventory views rebuild).
pub fn revision() -> u64 {
    REVISION.load(Ordering::Relaxed)
}

fn with<R>(f: impl FnOnce(&mut Labels) -> R) -> R {
    let mut guard = LABELS.lock().unwrap_or_else(|e| e.into_inner());
    f(guard.get_or_insert_with(Labels::default))
}

/// Reads the account's labels (at login).
pub fn load(path: PathBuf) {
    let map = match std::fs::read(&path) {
        Ok(bytes) => serde_json::from_slice::<HashMap<Uuid, String>>(&bytes).unwrap_or_else(|error| {
            log::warn!("Cannot read the local labels: {error}");
            HashMap::new()
        }),
        Err(_) => HashMap::new(),
    };
    with(|l| {
        l.path = Some(path);
        l.map = map;
    });
    REVISION.fetch_add(1, Ordering::Relaxed);
}

/// The label of one item, by its own id.
pub fn get(id: &Uuid) -> Option<String> {
    with(|l| l.map.get(id).cloned())
}

/// Sets (or, empty, removes) the label of `id` and writes the file.
pub fn set(id: Uuid, label: &str) {
    let label = label.trim();
    let (path, json) = with(|l| {
        if label.is_empty() {
            l.map.remove(&id);
        } else {
            l.map.insert(id, label.chars().take(63).collect());
        }
        (l.path.clone(), serde_json::to_vec_pretty(&l.map).ok())
    });
    REVISION.fetch_add(1, Ordering::Relaxed);
    if let (Some(path), Some(json)) = (path, json) {
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Err(error) = std::fs::write(&path, json) {
            log::warn!("Cannot write the local labels: {error}");
        }
    }
}

/// The item a label belongs to: the original of a link, else the item.
pub fn owner(inv: &Inventory, id: Uuid) -> Uuid {
    rules::original(inv, id).unwrap_or(id)
}

/// The label shown for `id` (an item or a link to one), if it has one.
pub fn label(inv: &Inventory, id: Uuid) -> Option<String> {
    if inv.folders.contains_key(&id) {
        return None;
    }
    get(&owner(inv, id)).or_else(|| get(&id))
}

/// The name to show for an item: its label, or its own name.
pub fn display_name(inv: &Inventory, id: Uuid) -> String {
    label(inv, id).unwrap_or_else(|| rules::name(inv, id))
}
