//! A zone change fades to black, then the new zone fades in once it is ready. While the screen darkens, the old zone's
//! last frame is held: the camera has already moved to the new zone.
//!
//! Ours, Settings' Zone Changes Soft: the held frame stays on screen the whole time, dimmed almost to black (DIM at
//! full fade) instead of going fully black, and the new zone fades in under it once it is ready (the held frame kept
//! that dark, so the new zone rises out of the near-black); the hold ends when the crossfade has. A zone change with no frame to hold (the login) fades through black as retail.

#[derive(Default, Clone, Copy, Debug)]
pub struct Hold {
    /// The frame is held: the last one drawn, copied from the scene texture before the next frame draws.
    pub captured: bool,
}

/// A zone change starts (a zone line touched, or the server's 0x00B): hold the frame once a world frame was drawn and
/// the screen is not black yet.
pub fn arm(h: &mut Option<Hold>, drawn: bool, fade: f32) {
    if drawn && fade < 1.0 && h.is_none() {
        log::info!("zone fade: holding the frame");
        *h = Some(Hold::default());
    }
}

/// How dark the soft hold gets at full fade (the held frame at 1 - DIM).
pub const DIM: f32 = 0.9;

/// Each frame after the fade: arms while a zone line waits; the hold ends once the screen is black (retail), or once
/// the new zone has faded in under it (soft: `fading_in` while the fade heads back to 0).
pub fn step(h: &mut Option<Hold>, zoning: bool, drawn: bool, fade: f32, soft: bool, fading_in: bool) {
    if zoning {
        arm(h, drawn, fade);
    }
    let captured = h.is_some_and(|x| x.captured);
    if !soft && captured && fade >= 1.0 {
        log::info!("zone fade: black");
        *h = None;
    }
    if soft && captured && fading_in && fade <= 0.0 && !zoning {
        log::info!("zone fade: crossfaded in");
        *h = None;
    }
}

/// What a soft hold draws this frame: the held frame alone at a level (still waiting), or over the new zone at an
/// alpha (crossfading in); None when nothing is held (or retail).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Soft {
    Alone(f32),
    Over(f32),
}

pub fn soft(h: Option<Hold>, soft: bool, fade: f32, fading_in: bool) -> Option<Soft> {
    if !soft || !h.is_some_and(|x| x.captured) {
        return None;
    }
    Some(if fading_in { Soft::Over(fade.clamp(0.0, 1.0)) } else { Soft::Alone(1.0 - DIM * fade.clamp(0.0, 1.0)) })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn holds_until_black() {
        let mut h = None;
        // Not zoning, or no world drawn yet (the login): nothing.
        step(&mut h, false, true, 0.0, false, false);
        step(&mut h, true, false, 0.0, false, false);
        assert!(h.is_none());
        step(&mut h, true, true, 0.0, false, false);
        assert!(h.is_some());
        // Not captured yet: kept even at black.
        step(&mut h, true, true, 1.0, false, false);
        assert!(h.is_some());
        h.as_mut().unwrap().captured = true;
        step(&mut h, true, true, 0.6, false, false);
        assert!(h.is_some());
        step(&mut h, true, true, 1.0, false, false);
        assert!(h.is_none());
        // Still waiting on the zone line, already black: no new hold.
        step(&mut h, true, true, 1.0, false, false);
        assert!(h.is_none());
    }

    #[test]
    fn soft_holds_until_crossfaded() {
        let mut h = None;
        step(&mut h, true, true, 0.0, true, false);
        h.as_mut().unwrap().captured = true;
        assert_eq!(soft(h, true, 0.0, false), Some(Soft::Alone(1.0)));
        // Full fade while the zone loads: dimmed, not black, and still held.
        step(&mut h, true, true, 1.0, true, false);
        step(&mut h, false, true, 1.0, true, false);
        assert!(h.is_some());
        assert_eq!(soft(h, true, 1.0, false), Some(Soft::Alone(1.0 - DIM)));
        // Ready: the new zone fades in under it.
        step(&mut h, false, true, 0.5, true, true);
        assert_eq!(soft(h, true, 0.5, true), Some(Soft::Over(0.5)));
        step(&mut h, false, true, 0.0, true, true);
        assert!(h.is_none());
        assert_eq!(soft(h, true, 0.0, true), None);
        // Retail never draws a soft hold.
        assert_eq!(soft(Some(Hold { captured: true }), false, 0.5, false), None);
    }
}
