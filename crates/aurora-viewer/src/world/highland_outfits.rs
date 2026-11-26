//! Highland: an item linked into, or unlinked from, every outfit under My
//! Outfits at once (the inventory's "Add to all outfits", the Outfits
//! window's "Remove from all outfits"). The outfits are the outfit folders
//! (FT_OUTFIT) anywhere under My Outfits, outside the Library; the links are
//! made and deleted as one inventory change, the originals never touched.

use super::inventory::{FetchState, Inventory};
use aurora_net::inventory::operations::{Mutation, Operation};
use std::collections::HashSet;
use uuid::Uuid;

/// FT_MY_OUTFITS and FT_OUTFIT.
const MY_OUTFITS: i32 = 48;
const OUTFIT: i32 = 47;
/// An inventory link (AT_LINK).
const LINK: i32 = 24;

/// Every outfit folder under My Outfits, at any depth.
pub fn outfits(inv: &Inventory) -> Vec<Uuid> {
    let Some(root) = inv
        .folders
        .iter()
        .find(|(_, f)| f.info.type_default as i32 == MY_OUTFITS && !f.library)
        .map(|(id, _)| *id)
    else {
        return Vec::new();
    };
    let mut found = Vec::new();
    let mut stack = vec![root];
    let mut seen = HashSet::new();
    while let Some(id) = stack.pop() {
        if !seen.insert(id) {
            continue;
        }
        let Some(f) = inv.folders.get(&id) else {
            continue;
        };
        if id != root && f.info.type_default as i32 == OUTFIT {
            found.push(id);
        }
        stack.extend(f.children.iter().copied());
    }
    found
}

/// The originals of `ids`: a link stands for the item it links to.
fn originals(inv: &Inventory, ids: &[Uuid]) -> Vec<Uuid> {
    let mut out = Vec::new();
    for id in ids {
        let target = match inv.items.get(id) {
            Some(it) if it.asset_type == LINK => it.asset_id,
            Some(_) => *id,
            None => continue,
        };
        if !out.contains(&target) {
            out.push(target);
        }
    }
    out
}

/// The links of folder `folder` to `target`.
fn links_to(inv: &Inventory, folder: Uuid, target: Uuid) -> Vec<Uuid> {
    inv.folders.get(&folder).map_or_else(Vec::new, |f| {
        f.items
            .iter()
            .filter(|id| inv.items.get(id).is_some_and(|it| it.asset_type == LINK && it.asset_id == target))
            .copied()
            .collect()
    })
}

/// What "Add to all outfits" changes: a link to each of `ids` in every
/// loaded outfit without one. Returns the change, the outfits it adds to
/// and the outfits left out because they have not loaded yet.
pub fn add_plan(inv: &Inventory, ids: &[Uuid]) -> (Mutation, usize, usize) {
    let mut change = Mutation::default();
    let (mut added, mut unloaded) = (0, 0);
    let targets: Vec<_> = originals(inv, ids).into_iter().filter_map(|id| inv.items.get(&id)).collect();
    for folder in outfits(inv) {
        if inv.folders.get(&folder).is_none_or(|f| f.state != FetchState::Fetched) {
            unloaded += 1;
            continue;
        }
        let links: Vec<_> = targets
            .iter()
            .filter(|it| links_to(inv, folder, it.id).is_empty())
            .map(|it| aurora_net::outfits::OutfitLink {
                target: it.id,
                name: it.name.clone(),
                desc: String::new(),
                folder: false,
                inv_type: it.inv_type,
            })
            .collect();
        if links.is_empty() {
            continue;
        }
        added += 1;
        change.operations.push(Operation::Links { parent: folder, links });
        change.refresh.push(folder);
    }
    (change, added, unloaded)
}

/// What "Remove from all outfits" changes: every link to one of `ids` in
/// the outfits deleted. Returns the change and the outfits it changes.
pub fn remove_plan(inv: &Inventory, ids: &[Uuid]) -> (Mutation, usize) {
    let mut change = Mutation::default();
    let mut changed = 0;
    let targets = originals(inv, ids);
    for folder in outfits(inv) {
        let links: Vec<Uuid> = targets.iter().flat_map(|t| links_to(inv, folder, *t)).collect();
        if links.is_empty() {
            continue;
        }
        changed += 1;
        for id in links {
            change.operations.push(Operation::Delete { id, folder: false });
            change.removed.push(id);
        }
        change.refresh.push(folder);
    }
    (change, changed)
}
