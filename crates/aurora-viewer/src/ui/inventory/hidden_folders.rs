//! Highland: system folders hidden from the inventory (Preferences ›
//! Highland › System Folders), as Oasis's God Mode › Hidden System Folders
//! (FSFloaterHiddenSystemFolders). A hidden folder only leaves the inventory
//! windows' trees; it stays on the server with everything in it, and comes
//! back as soon as it is ticked again.
//!
//! The setting (`InventoryPreferences::hidden_system_folders`) holds the
//! names of the hidden folders; `sync` turns it into a bit mask once a frame
//! and `ViewCache::children_iter` leaves those folders out.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// The folders that can be hidden, in the pane's order, with their system
/// folder type (LLFolderType). Matched by type, never by (translated) name;
/// `#Aurora` and `#Firestorm` have no type: each is the folder of that name
/// at the root.
pub const HIDEABLE: &[(&str, Option<i32>)] = &[
    ("Animations", Some(20)),
    ("Body Parts", Some(13)),
    ("Calling Cards", Some(2)),
    ("Clothing", Some(5)),
    ("Current Outfit", Some(46)),
    ("Favorites", Some(23)),
    ("Landmarks", Some(3)),
    ("Marketplace Listings", Some(53)),
    ("My Outfits", Some(48)),
    ("Notecards", Some(7)),
    ("Objects", Some(6)),
    ("Settings", Some(56)),
    ("Textures", Some(0)),
    ("#Aurora", None),
    ("#Firestorm", None),
];

/// The bits of the folders without a type, by name (folder types stay
/// below them).
pub const AURORA_BIT: u32 = 63;
const NAMED: &[(&str, u32)] = &[("#Aurora", AURORA_BIT), ("#Firestorm", 62)];

static MASK: AtomicU64 = AtomicU64::new(0);
static HIDE_EMPTY: AtomicBool = AtomicBool::new(true);
static GENERATION: AtomicU64 = AtomicU64::new(0);

fn bit(name: &str) -> Option<u32> {
    HIDEABLE
        .iter()
        .find(|(n, _)| *n == name)
        .and_then(|(n, kind)| match kind {
            Some(k) => Some(*k as u32),
            None => NAMED.iter().find(|(named, _)| named == n).map(|(_, b)| *b),
        })
}

/// Takes the settings (once a frame): the hidden folders' names and Hide
/// empty folders. A change lists the trees again.
pub fn sync(hidden: &[String], hide_empty: bool) {
    let mask = hidden.iter().filter_map(|n| bit(n)).fold(0u64, |m, b| m | (1 << b));
    let changed = MASK.swap(mask, Ordering::Relaxed) != mask;
    if HIDE_EMPTY.swap(hide_empty, Ordering::Relaxed) != hide_empty || changed {
        GENERATION.fetch_add(1, Ordering::Relaxed);
    }
}

/// Folders with nothing in them, system folders included, are left out.
pub fn hide_empty() -> bool {
    HIDE_EMPTY.load(Ordering::Relaxed)
}

/// The hidden folders' bits.
pub fn mask() -> u64 {
    MASK.load(Ordering::Relaxed)
}

/// Changes whenever the hidden folders change.
pub fn generation() -> u64 {
    GENERATION.load(Ordering::Relaxed)
}

/// The bit a folder at the root answers to, if it is one that can be hidden.
pub fn folder_bit(parent_is_root: bool, type_default: i32, name: &str) -> Option<u32> {
    if !parent_is_root {
        return None;
    }
    if type_default >= 0 {
        return HIDEABLE
            .iter()
            .any(|(_, k)| *k == Some(type_default))
            .then_some(type_default as u32);
    }
    NAMED.iter().find(|(n, _)| *n == name).map(|(_, b)| *b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_become_bits_and_a_change_bumps_the_generation() {
        let before = generation();
        sync(&["Clothing".into(), "#Aurora".into(), "Unknown".into()], true);
        assert_eq!(mask(), (1 << 5) | (1 << AURORA_BIT));
        assert_ne!(generation(), before);
        let same = generation();
        sync(&["#Aurora".into(), "Clothing".into()], true);
        assert_eq!(generation(), same, "same set, same generation");
        sync(&[], true);
        assert_eq!(mask(), 0);
        assert_eq!(folder_bit(true, 5, "Clothing"), Some(5));
        assert_eq!(folder_bit(false, 5, "Clothing"), None, "only at the root");
        assert_eq!(folder_bit(true, -1, "#Aurora"), Some(AURORA_BIT));
        assert_eq!(folder_bit(true, -1, "#Firestorm"), Some(62));
        assert_eq!(folder_bit(true, 14, "Trash"), None, "the Trash cannot be hidden");
    }
}
