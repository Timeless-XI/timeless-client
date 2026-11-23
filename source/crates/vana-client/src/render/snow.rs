//! Ours (Settings' Weather Modern, client/storm.rs): the falling snow of a snowfall or a blizzard, and the frost at
//! the screen's edges in a blizzard's gusts.
//!
//! The snow is up to MAX flakes in a box round the camera, all worked out on the GPU from the flake's number and the
//! clock (no buffers to fill): each falls and drifts with the wind (and a flutter of its own), wraps round the box as
//! the camera moves, is drawn as a soft dot stretched along its motion (a streak near the camera in a strong wind),
//! fades in from the camera and out at the box's edge, and is hidden behind walls by the scene's depth. Its colour is
//! the storm's air (client/storm.rs grade): white by day, dark grey-blue at night.

use super::{Gpu, Put};
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};

/// Flakes in a full blizzard.
pub const MAX: u32 = 22000;
/// The box round the camera: half its width and depth, half its height (yalms).
const HALF: f32 = 16.0;
const HALF_UP: f32 = 10.0;
/// The flakes' speed classes (0.85, 0.9 .. 1.15 of the wind): 20 x each is a whole number, so the drift can wrap
/// round every 20 boxes and every flake still lands where it would have (no jump, and no float precision lost
/// over a long session).
pub const PERIOD_XZ: f32 = HALF * 2.0 * 20.0;
pub const PERIOD_Y: f32 = HALF_UP * 2.0 * 20.0;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Uniforms {
    view_proj: [[f32; 4]; 4],
    /// xyz the camera, w seconds.
    eye: [f32; 4],
    /// xyz the wind (yalms a second, y down), w the fall speed (for the streaks).
    wind: [f32; 4],
    /// xyz how far the air has carried the snow (client/storm.rs drift, wrapped), w 0.
    drift: [f32; 4],
    /// rgb the flakes' colour, a their opacity.
    colour: [f32; 4],
    /// x half the box's width, y half its height, z 1 when the target is sRGB, w the gust.
    params: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct FrostUniforms {
    /// x the frost (0..1), y seconds, z 1 when the target is sRGB, w the aspect (width / height).
    params: [f32; 4],
    colour: [f32; 4],
}

const WGSL: &str = r#"
struct U {
    view_proj: mat4x4<f32>,
    eye: vec4<f32>,
    wind: vec4<f32>,
    drift: vec4<f32>,
    colour: vec4<f32>,
    params: vec4<f32>,
};
@group(0) @binding(0) var<uniform> u: U;

struct Out {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) alpha: f32,
};

fn hash(n: u32) -> f32 {
    var x = n * 747796405u + 2891336453u;
    x = ((x >> ((x >> 28u) + 4u)) ^ x) * 277803737u;
    x = (x >> 22u) ^ x;
    return f32(x) / 4294967295.0;
}

