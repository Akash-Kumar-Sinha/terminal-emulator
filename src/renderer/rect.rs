use bytemuck::{Pod, Zeroable};

const SHADER: &str = r#"
struct VertexInput {
    @location(0) pos:   vec2<f32>,
    @location(1) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) pos:   vec4<f32>,
    @location(0)       color: vec4<f32>,
}

@vertex
fn vs(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.pos   = vec4<f32>(in.pos, 0.0, 1.0);
    out.color = in.color;
    return out;
}

@fragment
fn fs(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
"#;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct Vertex {
    pos:   [f32; 2],
    color: [f32; 4],
}

pub struct RectPipeline {
    pipeline:      wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
}

impl RectPipeline {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 8,
                            shader_location: 1,
                        },
                    ],
                }],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (18 * std::mem::size_of::<Vertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self { pipeline, vertex_buffer }
    }

    pub fn draw<'pass>(
        &'pass self,
        queue:        &wgpu::Queue,
        pass:         &mut wgpu::RenderPass<'pass>,
        x:            f32,
        y:            f32,
        width:        f32,
        height:       f32,
        border_width: f32,
        bg_color:     [f32; 4],
        border_color: [f32; 4],
        screen_w:     f32,
        screen_h:     f32,
    ) {
        let to_clip_x = |px: f32| (px / screen_w) * 2.0 - 1.0;
        let to_clip_y = |py: f32| 1.0 - (py / screen_h) * 2.0;

        let left        = to_clip_x(x);
        let right       = to_clip_x(x + width);
        let top         = to_clip_y(y);
        let bottom      = to_clip_y(y + height);
        let top_border  = to_clip_y(y + border_width);
        let bot_border  = to_clip_y(y + height - border_width);

        let quad = |l: f32, t: f32, r: f32, b: f32, c: [f32; 4]| -> [Vertex; 6] {
            [
                Vertex { pos: [l, t], color: c },
                Vertex { pos: [r, t], color: c },
                Vertex { pos: [l, b], color: c },
                Vertex { pos: [r, t], color: c },
                Vertex { pos: [r, b], color: c },
                Vertex { pos: [l, b], color: c },
            ]
        };

        let bg            = quad(left, top,        right, bottom,     bg_color);
        let border_top    = quad(left, top,         right, top_border, border_color);
        let border_bottom = quad(left, bot_border,  right, bottom,     border_color);

        let mut vertices = [Vertex { pos: [0.0; 2], color: [0.0; 4] }; 18];
        vertices[0..6].copy_from_slice(&bg);
        vertices[6..12].copy_from_slice(&border_top);
        vertices[12..18].copy_from_slice(&border_bottom);

        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));

        pass.set_pipeline(&self.pipeline);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.draw(0..18, 0..1);
    }
}
