//! Model previews (TimelessXI, ours): a monster or NPC model drawn by its model id for a mod's portrait
//! (`hud.portrait{model = ...}`, a canvas's `portrait` command), with no entity in the zone. The model is loaded from
//! the player's own game files like any other; its actor stands out of sight under the player's feet, is posed (its
//! idle motion) and is drawn only into its portrait, never into the scene.
//!
//! An actor id here is `PREVIEW | model id`. A preview nobody has wanted for `KEEP` is dropped.

use super::{Place, Source, World};
use glam::Vec3;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// The id range of preview actors (unused by entities, furniture (0x0400_xxxx), borders (0x0800_0000), auras
/// (0x2000_0000)...).
pub const PREVIEW: u32 = 0x0600_0000;
const MODEL_MASK: u32 = 0x0fff;
/// How long a preview's actor stays after its last portrait (a page turned back finds it loaded).
const KEEP: Duration = Duration::from_secs(20);
/// How far under the player the previews stand (y is down): out of the scene, in the zone's light.
const BELOW: f32 = 400.0;

pub fn is(id: u32) -> bool {
    id & !MODEL_MASK == PREVIEW
}

pub fn id_of(model: u16) -> u32 {
    PREVIEW | (model as u32 & MODEL_MASK)
}

pub fn model_of(id: u32) -> u16 {
    (id & MODEL_MASK) as u16
}

struct Wanted {
    last: Instant,
    /// Its facing in radians (0 east, as entity headings) and a turn per second.
    turn: f32,
    spin: f32,
}

#[derive(Default)]
pub struct State {
    wanted: HashMap<u32, Wanted>,
    /// The clock the spins turn by.
    t0: Option<Instant>,
}

/// Each frame: the previews the portraits want (id, facing in degrees, degrees a second). A new one's model is
/// asked for (it loads like an NPC's, client/mod.rs's looks); one nobody wants for KEEP is dropped.
pub fn update(w: &mut World, want: &[(u32, f32, f32)]) {
    let now = Instant::now();
    w.previews.t0.get_or_insert(now);
    for &(id, turn, spin) in want.iter().filter(|(id, _, _)| is(*id)) {
        let fresh = !w.previews.wanted.contains_key(&id);
        w.previews.wanted.insert(id, Wanted { last: now, turn: turn.to_radians(), spin: spin.to_radians() });
        if fresh && !w.actors.contains_key(&id) {
            log::info!("preview: model {} (actor {id:#x})", model_of(id));
            w.event_looks.push((id, Source::Npc(crate::dat::npc::NpcLook::Model(model_of(id)))));
        }
    }
    let stale: Vec<u32> =
        w.previews.wanted.iter().filter(|(_, p)| now.duration_since(p.last) > KEEP).map(|(id, _)| *id).collect();
    for id in stale {
        w.previews.wanted.remove(&id);
        w.actors.remove(&id);
    }
}

/// A preview's place: under the player, facing its turn (plus its spin so far), standing (status 0).
pub fn place(w: &World, id: u32) -> Option<Place> {
    let p = w.previews.wanted.get(&id)?;
    let secs = w.previews.t0.map_or(0.0, |t| t.elapsed().as_secs_f32());
    let yaw = p.turn + p.spin * secs;
    Some(Place {
        pos: w.pos + Vec3::new(0.0, BELOW, 0.0),
        forward: Vec3::new(yaw.cos(), 0.0, -yaw.sin()),
        tilt: [0.0; 2],
        status: 0,
        ratio: 0.0,
        size: 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids() {
        assert!(is(id_of(328)));
        assert_eq!(model_of(id_of(328)), 328);
        assert!(!is(0x0400_0001));
        assert!(!is(0x0100_0000));
        assert!(!is(0x2000_0000 | id_of(5)));
        assert_eq!(id_of(0x1fff), PREVIEW | 0x0fff);
    }
}
