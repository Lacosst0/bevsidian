//! Which note grows from which: every note hangs from one other, up to the hub, and a note
//! the map leaves out is placed beside the note it hangs from.

use bevy::math::Vec2;

use bevsidian_core::vault::{NoteId, VaultData};

/// The top folder of a note ("" for the notes at the top).
pub fn top_folder(vault: &VaultData, id: NoteId) -> &str {
    vault.notes[id].folder.split('/').next().unwrap_or("")
}

pub fn has_property(vault: &VaultData, id: NoteId, key: &str) -> bool {
    property(vault, id, key) == Some("true")
}

/// The value of a front-matter property of a note.
pub fn property<'a>(vault: &'a VaultData, id: NoteId, key: &str) -> Option<&'a str> {
    vault.notes[id]
        .doc
        .properties
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, value)| value.as_str())
}

/// Whether a note is about the app itself (the welcome note and the `Bevsidian` folder): the
/// map is about Bevy, so they stay off it.
pub fn is_app_note(vault: &VaultData, id: NoteId) -> bool {
    top_folder(vault, id) == "Bevsidian" || vault.notes[id].name == "Welcome"
}

/// How deep a note sits: 0 the hub, 1 an area (a folder's index), 2 a concept, 3 a
/// sub-concept (`parent:` in its front matter), 4 a crate (`lib: true`).
pub fn rank(vault: &VaultData, id: NoteId) -> usize {
    if has_property(vault, id, "hub") {
        0
    } else if has_property(vault, id, "lib") {
        4
    } else if property(vault, id, "parent").is_some() {
        3
    } else if vault.notes[id].name == top_folder(vault, id) {
        1
    } else {
        2
    }
}

/// The tree of the `shown` notes (slots of `shown`) grown from `root`: every note hangs from
/// one other. A note that names its `parent:` hangs from it; otherwise from the first note
/// of its own folder that links to it (an area's index lists its notes), the areas from the
/// root; what no such link reaches hangs from any note linking to it, and the rest from its
/// folder's index or the root. A note hangs from a note of an inner rank where it can.
pub fn spanning_tree(
    vault: &VaultData,
    shown: &[NoteId],
    rank: &[usize],
    root: usize,
) -> Vec<Option<usize>> {
    let slot = |id: NoteId| shown.iter().position(|&n| n == id);
    let folder = |n: usize| top_folder(vault, shown[n]);
    let out: Vec<Vec<usize>> = shown
        .iter()
        .map(|&id| {
            vault.notes[id]
                .outgoing
                .iter()
                .filter_map(|&t| slot(t))
                .collect()
        })
        .collect();
    let mut parent = vec![None; shown.len()];
    let mut seen = vec![false; shown.len()];
    seen[root] = true;
    let named: Vec<Option<usize>> = (0..shown.len())
        .map(|n| {
            let name = property(vault, shown[n], "parent")?;
            (0..shown.len()).find(|&m| m != n && vault.notes[shown[m]].name == name)
        })
        .collect();
    for n in 0..shown.len() {
        if n != root
            && let Some(up) = named[n]
        {
            seen[n] = true;
            parent[n] = Some(up);
        }
    }
    // First the links inside a folder (and out of the root) to a rank further out, then such
    // links from anywhere, then any link at all.
    for pass in 0..3 {
        let mut queue: std::collections::VecDeque<usize> = (0..shown.len())
            .filter(|&n| seen[n] && (n == root || named[n].is_none()))
            .collect();
        while let Some(from) = queue.pop_front() {
            for &to in &out[from] {
                let outwards = rank[from] < rank[to];
                let allowed = match pass {
                    0 => outwards && (from == root || folder(from) == folder(to)),
                    1 => outwards,
                    _ => true,
                };
                if !seen[to] && allowed {
                    seen[to] = true;
                    parent[to] = Some(from);
                    queue.push_back(to);
                }
            }
        }
    }
    for n in 0..shown.len() {
        if !seen[n] {
            let index = (0..shown.len())
                .find(|&i| vault.notes[shown[i]].name == folder(n))
                .filter(|&i| i != n);
            parent[n] = Some(index.unwrap_or(root));
        }
    }
    parent
}

/// Homes for the notes the map leaves out (`None` in `placed`): each beside the nearest
/// placed note it hangs from, on the side away from the hub, clear of the notes around.
pub fn place_the_rest(placed: &[Option<Vec2>], parent: &[Option<usize>]) -> Vec<Vec2> {
    let mut homes: Vec<Option<Vec2>> = placed.to_vec();
    for n in 0..homes.len() {
        if homes[n].is_some() {
            continue;
        }
        let mut up = parent[n];
        let mut steps = 0;
        while let Some(p) = up {
            if homes[p].is_some() || steps > homes.len() {
                break;
            }
            up = parent[p];
            steps += 1;
        }
        let base = up.and_then(|p| homes[p]).unwrap_or(Vec2::ZERO);
        let away = base.normalize_or(Vec2::Y);
        // Try spots around the base, the outward one first, then further out.
        let spot = (0..48)
            .map(|i| {
                let turn = ((i % 8) as f32 - 3.5) * 0.45;
                let distance = UNPLACED_GAP * (1.0 + (i / 8) as f32 * 0.6);
                base + Vec2::from_angle(turn).rotate(away) * distance
            })
            .find(|spot| {
                homes
                    .iter()
                    .flatten()
                    .all(|other| other.distance(*spot) > UNPLACED_GAP * 0.8)
            })
            .unwrap_or(base + away * UNPLACED_GAP);
        homes[n] = Some(spot);
    }
    homes.into_iter().map(Option::unwrap_or_default).collect()
}

/// How far from its parent a note the map leaves out is placed.
const UNPLACED_GAP: f32 = 130.0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unplaced_notes_go_beside_their_parent_and_clear_of_others() {
        let placed = [
            Some(Vec2::ZERO),
            Some(Vec2::new(500.0, 0.0)),
            None,
            None,
            None,
        ];
        // 2 and 3 hang from 1, 4 from 3.
        let parent = [None, Some(0), Some(1), Some(1), Some(3)];
        let homes = place_the_rest(&placed, &parent);
        assert_eq!(homes[1], Vec2::new(500.0, 0.0));
        for n in 2..5 {
            assert!(homes[n].x > 500.0 - 1.0, "away from the hub: {}", homes[n]);
            for m in 0..5 {
                if m != n {
                    assert!(homes[n].distance(homes[m]) > 100.0, "{n} and {m}");
                }
            }
        }
    }
}
