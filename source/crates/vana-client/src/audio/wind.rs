//! Ours (Settings' Weather Modern, client/storm.rs): a storm's howling wind, made here rather than taken from the
//! game's files: eight seconds of noise through a slowly sweeping resonant filter (the howl) over a low rumble, as a
//! looping SeWave (PCM, loop from the start) the mixer plays like any effect. Its level follows the gusts.

use std::sync::Arc;

pub const RATE: u32 = 22_050;
const SECONDS: u32 = 8;

/// The loop's samples (mono, 16-bit): seamless, every slow wave a whole number of times round in the loop.
pub fn samples() -> Vec<i16> {
    let n = (RATE * SECONDS) as usize;
    let tau = std::f32::consts::TAU;
    let mut seed: u32 = 0x9e37_79b9;
    let mut noise = || {
        seed ^= seed << 13;
        seed ^= seed >> 17;
        seed ^= seed << 5;
        (seed as f32 / u32::MAX as f32) * 2.0 - 1.0
    };
    let mut out = vec![0f32; n];
    let (mut low, mut band, mut lp) = (0f32, 0f32, 0f32);
    // Run the filters a second first so the loop starts settled.
    let warm = RATE as usize;
    for i in 0..warm + n {
        let t = (i % n) as f32 / RATE as f32;
        let phase = t / SECONDS as f32;
        // The howl's pitch wanders (2 and 3 times round in the loop), 230..680 Hz.
        let f = 430.0 + 160.0 * (tau * 2.0 * phase).sin() + 90.0 * (tau * 3.0 * phase + 1.1).sin();
        let g = 2.0 * (std::f32::consts::PI * f / RATE as f32).sin();
        let q = 0.12;
        let x = noise();
        // A state-variable filter: its band output is the howl.
        low += g * band;
        let high = x - low - q * band;
        band += g * high;
        // A low rumble under it.
        lp += 0.03 * (x - lp);
        // The wind swells and eases (1, 2 and 5 times round).
        let swell =
            0.6 + 0.25 * (tau * phase).sin() + 0.1 * (tau * 2.0 * phase + 0.7).sin() + 0.05 * (tau * 5.0 * phase).sin();
        if i >= warm {
            out[i - warm] = (band * 0.55 + lp * 2.2) * swell;
        }
    }
    // The end crossfaded into the start (the filters' state differs there).
    let fade = (RATE / 3) as usize;
    for k in 0..fade {
        let a = k as f32 / fade as f32;
        out[k] = out[k] * a + out[n - fade + k] * (1.0 - a);
    }
    out.truncate(n - fade);
    let peak = out.iter().fold(0f32, |m, v| m.max(v.abs())).max(1e-6);
    out.iter().map(|v| (v / peak * 0.7 * 32767.0) as i16).collect()
}

/// The loop as a SeWave file (PCM16 little-endian, mono, looping from the start), as format::Header::parse reads it.
pub fn sewave() -> Arc<[u8]> {
    let s = samples();
    let data = 0x30usize;
    let mut b = vec![0u8; data + s.len() * 2];
    b[..7].copy_from_slice(b"SeWave\0");
    let put = |b: &mut [u8], o: usize, v: u32| b[o..o + 4].copy_from_slice(&v.to_le_bytes());
    put(&mut b, 0x0c, 1); // PCM16 little-endian
    put(&mut b, 0x10, 0xfe00); // id
    put(&mut b, 0x14, s.len() as u32); // length in frames
    put(&mut b, 0x18, 0); // loop from the start
    put(&mut b, 0x1c, RATE);
    put(&mut b, 0x20, 0);
    put(&mut b, 0x24, data as u32);
    b[0x28] = 0x7f; // volume
    b[0x29] = 0;
    b[0x2a] = 1; // channels
    b[0x2b] = 16; // bits
    for (i, v) in s.iter().enumerate() {
        b[data + 2 * i..data + 2 * i + 2].copy_from_slice(&v.to_le_bytes());
    }
    Arc::from(b)
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_looping_sewave() {
        let b = super::sewave();
        let h = crate::audio::format::Header::parse(&b).expect("parses");
        assert_eq!((h.channels, h.rate, h.loop_start, h.data), (1, super::RATE, Some(0), 0x30));
        assert!(matches!(h.codec, crate::audio::format::Codec::Pcm16 { big_endian: false }));
        assert_eq!(h.length as usize * 2 + 0x30, b.len());
        let s = super::samples();
        // Loud enough, never clipped, and the seam is smooth.
        let peak = s.iter().map(|v| v.unsigned_abs()).max().unwrap();
        assert!(peak > 20_000 && peak < 32_767);
        let seam = (s[s.len() - 1] as i32 - s[0] as i32).abs();
        assert!(seam < 6_000, "seam jump {seam}");
        let d = crate::audio::format::Decoder::new(b).expect("decodes");
        assert!(d.loops);
    }
}
