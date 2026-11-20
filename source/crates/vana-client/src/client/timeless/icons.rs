//! Timeless XI: the painted Timeless View icons (assets/timeless/icons, 128 x 128 PNG) for spells, abilities, weapon
//! skills and the Misc Book, used by the Timeless interface in place of the game's list icons. Which icon an action
//! gets is assets/timeless/icons/map.tsv, built from the server's tables by tools/timeless/icon_map.py (s = spell id,
//! a = client ability id, m = Misc Book key); actions it doesn't name keep the game's icon.

use std::collections::HashMap;
use std::sync::OnceLock;

#[path = "icon_files.rs"]
mod files;

fn map() -> &'static HashMap<String, &'static str> {
    static MAP: OnceLock<HashMap<String, &'static str>> = OnceLock::new();
    MAP.get_or_init(|| {
        include_str!("../../../assets/timeless/icons/map.tsv")
            .lines()
            .filter(|l| !l.starts_with('#'))
            .filter_map(|l| l.split_once('\t'))
            .map(|(k, v)| (k.to_string(), v.trim()))
            .collect()
    })
}

fn files() -> &'static HashMap<&'static str, &'static [u8]> {
    static FILES: OnceLock<HashMap<&'static str, &'static [u8]>> = OnceLock::new();
    FILES.get_or_init(|| files::FILES.iter().copied().collect())
}

/// Every embedded icon's name.
pub fn names() -> impl Iterator<Item = &'static str> {
    files::FILES.iter().map(|(n, _)| *n)
}

/// The icons a macro's writer can pick from: the Misc Book, gathering, weapon types, job emblems and the elements.
pub fn macro_choices() -> Vec<&'static str> {
    let mut v: Vec<&'static str> = names()
        .filter(|n| {
            n.starts_with("misc_")
                || n.starts_with("helm_")
                || n.starts_with("ws_")
                || n.starts_with("ja_job_")
                || n.ends_with('1') && n.starts_with("sp_")
        })
        .collect();
    v.sort();
    v
}

/// A spell's icon name.
pub fn spell(id: u16) -> Option<&'static str> {
    map().get(&format!("s{id}")).copied()
}

/// An ability's or weapon skill's icon name, by its client ability id (0x200 + the server's for job abilities).
pub fn ability(id: u16) -> Option<&'static str> {
    map().get(&format!("a{id}")).copied()
}

/// A Misc Book action's icon name (book::MISC keys, plus "disengage" and "gear").
pub fn misc(key: &str) -> Option<&'static str> {
    map().get(&format!("m{key}")).copied()
}

/// The pixel size an icon is made at for `pixels_per_point`: about a hotbar slot's size on screen, in steps of 8, so a
/// few sizes cover every scale.
pub fn size_for(pixels_per_point: f32) -> usize {
    (((56.0 * pixels_per_point) / 8.0).ceil() as usize * 8).clamp(40, 128)
}

/// Icon `name` as RGBA at `size` x `size`: decoded, then area-averaged down (egui draws without mipmaps, so a 128 px
/// icon shrunk on the GPU to a 44 px slot would shimmer).
pub fn rgba(name: &str, size: usize) -> Option<(usize, usize, Vec<u8>)> {
    let bytes = files().get(name)?;
    let mut d = png::Decoder::new(std::io::Cursor::new(*bytes));
    d.set_transformations(png::Transformations::normalize_to_color8() | png::Transformations::ALPHA);
    let mut r = d.read_info().ok()?;
    let mut buf = vec![0u8; r.output_buffer_size()?];
    let info = r.next_frame(&mut buf).ok()?;
    let (w, h) = (info.width as usize, info.height as usize);
    buf.truncate(w * h * 4);
    if size >= w || size == 0 {
        return Some((w, h, buf));
    }
    Some((size, size, shrink(&buf, w, h, size)))
}

/// Box filter `src` (w x h RGBA) down to size x size, weighting colour by alpha so transparent corners don't darken
/// the frame's edge.
fn shrink(src: &[u8], w: usize, h: usize, size: usize) -> Vec<u8> {
    let mut out = vec![0u8; size * size * 4];
    let (sx, sy) = (w as f32 / size as f32, h as f32 / size as f32);
    for oy in 0..size {
        let y0 = oy as f32 * sy;
        let y1 = y0 + sy;
        for ox in 0..size {
            let x0 = ox as f32 * sx;
            let x1 = x0 + sx;
            let mut acc = [0f32; 4];
            let mut area = 0f32;
            let mut y = y0.floor() as usize;
            while (y as f32) < y1 && y < h {
                let wy = (y1.min(y as f32 + 1.0) - y0.max(y as f32)).max(0.0);
                let mut x = x0.floor() as usize;
                while (x as f32) < x1 && x < w {
                    let wx = (x1.min(x as f32 + 1.0) - x0.max(x as f32)).max(0.0);
                    let k = wx * wy;
                    let p = &src[(y * w + x) * 4..(y * w + x) * 4 + 4];
                    let a = p[3] as f32 / 255.0;
                    acc[0] += p[0] as f32 * a * k;
                    acc[1] += p[1] as f32 * a * k;
                    acc[2] += p[2] as f32 * a * k;
                    acc[3] += a * k;
                    area += k;
                    x += 1;
                }
                y += 1;
            }
            let o = (oy * size + ox) * 4;
            if acc[3] > 0.0 {
                out[o] = (acc[0] / acc[3]).round().clamp(0.0, 255.0) as u8;
                out[o + 1] = (acc[1] / acc[3]).round().clamp(0.0, 255.0) as u8;
                out[o + 2] = (acc[2] / acc[3]).round().clamp(0.0, 255.0) as u8;
            }
            out[o + 3] = (acc[3] / area.max(1e-6) * 255.0).round().clamp(0.0, 255.0) as u8;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_mapped_icon_is_embedded() {
        for v in map().values() {
            assert!(files().contains_key(v), "{v} is mapped but not embedded");
        }
    }

    #[test]
    fn icons_decode_and_shrink() {
        let (w, h, px) = rgba("misc_attack", 0).expect("decodes");
        assert_eq!((w, h, px.len()), (128, 128, 128 * 128 * 4));
        let (w, h, px) = rgba("sp_fire1", 48).expect("shrinks");
        assert_eq!((w, h, px.len()), (48, 48, 48 * 48 * 4));
        assert!(px.chunks(4).any(|p| p[3] == 255), "the inside stays opaque");
    }

    #[test]
    fn known_actions_have_icons() {
        assert_eq!(spell(1), Some("sp_cure1"));
        assert_eq!(misc("map"), Some("misc_map"));
        assert!(misc("disengage").is_some());
        assert!(ability(0x200 + 16).is_some() || ability(547).is_some());
    }
}
