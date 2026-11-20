//! Timeless XI: HD game icons. The Timeless interface draws the game's own icons (status, items, spells and abilities,
//! read from the player's DATs) through this upscaler, so they stay the classic pictures, only sharper and smoother at
//! today's resolutions. Nothing is shipped: every icon comes from the player's own files and is scaled when first
//! drawn.
//!
//! The pipeline, on premultiplied colour so transparent edges never darken:
//! 1. An edge pass: Scale2x (EPX) three times with a colour-tolerant match, then halved by area: 4x with crisp
//!    outlines and anti-aliased diagonals, the classic pixel-art shape kept.
//! 2. A smooth pass: Catmull-Rom bicubic at 4x, for the painted gradients inside the icons.
//! 3. A per-pixel blend of the two by the source's local edge strength (outlines from 1, insides from 2).
//! 4. A light unsharp mask, so the result reads sharp rather than soft.
//! 5. An area (box) resample from that 4x master straight to the icon's size on screen (its size times the screen's
//!    pixels per point): egui makes no mipmaps, so the GPU's linear filter shrinking a large texture would throw most
//!    of the detail away and shimmer; done here, every master pixel counts and the GPU draws about 1:1.

/// The master's scale over the source.
const MASTER: usize = 4;

/// A premultiplied pixel, channels in 0..=1.
type Px = [f32; 4];

/// The factor (the icon's size over its source's) for a screen of `pixels_per_point` (egui's, the interface size
/// included): the icon drawn at its own size in points, rounded to a twentieth so a few textures serve every size.
pub fn factor_for(pixels_per_point: f32) -> f32 {
    ((pixels_per_point * 20.0).round() / 20.0).clamp(0.5, MASTER as f32)
}

/// The icon `w` x `h` (straight RGBA) at `scale` (1..=4) times its size: (w, h, straight RGBA).
#[cfg_attr(not(test), allow(dead_code))]
pub fn upscale(w: usize, h: usize, rgba: &[u8], scale: usize) -> Option<(usize, usize, Vec<u8>)> {
    upscale_by(w, h, rgba, scale as f32)
}

/// The icon `w` x `h` (straight RGBA) at `factor` (0.5..=4) times its size, through the 4x master: (w, h, RGBA).
pub fn upscale_by(w: usize, h: usize, rgba: &[u8], factor: f32) -> Option<(usize, usize, Vec<u8>)> {
    if w == 0 || h == 0 || rgba.len() < w * h * 4 {
        return None;
    }
    let factor = factor.clamp(0.5, MASTER as f32);
    let src: Vec<Px> = rgba.chunks_exact(4).take(w * h).map(premultiply).collect();
    let (mw, mh) = (w * MASTER, h * MASTER);

    let edge = epx_master(&src, w, h);
    let smooth = bicubic(&src, w, h, MASTER);
    let strength = edge_strength(&src, w, h);
    let mut master = vec![[0.0; 4]; mw * mh];
    for y in 0..mh {
        for x in 0..mw {
            // The source's edge strength at this output pixel, bilinear so the blend has no blocks.
            let k = sample_bilinear(&strength, w, h, (x as f32 + 0.5) / MASTER as f32 - 0.5, (y as f32 + 0.5) / MASTER as f32 - 0.5);
            let t = smoothstep(0.06, 0.22, k);
            let (a, b) = (smooth[y * mw + x], edge[y * mw + x]);
            master[y * mw + x] = std::array::from_fn(|c| a[c] + (b[c] - a[c]) * t);
        }
    }
    let master = unsharp(&master, mw, mh, 0.45);
    let (ow, oh) = (((w as f32 * factor).round() as usize).max(1), ((h as f32 * factor).round() as usize).max(1));
    let out = if (ow, oh) == (mw, mh) { master } else { area_resample(&master, mw, mh, ow, oh) };
    Some((ow, oh, out.iter().flat_map(|p| unpremultiply(*p)).collect()))
}

fn premultiply(p: &[u8]) -> Px {
    let a = p[3] as f32 / 255.0;
    [p[0] as f32 / 255.0 * a, p[1] as f32 / 255.0 * a, p[2] as f32 / 255.0 * a, a]
}

