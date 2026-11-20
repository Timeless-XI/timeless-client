//! Timeless XI: `vana-client reel-shot <shot>`: frames of a Timeless title reel shot (client/timeless/title.rs)
//! rendered offscreen with the nostalgia filter and the logo laid over them, for checking a shot's framing without
//! logging out to the title. Each move is sampled `steps` times; the match shot's last frame shows its light.

use crate::client::tl_title::{Reel, filter_preset};
use crate::dat::title::{fov_y, sample, up_of};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

const LOGO_PNG: &[u8] = include_bytes!("../assets/timeless/title_logo.png");

#[allow(clippy::too_many_arguments)]
pub fn reel_shot(
    shot_id: &str,
    out_dir: &Path,
    steps: u32,
    width: u32,
    height: u32,
    filter: Option<&str>,
    logo: bool,
) -> Result<()> {
    let reel = Reel::load();
    let shot = reel.shot(shot_id).with_context(|| {
        let ids: Vec<&str> = reel.shots.iter().map(|s| s.id.as_str()).collect();
        format!("no shot {shot_id} (the reel has {})", ids.join(", "))
    })?;
    let (title, _) = shot.movie();
    let scene = crate::load_zone_scene(shot.zone)?;
    let minute = shot.hour.rem_euclid(24.0) * 60.0;
    let lighting = crate::load_zone_weather(shot.zone)
        .ok()
        .and_then(|w| w.at(&shot.weather, minute))
        .map(|e| crate::dat::weather::Lighting::from_env(&e, minute));
    std::fs::create_dir_all(out_dir)?;
    let mut frames = Vec::new();
    for (m, path) in title.paths.iter().enumerate() {
        if path.keys.is_empty() {
            continue;
        }
        for i in 0..steps.max(1) {
            let t = if steps <= 1 { 0.0 } else { i as f32 / (steps - 1) as f32 };
            let k = sample(path, t);
            let cam = crate::render::Camera {
                eye: k.eye,
                target: k.target,
                up: up_of(k.eye, k.target, k.roll),
                fov_y: fov_y(k.focal),
            };
            frames.push((cam, out_dir.join(format!("{shot_id}_m{}_{i}.png", m + 1))));
        }
    }
    let grade = filter_preset(filter.unwrap_or(&reel.filter));
    let last = frames.len().saturating_sub(1);
    let crystal = reel.logo.crystal_uv([width as f32, height as f32]);
    let matched = shot.logo.as_deref() == Some("match");
    let list: Vec<(&crate::render::Camera, &Path, Option<crate::dat::weather::Lighting>)> =
        frames.iter().map(|(c, p)| (c, p.as_path(), lighting)).collect();
    let mut pass: Option<crate::render::nostalgia::Nostalgia> = None;
    let mut n = 0usize;
    crate::render::render_pngs_post(
        &scene,
        &list,
        width,
        height,
        &|_, _| Vec::new(),
        &mut |gpu, enc, (view, tex), _, _, _, _| {
            if let Some(g) = &grade {
                let p = pass.get_or_insert_with(|| crate::render::nostalgia::Nostalgia::new(gpu, tex.format()));
                let flare = if matched && n == last { [crystal[0], crystal[1], 0.7] } else { [0.0; 3] };
                p.draw(gpu, enc, view, tex, [width, height], g, flare);
            }
            n += 1;
        },
    )?;
    if logo {
        for (_, p) in &frames {
            lay_logo(p, &reel.logo)?;
        }
    }
    println!("{shot_id}: zone {}, {} frames in {}", shot.zone, frames.len(), out_dir.display());
    Ok(())
}

fn decode(bytes: &[u8]) -> Result<(usize, usize, Vec<u8>)> {
    let mut d = png::Decoder::new(std::io::Cursor::new(bytes));
    d.set_transformations(png::Transformations::normalize_to_color8() | png::Transformations::ALPHA);
    let mut r = d.read_info()?;
    let mut buf = vec![0u8; r.output_buffer_size().context("png size")?];
    let info = r.next_frame(&mut buf)?;
    buf.truncate(info.buffer_size());
    Ok((info.width as usize, info.height as usize, buf))
}

/// The logo over a frame where the title puts it (bilinear, straight alpha).
fn lay_logo(file: &PathBuf, place: &crate::client::tl_title::LogoPlace) -> Result<()> {
    let (w, h, mut img) = decode(&std::fs::read(file)?)?;
    let (lw, lh, logo) = decode(LOGO_PNG)?;
    let r = place.rect([w as f32, h as f32]);
    let px = |x: usize, y: usize| -> [f32; 4] {
        let o = (y.min(lh - 1) * lw + x.min(lw - 1)) * 4;
        [logo[o] as f32, logo[o + 1] as f32, logo[o + 2] as f32, logo[o + 3] as f32]
    };
    let (x0, y0) = (r[0].floor().max(0.0) as usize, r[1].floor().max(0.0) as usize);
    let (x1, y1) = (((r[0] + r[2]).ceil() as usize).min(w), ((r[1] + r[3]).ceil() as usize).min(h));
    for y in y0..y1 {
        for x in x0..x1 {
            let u = ((x as f32 + 0.5 - r[0]) / r[2] * lw as f32 - 0.5).max(0.0);
            let v = ((y as f32 + 0.5 - r[1]) / r[3] * lh as f32 - 0.5).max(0.0);
            let (ux, vy) = (u.floor() as usize, v.floor() as usize);
            let (fx, fy) = (u.fract(), v.fract());
            let (a, b, c, d) = (px(ux, vy), px(ux + 1, vy), px(ux, vy + 1), px(ux + 1, vy + 1));
            let s: [f32; 4] = std::array::from_fn(|i| {
                (a[i] * (1.0 - fx) + b[i] * fx) * (1.0 - fy) + (c[i] * (1.0 - fx) + d[i] * fx) * fy
            });
            let al = s[3] / 255.0;
            let o = (y * w + x) * 4;
            for i in 0..3 {
                img[o + i] = (img[o + i] as f32 * (1.0 - al) + s[i] * al).round() as u8;
            }
        }
    }
    let f = std::fs::File::create(file)?;
    let mut e = png::Encoder::new(std::io::BufWriter::new(f), w as u32, h as u32);
    e.set_color(png::ColorType::Rgba);
    e.set_depth(png::BitDepth::Eight);
    e.write_header()?.write_image_data(&img)?;
    Ok(())
}
