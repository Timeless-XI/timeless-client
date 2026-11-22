//! Timeless XI: the title screen's nostalgia filter, one full-screen pass over the flyover's finished frame (after
//! its crossfade and fade): a faded-print grade (lifted blacks, split toning, less saturation), halation (a warm glow
//! that bleeds around bright light, as on film), soft focus toward the edges, a vignette, fine moving grain, a
//! touch of exposure flicker and gate weave. It also draws the match cut's light at the logo crystal.

use super::Gpu;
use super::Put;

const SHADER: &str = r#"
struct U {
    lift: vec4<f32>,         // rgb lift, saturation
    shadow: vec4<f32>,       // shadow tint rgb, contrast
    high: vec4<f32>,         // highlight tint rgb, halation
    halo: vec4<f32>,         // halation tint rgb, vignette
    misc: vec4<f32>,         // grain, soft edges, weave, flicker
    time: vec4<f32>,         // seconds, texel w, texel h, aspect
    flare: vec4<f32>,        // uv x, uv y, amount, 0
    area: vec4<f32>,         // the drawn part of the texture (uv), 0, 0
};
@group(0) @binding(0) var<uniform> u: U;
@group(0) @binding(1) var tex: texture_2d<f32>;
@group(0) @binding(2) var samp: sampler;
struct VOut { @builtin(position) clip: vec4<f32>, @location(0) uv: vec2<f32> };
@vertex fn vs(@builtin(vertex_index) i: u32) -> VOut {
    let p = vec2<f32>(f32((i << 1u) & 2u), f32(i & 2u));
    var o: VOut;
    o.clip = vec4<f32>(p * 2.0 - 1.0, 0.0, 1.0);
    o.uv = vec2<f32>(p.x, 1.0 - p.y);
    return o;
}
fn hash(p: vec2<f32>) -> f32 {
    let q = fract(p * vec2<f32>(0.1031, 0.1030));
    let r = q + dot(q, q.yx + 33.33);
    return fract((r.x + r.y) * r.x);
}
fn luma(c: vec3<f32>) -> f32 { return dot(c, vec3<f32>(0.2126, 0.7152, 0.0722)); }
// A sample of the drawn part only (screen uv 0..1), kept a texel inside its edge.
fn at(p: vec2<f32>) -> vec3<f32> {
    let q = clamp(p * u.area.xy, u.time.yz * 0.5, u.area.xy - u.time.yz * 0.5);
    return textureSampleLevel(tex, samp, q, 0.0).rgb;
}
@fragment fn fs(i: VOut) -> @location(0) vec4<f32> {
    let t = u.time.x;
    let px = u.time.yz;
    // Gate weave: the whole picture drifts by a fraction of a pixel, slowly.
    let weave = u.misc.z * px * vec2<f32>(sin(t * 1.7) + 0.5 * sin(t * 4.3), cos(t * 1.3) * 0.6);
    let uv = i.uv + weave;
    let d = uv - vec2<f32>(0.5);
    let r2 = dot(d * vec2<f32>(u.time.w, 1.0), d * vec2<f32>(u.time.w, 1.0)) / (0.25 * (u.time.w * u.time.w + 1.0));
    // Soft focus toward the edges: a small ring of taps that widens with the distance from the centre.
    let spread = u.misc.y * smoothstep(0.25, 1.0, r2) * 3.0;
    var col = at(uv);
    if (spread > 0.01) {
        var acc = col;
        for (var k = 0; k < 6; k = k + 1) {
            let a = f32(k) * 1.0472;
            acc = acc + at(uv + vec2<f32>(cos(a), sin(a)) * px * spread);
        }
        col = acc / 7.0;
    }
    // Halation: bright parts glow outward in a warm (or cool) tint.
    var glow = vec3<f32>(0.0);
    let hr = 7.0;
    for (var k = 0; k < 12; k = k + 1) {
        let a = f32(k) * 0.5236 + 0.26;
        let ring = select(1.0, 2.2, (k & 1) == 1);
        let s = at(uv + vec2<f32>(cos(a), sin(a)) * px * hr * ring * 2.0);
        glow = glow + max(s - vec3<f32>(0.55), vec3<f32>(0.0));
    }
    col = col + glow / 12.0 * u.high.w * 2.2 * u.halo.rgb;
    // The grade: contrast about mid grey, then saturation, split toning by brightness, and lifted blacks.
    col = (col - 0.5) * u.shadow.w + 0.5;
    let y = luma(col);
    col = mix(vec3<f32>(y), col, u.lift.w);
    let sh = 1.0 - smoothstep(0.0, 0.5, y);
    let hi = smoothstep(0.45, 1.0, y);
    col = col + u.shadow.rgb * sh + u.high.rgb * hi;
    col = u.lift.rgb + col * (1.0 - u.lift.rgb);
    // Exposure flicker, as an old projector's.
    col = col * (1.0 + u.misc.w * (hash(vec2<f32>(floor(t * 12.0), 7.0)) - 0.5) * 2.0);
    // Vignette.
    col = col * mix(1.0, smoothstep(1.45, 0.25, r2), u.halo.w);
    // The match cut's light at the logo crystal: a soft core and a wide bloom, cool white.
    if (u.flare.z > 0.0) {
        let fd = (i.uv - u.flare.xy) * vec2<f32>(u.time.w, 1.0);
        let r = length(fd);
        let core = exp(-r * r / 0.0009);
        let bloom = exp(-r * r / 0.03);
        let rays = exp(-abs(fd.y) * 260.0) * exp(-abs(fd.x) * 5.0) + exp(-abs(fd.x) * 260.0) * exp(-abs(fd.y) * 7.0);
        let light = vec3<f32>(0.80, 0.90, 1.0) * (core * 1.6 + bloom * 0.55 + rays * 0.35) * u.flare.z;
        col = col + light + vec3<f32>(0.06, 0.08, 0.12) * u.flare.z * u.flare.z;
    }
    // Grain: fine, moving, strongest in the mid-tones.
    let g = hash(i.uv * u.area.xy / px + vec2<f32>(floor(t * 24.0) * 17.0, floor(t * 24.0) * 31.0)) - 0.5;
    let m = 4.0 * y * (1.0 - y) + 0.25;
    col = col + vec3<f32>(g) * u.misc.x * m;
    return vec4<f32>(clamp(col, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
"#;

/// One preset's settings (client/timeless/title.rs filter_preset).
#[derive(Clone, Copy, Debug)]
pub struct Grade {
    pub lift: [f32; 3],
    pub shadow_tint: [f32; 3],
    pub high_tint: [f32; 3],
    pub saturation: f32,
    pub contrast: f32,
    pub halation: f32,
    pub halation_tint: [f32; 3],
    pub vignette: f32,
    pub grain: f32,
    pub soft_edges: f32,
    pub weave: f32,
    pub flicker: f32,
}

pub struct Nostalgia {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    uniform: wgpu::Buffer,
    /// The frame copied out of the scene texture (it can't be sampled while drawn into), and its group.
    copy: Option<(wgpu::Texture, wgpu::BindGroup)>,
    started: std::time::Instant,
    /// The format the pass writes.
    format: wgpu::TextureFormat,
}

impl Nostalgia {
    pub fn new(gpu: &Gpu, format: wgpu::TextureFormat) -> Self {
        let device = &gpu.device;
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("nostalgia"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("nostalgia"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("nostalgia"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("nostalgia"),
            layout: Some(&pl),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState { cull_mode: None, ..Default::default() },
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState { format, blend: None, write_mask: wgpu::ColorWrites::COLOR })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("nostalgia"),
            size: 8 * 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self { pipeline, layout, sampler, uniform, copy: None, started: std::time::Instant::now(), format }
    }

    /// Grades what is in `t` (behind `view`) in place. `flare`: the match cut's light (uv x, uv y, amount).
    /// Needs `t` to allow COPY_SRC; without it nothing is drawn.
    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &mut self,
        gpu: &Gpu,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        t: &wgpu::Texture,
        size: [u32; 2],
        g: &Grade,
        flare: [f32; 3],
    ) {
        if !t.usage().contains(wgpu::TextureUsages::COPY_SRC) || size[0] == 0 || size[1] == 0 {
            return;
        }
        let fits = self.copy.as_ref().is_some_and(|(c, _)| c.size() == t.size() && c.format() == t.format());
        if !fits {
            // Sampled through a view of the format the pass writes, so reading and writing round-trip unchanged.
            let plain: Vec<wgpu::TextureFormat> = Some(self.format).filter(|f| *f != t.format()).into_iter().collect();
            let c = gpu.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("nostalgia frame"),
                size: t.size(),
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: t.format(),
                usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &plain,
            });
            let cv = c.create_view(&wgpu::TextureViewDescriptor { format: Some(self.format), ..Default::default() });
            let group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("nostalgia"),
                layout: &self.layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: self.uniform.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&cv) },
                    wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&self.sampler) },
                ],
            });
            self.copy = Some((c, group));
        }
        let (c, group) = self.copy.as_ref().unwrap();
        encoder.copy_texture_to_texture(t.as_image_copy(), c.as_image_copy(), t.size());
        // The drawn area may be smaller than the texture (the scene is drawn at its own size into the corner).
        let (tw, th) = (t.width() as f32, t.height() as f32);
        let secs = self.started.elapsed().as_secs_f32() % 3600.0;
        let data: [f32; 32] = [
            g.lift[0],
            g.lift[1],
            g.lift[2],
            g.saturation,
            g.shadow_tint[0],
            g.shadow_tint[1],
            g.shadow_tint[2],
            g.contrast,
            g.high_tint[0],
            g.high_tint[1],
            g.high_tint[2],
            g.halation,
            g.halation_tint[0],
            g.halation_tint[1],
            g.halation_tint[2],
            g.vignette,
            g.grain,
            g.soft_edges,
            g.weave,
            g.flicker,
            secs,
            1.0 / tw,
            1.0 / th,
            size[0] as f32 / size[1] as f32,
            flare[0],
            flare[1],
            flare[2],
            0.0,
            size[0] as f32 / tw,
            size[1] as f32 / th,
            0.0,
            0.0,
        ];
        gpu.queue.put(&self.uniform, 0, bytemuck::cast_slice(&data));
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("nostalgia"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_viewport(0.0, 0.0, size[0] as f32, size[1] as f32, 0.0, 1.0);
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, group, &[]);
        pass.draw(0..3, 0..1);
    }
}