fn unpremultiply(p: Px) -> [u8; 4] {
    let a = p[3].clamp(0.0, 1.0);
    if a <= 1.0 / 512.0 {
        return [0; 4];
    }
    let c = |v: f32| ((v / a).clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
    [c(p[0]), c(p[1]), c(p[2]), (a * 255.0 + 0.5) as u8]
}

/// Two pixels look alike to the eye (hqx's YUV thresholds, on premultiplied colour, plus alpha).
fn alike(a: &Px, b: &Px) -> bool {
    let (dr, dg, db) = (a[0] - b[0], a[1] - b[1], a[2] - b[2]);
    let y = 0.299 * dr + 0.587 * dg + 0.114 * db;
    let u = -0.169 * dr - 0.331 * dg + 0.5 * db;
    let v = 0.5 * dr - 0.419 * dg - 0.081 * db;
    y.abs() <= 48.0 / 255.0 && u.abs() <= 7.0 / 255.0 && v.abs() <= 6.0 / 255.0 && (a[3] - b[3]).abs() <= 0.15
}

/// How different two pixels are (luma and alpha), 0..~1.
fn distance(a: &Px, b: &Px) -> f32 {
    let y = |p: &Px| 0.299 * p[0] + 0.587 * p[1] + 0.114 * p[2];
    (y(a) - y(b)).abs().max((a[3] - b[3]).abs())
}

fn at(img: &[Px], w: usize, h: usize, x: isize, y: isize) -> Px {
    let x = x.clamp(0, w as isize - 1) as usize;
    let y = y.clamp(0, h as isize - 1) as usize;
    img[y * w + x]
}

/// Scale2x (EPX): each pixel to four, a corner taking a neighbour's colour where two neighbours meet across it.
fn scale2x(img: &[Px], w: usize, h: usize) -> Vec<Px> {
    let ow = w * 2;
    let mut out = vec![[0.0; 4]; ow * h * 2];
    for y in 0..h as isize {
        for x in 0..w as isize {
            let e = at(img, w, h, x, y);
            let b = at(img, w, h, x, y - 1);
            let d = at(img, w, h, x - 1, y);
            let f = at(img, w, h, x + 1, y);
            let hh = at(img, w, h, x, y + 1);
            let (mut e0, mut e1, mut e2, mut e3) = (e, e, e, e);
            if !alike(&b, &hh) && !alike(&d, &f) {
                if alike(&d, &b) {
                    e0 = d;
                }
                if alike(&b, &f) {
                    e1 = f;
                }
                if alike(&d, &hh) {
                    e2 = d;
                }
                if alike(&hh, &f) {
                    e3 = f;
                }
            }
            let (ox, oy) = (x as usize * 2, y as usize * 2);
            out[oy * ow + ox] = e0;
            out[oy * ow + ox + 1] = e1;
            out[(oy + 1) * ow + ox] = e2;
            out[(oy + 1) * ow + ox + 1] = e3;
        }
    }
    out
}

/// The edge pass: Scale2x to 8x, halved by area to the 4x master.
fn epx_master(src: &[Px], w: usize, h: usize) -> Vec<Px> {
    let x2 = scale2x(src, w, h);
    let x4 = scale2x(&x2, w * 2, h * 2);
    let x8 = scale2x(&x4, w * 4, h * 4);
    let (ow, oh, iw) = (w * MASTER, h * MASTER, w * 8);
    let mut out = vec![[0.0; 4]; ow * oh];
    for y in 0..oh {
        for x in 0..ow {
            let s = |dx: usize, dy: usize| x8[(y * 2 + dy) * iw + x * 2 + dx];
            let (a, b, c, d) = (s(0, 0), s(1, 0), s(0, 1), s(1, 1));
            out[y * ow + x] = std::array::from_fn(|k| (a[k] + b[k] + c[k] + d[k]) * 0.25);
        }
    }
    out
}

fn catmull_rom(t: f32) -> [f32; 4] {
    let (t2, t3) = (t * t, t * t * t);
    [
        -0.5 * t3 + t2 - 0.5 * t,
        1.5 * t3 - 2.5 * t2 + 1.0,
        -1.5 * t3 + 2.0 * t2 + 0.5 * t,
        0.5 * t3 - 0.5 * t2,
    ]
}

/// Catmull-Rom bicubic at `s` times, clamped so it never rings past the premultiplied range.
fn bicubic(src: &[Px], w: usize, h: usize, s: usize) -> Vec<Px> {
    let (ow, oh) = (w * s, h * s);
    let mut out = vec![[0.0; 4]; ow * oh];
    for y in 0..oh {
        let fy = (y as f32 + 0.5) / s as f32 - 0.5;
        let (iy, ty) = (fy.floor() as isize, fy - fy.floor());
        let wy = catmull_rom(ty);
        for x in 0..ow {
            let fx = (x as f32 + 0.5) / s as f32 - 0.5;
            let (ix, tx) = (fx.floor() as isize, fx - fx.floor());
            let wx = catmull_rom(tx);
            let mut acc = [0.0f32; 4];
            for (j, wyj) in wy.iter().enumerate() {
                for (i, wxi) in wx.iter().enumerate() {
                    let p = at(src, w, h, ix + i as isize - 1, iy + j as isize - 1);
                    let k = wxi * wyj;
                    for c in 0..4 {
                        acc[c] += p[c] * k;
                    }
                }
            }
            out[y * ow + x] = clamp_px(acc);
        }
    }
    out
}

fn clamp_px(p: Px) -> Px {
    let a = p[3].clamp(0.0, 1.0);
    [p[0].clamp(0.0, a), p[1].clamp(0.0, a), p[2].clamp(0.0, a), a]
}

/// Each source pixel's largest difference from its eight neighbours.
fn edge_strength(src: &[Px], w: usize, h: usize) -> Vec<f32> {
    let mut out = vec![0.0; w * h];
    for y in 0..h as isize {
        for x in 0..w as isize {
            let e = at(src, w, h, x, y);
            let mut m = 0.0f32;
            for dy in -1..=1 {
                for dx in -1..=1 {
                    if dx != 0 || dy != 0 {
                        m = m.max(distance(&e, &at(src, w, h, x + dx, y + dy)));
                    }
                }
            }
            out[y as usize * w + x as usize] = m;
        }
    }
    out
}

fn sample_bilinear(v: &[f32], w: usize, h: usize, x: f32, y: f32) -> f32 {
    let g = |x: isize, y: isize| v[y.clamp(0, h as isize - 1) as usize * w + x.clamp(0, w as isize - 1) as usize];
    let (x0, y0) = (x.floor() as isize, y.floor() as isize);
    let (tx, ty) = (x - x.floor(), y - y.floor());
    let top = g(x0, y0) + (g(x0 + 1, y0) - g(x0, y0)) * tx;
    let bottom = g(x0, y0 + 1) + (g(x0 + 1, y0 + 1) - g(x0, y0 + 1)) * tx;
    top + (bottom - top) * ty
}

fn smoothstep(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// `img` plus `amount` times its difference from a 3 x 3 Gaussian blur.
fn unsharp(img: &[Px], w: usize, h: usize, amount: f32) -> Vec<Px> {
    const K: [f32; 3] = [0.25, 0.5, 0.25];
    let mut out = vec![[0.0; 4]; w * h];
    for y in 0..h as isize {
        for x in 0..w as isize {
            let mut blur = [0.0f32; 4];
            for (j, kj) in K.iter().enumerate() {
                for (i, ki) in K.iter().enumerate() {
                    let p = at(img, w, h, x + i as isize - 1, y + j as isize - 1);
                    for c in 0..4 {
                        blur[c] += p[c] * ki * kj;
                    }
                }
            }
            let p = img[y as usize * w + x as usize];
            out[y as usize * w + x as usize] = clamp_px(std::array::from_fn(|c| p[c] + (p[c] - blur[c]) * amount));
        }
    }
    out
}

/// A box (area-weighted) resample from `w` x `h` to `ow` x `oh` (smaller).
fn area_resample(img: &[Px], w: usize, h: usize, ow: usize, oh: usize) -> Vec<Px> {
    let (sx, sy) = (w as f32 / ow as f32, h as f32 / oh as f32);
    let mut out = vec![[0.0; 4]; ow * oh];
    for oy in 0..oh {
        let (y0, y1) = (oy as f32 * sy, (oy + 1) as f32 * sy);
        for ox in 0..ow {
            let (x0, x1) = (ox as f32 * sx, (ox + 1) as f32 * sx);
            let mut acc = [0.0f32; 4];
            let mut total = 0.0;
            for y in y0.floor() as usize..(y1.ceil() as usize).min(h) {
                let wy = (y1.min(y as f32 + 1.0) - y0.max(y as f32)).max(0.0);
                for x in x0.floor() as usize..(x1.ceil() as usize).min(w) {
                    let k = wy * (x1.min(x as f32 + 1.0) - x0.max(x as f32)).max(0.0);
                    let p = img[y * w + x];
                    for c in 0..4 {
                        acc[c] += p[c] * k;
                    }
                    total += k;
                }
            }
            out[oy * ow + ox] = if total > 0.0 { clamp_px(acc.map(|v| v / total)) } else { [0.0; 4] };
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn img(w: usize, h: usize, f: impl Fn(usize, usize) -> [u8; 4]) -> Vec<u8> {
        (0..h).flat_map(|y| (0..w).map(move |x| (x, y))).flat_map(|(x, y)| f(x, y)).collect()
    }

    #[test]
    fn sizes_and_scale() {
        let src = img(32, 32, |_, _| [200, 100, 50, 255]);
        for s in 2..=4 {
            let (w, h, out) = upscale(32, 32, &src, s).unwrap();
            assert_eq!((w, h, out.len()), (32 * s, 32 * s, 32 * s * 32 * s * 4));
        }
        assert_eq!(factor_for(0.72), 0.7);
        assert_eq!(factor_for(9.0), 4.0);
        let (w, h, _) = upscale_by(32, 32, &src, 0.7).unwrap();
        assert_eq!((w, h), (22, 22));
        assert!(upscale(0, 4, &[], 2).is_none());
        assert!(upscale(4, 4, &[0; 10], 2).is_none());
    }

    #[test]
    fn flat_stays_flat_and_clear_stays_clear() {
        let src = img(8, 8, |x, _| if x < 4 { [10, 200, 30, 255] } else { [0, 0, 0, 0] });
        let (w, _, out) = upscale(8, 8, &src, 4).unwrap();
        // Deep inside the opaque half: the colour unchanged; deep inside the clear half: clear.
        let px = |x: usize, y: usize| &out[(y * w + x) * 4..(y * w + x) * 4 + 4];
        let inside = px(4, 16);
        assert!((inside[0] as i32 - 10).abs() <= 2 && (inside[1] as i32 - 200).abs() <= 2 && inside[3] == 255);
        assert_eq!(px(28, 16)[3], 0);
    }

    #[test]
    fn no_dark_halo_on_a_transparent_edge() {
        // A white dot on clear: every partly clear pixel stays white (premultiplied work, no black fringe).
        let src = img(8, 8, |x, y| if (2..6).contains(&x) && (2..6).contains(&y) { [255, 255, 255, 255] } else { [0, 0, 0, 0] });
        let (_, _, out) = upscale(8, 8, &src, 4).unwrap();
        for p in out.chunks_exact(4).filter(|p| p[3] > 8) {
            assert!(p[0] > 240 && p[1] > 240 && p[2] > 240, "{p:?}");
        }
    }

    #[test]
    fn diagonal_is_smoothed() {
        // A hard staircase diagonal: the upscale has in-between alpha along it (anti-aliased), not just 0 and 255.
        let src = img(8, 8, |x, y| if x >= y { [255, 255, 255, 255] } else { [0, 0, 0, 0] });
        let (_, _, out) = upscale(8, 8, &src, 4).unwrap();
        assert!(out.chunks_exact(4).any(|p| p[3] > 40 && p[3] < 215));
    }
}
