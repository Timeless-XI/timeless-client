//! Ours (Settings' Night Lighting Modern, client/night.rs): a soft halo of light round each lamp's glow and lantern
//! after dark, drawn over the scene and its effects, added (the scene's depth hides one behind a wall). A halo with a
//! table slot takes that light's live colour and intensity (a lamp that dims at dawn, one the lamplighter has not lit
//! yet) and a warm one flickers as the light does (zone.wgsl `lw`); one without a slot its glow's colour.

use super::{Gpu, Put};
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};

/// A halo to draw: where, the table slot whose light it shows (none: `colour`, 1 = 0x80), its size, how lit (0..1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Halo {
    pub pos: Vec3,
    pub slot: Option<u32>,
    pub colour: [f32; 3],
    pub size: f32,
    pub level: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Uniforms {
    view_proj: [[f32; 4]; 4],
    right: [f32; 4],
    up: [f32; 4],
    /// Toward the camera (a halo is drawn this much nearer, w yalms, so the wall it is on doesn't cut it).
    back: [f32; 4],
    /// x strength (the night), y seconds (flicker), z 1 when the target is sRGB (write linear), w 0.
    params: [f32; 4],
}

/// One halo: position and size, colour and level, the table slot (u32::MAX none).
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Instance {
    pos_size: [f32; 4],
    colour_level: [f32; 4],
    slot: u32,
}

/// Halos drawn at most at once.
const MAX: usize = 512;

pub struct Halos {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    uniforms: wgpu::Buffer,
    instances: wgpu::Buffer,
    group: Option<(wgpu::Buffer, wgpu::BindGroup)>,
    format: wgpu::TextureFormat,
}

const WGSL: &str = r#"
struct U {
    view_proj: mat4x4<f32>,
    right: vec4<f32>,
    up: vec4<f32>,
    back: vec4<f32>,
    params: vec4<f32>,
};
@group(0) @binding(0) var<uniform> u: U;
@group(0) @binding(1) var<storage, read> plights: array<vec4<f32>>;

struct In {
    @location(0) pos_size: vec4<f32>,
    @location(1) colour_level: vec4<f32>,
    @location(2) slot: u32,
};

struct Out {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) colour: vec3<f32>,
};

@vertex
fn vs(v: In, @builtin(vertex_index) vi: u32) -> Out {
    var corners = array<vec2<f32>, 6>(vec2(-1.0, -1.0), vec2(1.0, -1.0), vec2(1.0, 1.0), vec2(-1.0, -1.0), vec2(1.0, 1.0), vec2(-1.0, 1.0));
    let c = corners[vi];
    var colour = v.colour_level.rgb;
    var k = v.colour_level.w * u.params.x;
    if (v.slot != 0xffffffffu) {
        let l = plights[2u * v.slot + 1u];
        colour = l.rgb;
        // The light's live intensity: none when it's out, full from about a lamp's.
        k = k * (1.0 - exp(-max(l.w, 0.0) / 12.0));
        if (l.r > l.b * 1.3 && l.r > 0.0) {
            let f = f32(v.slot);
            k = k * (1.0 + 0.07 * sin(u.params.y * 7.7 + f * 1.93) + 0.04 * sin(u.params.y * 17.3 + f * 4.1));
        }
    }
    let p = v.pos_size.xyz + (u.right.xyz * c.x + u.up.xyz * c.y) * v.pos_size.w + u.back.xyz * u.back.w;
    var o: Out;
    o.clip = u.view_proj * vec4<f32>(p, 1.0);
    o.uv = c;
    o.colour = colour * k;
    return o;
}

@fragment
fn fs(i: Out) -> @location(0) vec4<f32> {
    let d = length(i.uv);
    if (d >= 1.0) { discard; }
    let a = pow(1.0 - d, 2.2) * 0.55 + pow(1.0 - d, 8.0) * 0.45;
    var rgb = min(i.colour, vec3<f32>(1.5)) * a;
    if (u.params.z > 0.5) {
        rgb = pow(max(rgb, vec3<f32>(0.0)), vec3<f32>(2.2));
    }
    return vec4<f32>(rgb, 0.0);
}
"#;