@vertex
fn vs(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> Out {
    var corners = array<vec2<f32>, 6>(vec2(-1.0, -1.0), vec2(1.0, -1.0), vec2(1.0, 1.0), vec2(-1.0, -1.0), vec2(1.0, 1.0), vec2(-1.0, 1.0));
    let c = corners[vi];
    let half = vec3<f32>(u.params.x, u.params.y, u.params.x);
    let size = half * 2.0;
    let t = u.eye.w;
    let h0 = hash(ii * 3u);
    let h1 = hash(ii * 3u + 1u);
    let h2 = hash(ii * 3u + 2u);
    let h3 = hash(ii * 7u + 11u);
    // Each flake its own pace (one of seven classes, 0.85 .. 1.15 of the air's): carried by the drift the client
    // adds up frame by frame, so a gust speeds the snow up rather than swinging every flake back and forth.
    let k = 0.85 + 0.05 * f32(min(u32(h3 * 7.0), 6u));
    let vel = vec3<f32>(u.wind.x, u.wind.w, u.wind.z) * k;
    let flutter = vec3<f32>(sin(t * (0.9 + h0 * 0.6) + h1 * 6.28), 0.0, cos(t * (0.8 + h2 * 0.6) + h0 * 6.28)) * 0.18;
    let p0 = vec3<f32>(h0, h1, h2) * size + u.drift.xyz * k + flutter;
    // Wrapped round the box, centred on the camera.
    let rel = (fract((p0 - u.eye.xyz) / size + 0.5) - 0.5) * size;
    let pos = u.eye.xyz + rel;
    let d = length(rel);
    // A soft dot, stretched along its motion as seen (a streak in a strong wind, near the camera).
    let view_dir = rel / max(d, 1e-3);
    var along = vel - view_dir * dot(vel, view_dir);
    let speed = length(along);
    along = select(vec3<f32>(0.0, 1.0, 0.0), along / max(speed, 1e-4), speed > 1e-4);
    let across = normalize(cross(along, view_dir));
    let r = 0.03 + 0.03 * h2;
    let streak = r + speed * 0.007 * clamp(1.2 - d / 12.0, 0.2, 1.2);
    let p = pos + along * c.y * streak + across * c.x * r;
    var o: Out;
    o.clip = u.view_proj * vec4<f32>(p, 1.0);
    o.uv = c;
    // None right at the lens (they'd be big blurry blobs), fading in by 2.5 yalms.
    let near = smoothstep(0.9, 2.5, d);
    let far = 1.0 - smoothstep(half.x * 0.6, half.x * 0.98, d);
    o.alpha = u.colour.a * near * far * (0.7 + 0.3 * h1);
    return o;
}

@fragment
fn fs(i: Out) -> @location(0) vec4<f32> {
    let d = length(i.uv);
    if (d >= 1.0) { discard; }
    // A crisp flake: solid to half its size, a short soft edge.
    let a = i.alpha * (1.0 - smoothstep(0.45, 1.0, d));
    var rgb = u.colour.rgb;
    if (u.params.z > 0.5) {
        rgb = pow(max(rgb, vec3<f32>(0.0)), vec3<f32>(2.2));
    }
    return vec4<f32>(rgb, a);
}

"#;

const FROST_WGSL: &str = r#"
struct F {
    params: vec4<f32>,
    colour: vec4<f32>,
};
@group(0) @binding(0) var<uniform> f: F;

struct FOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_frost(@builtin(vertex_index) i: u32) -> FOut {
    let p = vec2<f32>(f32((i << 1u) & 2u), f32(i & 2u)) * 2.0 - 1.0;
    var o: FOut;
    o.clip = vec4<f32>(p, 0.0, 1.0);
    o.uv = p;
    return o;
}

fn vnoise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let s = fract(p);
    let w = s * s * (3.0 - 2.0 * s);
    let n = i.x + i.y * 57.0;
    let a = fract(sin(n) * 43758.547);
    let b = fract(sin(n + 1.0) * 43758.547);
    let c = fract(sin(n + 57.0) * 43758.547);
    let d = fract(sin(n + 58.0) * 43758.547);
    return mix(mix(a, b, w.x), mix(c, d, w.x), w.y);
}

@fragment
fn fs_frost(i: FOut) -> @location(0) vec4<f32> {
    // How near an edge (the corners most), roughened by frost's crystals.
    let q = vec2<f32>(i.uv.x * f.params.w, i.uv.y);
    let edge = max(abs(i.uv.x), abs(i.uv.y)) * 0.6 + length(i.uv) * 0.4 * 0.72;
    let n = vnoise(q * 9.0) * 0.6 + vnoise(q * 23.0 + 3.1) * 0.3 + vnoise(q * 61.0 + 7.7) * 0.1;
    let reach = 1.0 - 0.12 * f.params.x;
    let v = smoothstep(reach - 0.04, reach + 0.08, edge + (n - 0.5) * 0.14);
    let a = v * f.params.x * 0.18;
    var rgb = f.colour.rgb * (0.85 + 0.3 * n);
    if (f.params.z > 0.5) {
        rgb = pow(max(rgb, vec3<f32>(0.0)), vec3<f32>(2.2));
    }
    return vec4<f32>(rgb, a);
}
"#;