impl Halos {
    pub fn new(gpu: &Gpu, format: wgpu::TextureFormat) -> Self {
        let device = &gpu.device;
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("halos"),
            source: wgpu::ShaderSource::Wgsl(WGSL.into()),
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("halos"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("halos"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let add = wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::One,
            dst_factor: wgpu::BlendFactor::One,
            operation: wgpu::BlendOperation::Add,
        };
        let keep = wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::Zero,
            dst_factor: wgpu::BlendFactor::One,
            operation: wgpu::BlendOperation::Add,
        };
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("halos"),
            layout: Some(&pl),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs"),
                compilation_options: Default::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Instance>() as u64,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4, 2 => Uint32],
                })],
            },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState { color: add, alpha: keep }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: super::DEPTH_FORMAT,
                depth_write_enabled: Some(false),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            multiview_mask: None,
            cache: None,
        });
        let uniforms = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("halos"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let instances = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("halo instances"),
            size: (std::mem::size_of::<Instance>() * MAX) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self { pipeline, layout, uniforms, instances, group: None, format }
    }

    pub fn format(&self) -> wgpu::TextureFormat {
        self.format
    }

    /// Draws `halos` (client/night.rs) into the scene: `lights` the zone's point-light table, `strength` the night's.
    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &mut self,
        gpu: &Gpu,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        depth: &wgpu::TextureView,
        lights: &wgpu::Buffer,
        view: Mat4,
        view_proj: Mat4,
        strength: f32,
        seconds: f32,
        halos: &[Halo],
    ) {
        if halos.is_empty() || strength <= 0.0 {
            return;
        }
        let n = halos.len().min(MAX);
        let inst: Vec<Instance> = halos[..n]
            .iter()
            .map(|h| Instance {
                pos_size: [h.pos.x, h.pos.y, h.pos.z, h.size],
                colour_level: [h.colour[0], h.colour[1], h.colour[2], h.level],
                slot: h.slot.unwrap_or(u32::MAX),
            })
            .collect();
        // The camera's right and up in the world (the view matrix's rows).
        let row = |k: usize| Vec3::new(view.x_axis[k], view.y_axis[k], view.z_axis[k]);
        let (right, up, back) = (row(0), row(1), row(2));
        let u = Uniforms {
            view_proj: view_proj.to_cols_array_2d(),
            right: [right.x, right.y, right.z, 0.0],
            up: [up.x, up.y, up.z, 0.0],
            back: [back.x, back.y, back.z, 0.6],
            params: [strength, seconds, self.format.is_srgb() as u32 as f32, 0.0],
        };
        gpu.queue.put(&self.uniforms, 0, bytemuck::bytes_of(&u));
        gpu.queue.put(&self.instances, 0, bytemuck::cast_slice(&inst));
        if self.group.as_ref().is_none_or(|(b, _)| b != lights) {
            let g = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("halos"),
                layout: &self.layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: self.uniforms.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 1, resource: lights.as_entire_binding() },
                ],
            });
            self.group = Some((lights.clone(), g));
        }
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("halos"),
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
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, self.group.as_ref().map(|g| &g.1), &[]);
        pass.set_vertex_buffer(0, self.instances.slice(..));
        pass.draw(0..6, 0..n as u32);
    }
}

#[cfg(test)]
mod tests {
    /// The halo shader parses and validates (no GPU needed); its uniform is as the shader reads it.
    #[test]
    fn halo_shader_is_valid() {
        let m = wgpu::naga::front::wgsl::parse_str(super::WGSL).expect("parse");
        wgpu::naga::valid::Validator::new(Default::default(), wgpu::naga::valid::Capabilities::all())
            .validate(&m)
            .expect("valid");
        assert_eq!(std::mem::size_of::<super::Uniforms>(), 64 + 4 * 16);
        assert_eq!(std::mem::size_of::<super::Instance>(), 36);
    }
}