const PUFF_WGSL: &str = r#"
struct P {
    view_proj: mat4x4<f32>,
    right: vec4<f32>,
    up: vec4<f32>,
    // rgb the breath's colour, a 1 when the target is sRGB
    colour: vec4<f32>,
};
@group(0) @binding(0) var<uniform> p: P;

struct In {
    @location(0) pos_size: vec4<f32>,
    // x alpha, y seed, z age (0..1)
    @location(1) look: vec4<f32>,
};

struct Out {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) look: vec4<f32>,
};

@vertex
fn vs_puff(v: In, @builtin(vertex_index) vi: u32) -> Out {
    var corners = array<vec2<f32>, 6>(vec2(-1.0, -1.0), vec2(1.0, -1.0), vec2(1.0, 1.0), vec2(-1.0, -1.0), vec2(1.0, 1.0), vec2(-1.0, 1.0));
    let c = corners[vi];
    let q = v.pos_size.xyz + (p.right.xyz * c.x + p.up.xyz * c.y) * v.pos_size.w;
    var o: Out;
    o.clip = p.view_proj * vec4<f32>(q, 1.0);
    o.uv = c;
    o.look = v.look;
    return o;
}

@fragment
fn fs_puff(i: Out) -> @location(0) vec4<f32> {
    // A soft cloud, a little uneven (two offset lobes that drift apart as it spreads)
    let s = i.look.y * 6.28;
    let off = vec2<f32>(cos(s), sin(s)) * 0.25 * i.look.z;
    let d1 = length(i.uv - off);
    let d2 = length(i.uv + off * 0.8) * 1.15;
    let a = i.look.x * max(exp(-d1 * d1 * 3.2), exp(-d2 * d2 * 3.8) * 0.8);
    if (a < 0.004) { discard; }
    var rgb = p.colour.rgb;
    if (p.colour.a > 0.5) {
        rgb = pow(max(rgb, vec3<f32>(0.0)), vec3<f32>(2.2));
    }
    return vec4<f32>(rgb, a);
}
"#;

/// A puff of breath to draw: where, how big (yalms), how visible (0..1), its seed and age (0..1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Puff {
    pub pos: Vec3,
    pub size: f32,
    pub alpha: f32,
    pub seed: f32,
    pub age: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct PuffUniforms {
    view_proj: [[f32; 4]; 4],
    right: [f32; 4],
    up: [f32; 4],
    colour: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct PuffInstance {
    pos_size: [f32; 4],
    look: [f32; 4],
}

/// Puffs drawn at most at once.
pub const PUFFS: usize = 640;

pub struct Snow {
    puffs: wgpu::RenderPipeline,
    puff_uniforms: wgpu::Buffer,
    puff_instances: wgpu::Buffer,
    puff_group: wgpu::BindGroup,
    flakes: wgpu::RenderPipeline,
    frost: wgpu::RenderPipeline,
    uniforms: wgpu::Buffer,
    frost_uniforms: wgpu::Buffer,
    group: wgpu::BindGroup,
    frost_group: wgpu::BindGroup,
    format: wgpu::TextureFormat,
}

/// What a frame's snow is.
#[derive(Clone, Copy, Debug)]
pub struct Fall {
    /// 0..1 of MAX flakes.
    pub amount: f32,
    pub wind: Vec3,
    /// How far the air has carried the snow (client/storm.rs drift), wrapped by PERIOD_XZ / PERIOD_Y.
    pub drift: Vec3,
    pub gust: f32,
    pub seconds: f32,
    /// The storm's air (client/storm.rs grade), 0..1.
    pub colour: Vec3,
    /// The breath's colour (the air, lifted), 0..1.
    pub breath: Vec3,
}

fn uniform_layout(device: &wgpu::Device, label: &str) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some(label),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    })
}

impl Snow {
    pub fn new(gpu: &Gpu, format: wgpu::TextureFormat) -> Self {
        let device = &gpu.device;
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("snow"),
            source: wgpu::ShaderSource::Wgsl(WGSL.into()),
        });
        let frost_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("frost"),
            source: wgpu::ShaderSource::Wgsl(FROST_WGSL.into()),
        });
        let layout = uniform_layout(device, "snow");
        let frost_layout = uniform_layout(device, "frost");
        let blend = wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::Zero,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
        };
        let pipeline = |module: &wgpu::ShaderModule,
                        layout: &wgpu::BindGroupLayout,
                        vs: &str,
                        fs: &str,
                        depth: bool,
                        label: &str,
                        buffers: &[Option<wgpu::VertexBufferLayout>]| {
            let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(label),
                bind_group_layouts: &[Some(layout)],
                immediate_size: 0,
            });
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(&pl),
                vertex: wgpu::VertexState {
                    module,
                    entry_point: Some(vs),
                    compilation_options: Default::default(),
                    buffers,
                },
                fragment: Some(wgpu::FragmentState {
                    module,
                    entry_point: Some(fs),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: Some(blend),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: super::DEPTH_FORMAT,
                    depth_write_enabled: Some(false),
                    depth_compare: Some(if depth {
                        wgpu::CompareFunction::Less
                    } else {
                        wgpu::CompareFunction::Always
                    }),
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: Default::default(),
                multiview_mask: None,
                cache: None,
            })
        };
        let flakes = pipeline(&module, &layout, "vs", "fs", true, "snow", &[]);
        let frost = pipeline(&frost_module, &frost_layout, "vs_frost", "fs_frost", false, "frost", &[]);
        let puff_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("breath"),
            source: wgpu::ShaderSource::Wgsl(PUFF_WGSL.into()),
        });
        let puff_layout = uniform_layout(device, "breath");
        let puffs = pipeline(
            &puff_module,
            &puff_layout,
            "vs_puff",
            "fs_puff",
            true,
            "breath",
            &[Some(wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<PuffInstance>() as u64,
                step_mode: wgpu::VertexStepMode::Instance,
                attributes: &wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4],
            })],
        );
        let buffer = |size: usize, label: &str| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: size as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        };
        let uniforms = buffer(std::mem::size_of::<Uniforms>(), "snow");
        let frost_uniforms = buffer(std::mem::size_of::<FrostUniforms>(), "frost");
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("snow"),
            layout: &layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: uniforms.as_entire_binding() }],
        });
        let frost_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("frost"),
            layout: &frost_layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: frost_uniforms.as_entire_binding() }],
        });
        let puff_uniforms = buffer(std::mem::size_of::<PuffUniforms>(), "breath");
        let puff_instances = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("breath puffs"),
            size: (std::mem::size_of::<PuffInstance>() * PUFFS) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let puff_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("breath"),
            layout: &puff_layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: puff_uniforms.as_entire_binding() }],
        });
        Self {
            puffs,
            puff_uniforms,
            puff_instances,
            puff_group,
            flakes,
            frost,
            uniforms,
            frost_uniforms,
            group,
            frost_group,
            format,
        }
    }

    pub fn format(&self) -> wgpu::TextureFormat {
        self.format
    }

    /// The breath puffs and the flakes into the scene (hidden behind walls by its depth), then the frost over it
    /// (`frost` 0..1). `view` gives the camera's right and up for the puffs.
    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &self,
        gpu: &Gpu,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        depth: &wgpu::TextureView,
        (view, view_proj): (Mat4, Mat4),
        eye: Vec3,
        fall: &Fall,
        puffs: &[Puff],
        frost: f32,
        aspect: f32,
    ) {
        let n = (fall.amount.clamp(0.0, 1.0) * MAX as f32) as u32;
        let np = puffs.len().min(PUFFS);
        if n == 0 && frost <= 0.0 && np == 0 {
            return;
        }
        if np > 0 {
            let row = |k: usize| Vec3::new(view.x_axis[k], view.y_axis[k], view.z_axis[k]);
            let (right, up) = (row(0), row(1));
            let pu = PuffUniforms {
                view_proj: view_proj.to_cols_array_2d(),
                right: [right.x, right.y, right.z, 0.0],
                up: [up.x, up.y, up.z, 0.0],
                colour: [fall.breath.x, fall.breath.y, fall.breath.z, self.format.is_srgb() as u32 as f32],
            };
            gpu.queue.put(&self.puff_uniforms, 0, bytemuck::bytes_of(&pu));
            let inst: Vec<PuffInstance> = puffs[..np]
                .iter()
                .map(|p| PuffInstance {
                    pos_size: [p.pos.x, p.pos.y, p.pos.z, p.size],
                    look: [p.alpha, p.seed, p.age, 0.0],
                })
                .collect();
            gpu.queue.put(&self.puff_instances, 0, bytemuck::cast_slice(&inst));
        }
        let srgb = self.format.is_srgb() as u32 as f32;
        let u = Uniforms {
            view_proj: view_proj.to_cols_array_2d(),
            eye: [eye.x, eye.y, eye.z, fall.seconds],
            wind: [fall.wind.x, fall.wind.y, fall.wind.z, crate::client::storm_fall(fall.gust)],
            drift: [fall.drift.x, fall.drift.y, fall.drift.z, 0.0],
            colour: [fall.colour.x, fall.colour.y, fall.colour.z, 0.95],
            params: [HALF, HALF_UP, srgb, fall.gust],
        };
        gpu.queue.put(&self.uniforms, 0, bytemuck::bytes_of(&u));
        let fu = FrostUniforms {
            params: [frost.clamp(0.0, 1.0), fall.seconds, srgb, aspect.max(0.1)],
            colour: [0.86, 0.93, 1.0, 1.0],
        };
        gpu.queue.put(&self.frost_uniforms, 0, bytemuck::bytes_of(&fu));
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("snow"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth,
                depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        if np > 0 {
            pass.set_pipeline(&self.puffs);
            pass.set_bind_group(0, &self.puff_group, &[]);
            pass.set_vertex_buffer(0, self.puff_instances.slice(..));
            pass.draw(0..6, 0..np as u32);
        }
        if n > 0 {
            pass.set_pipeline(&self.flakes);
            pass.set_bind_group(0, &self.group, &[]);
            pass.draw(0..6, 0..n);
        }
        if frost > 0.0 {
            pass.set_pipeline(&self.frost);
            pass.set_bind_group(0, &self.frost_group, &[]);
            pass.draw(0..3, 0..1);
        }
    }
}

#[cfg(test)]
mod tests {
    /// The snow and frost shader parses and validates (no GPU needed); the uniforms are as the shader reads them.
    #[test]
    fn snow_shader_is_valid() {
        assert_eq!(std::mem::size_of::<super::PuffUniforms>(), 64 + 3 * 16);
        assert_eq!(std::mem::size_of::<super::PuffInstance>(), 32);
        for src in [super::WGSL, super::FROST_WGSL, super::PUFF_WGSL] {
            let m = wgpu::naga::front::wgsl::parse_str(src).expect("parse");
            wgpu::naga::valid::Validator::new(Default::default(), wgpu::naga::valid::Capabilities::all())
                .validate(&m)
                .expect("valid");
        }
        assert_eq!(std::mem::size_of::<super::Uniforms>(), 64 + 5 * 16);
        // The speed classes times 20 are whole numbers: the drift's wrap moves no flake.
        for i in 0..7 {
            let k = 0.85 + 0.05 * i as f32;
            assert!((k * 20.0 - (k * 20.0).round()).abs() < 1e-4);
        }
        assert_eq!(std::mem::size_of::<super::FrostUniforms>(), 32);
    }
}
