use std::collections::HashMap;

use bytemuck::{Pod, Zeroable};

use crate::app::block::Block;
use crate::app::ui::header::{Header, HeaderButton};
use crate::fonts::FontManager;
use crate::theme::Theme;

pub struct ChromeRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub color: [f32; 4],
}

pub struct ChromeText {
    pub x: f32,
    pub y: f32,
    pub text: String,
    pub color: [f32; 4],
    pub bold: bool,
}

const ATLAS_SIZE: u32 = 1024;
const PAD: u32 = 1;

const SHADER: &str = r#"
struct VtxIn {
    @location(0) pos:   vec2<f32>,
    @location(1) uv:    vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VtxOut {
    @builtin(position) pos:   vec4<f32>,
    @location(0)       uv:    vec2<f32>,
    @location(1)       color: vec4<f32>,
}

@group(0) @binding(0) var atlas_tex:  texture_2d<f32>;
@group(0) @binding(1) var atlas_samp: sampler;

@vertex
fn vs(in: VtxIn) -> VtxOut {
    var out: VtxOut;
    out.pos   = vec4<f32>(in.pos, 0.0, 1.0);
    out.uv    = in.uv;
    out.color = in.color;
    return out;
}

@fragment
fn fs(in: VtxOut) -> @location(0) vec4<f32> {
    let coverage = textureSample(atlas_tex, atlas_samp, in.uv).r;
    return vec4<f32>(in.color.rgb, in.color.a * coverage);
}
"#;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct GlyphVertex {
    pos: [f32; 2],
    uv: [f32; 2],
    color: [f32; 4],
}

#[derive(Clone, Copy)]
struct GlyphInfo {
    uv_min: [f32; 2],
    uv_max: [f32; 2],
    width: f32,
    height: f32,
    bearing_left: f32,
    bearing_top: f32,
}

pub struct GlyphRenderer {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    texture: wgpu::Texture,
    vertex_buffer: wgpu::Buffer,
    vertex_capacity: usize,
    vertex_count: u32,

    cache: HashMap<(char, bool, bool), Option<GlyphInfo>>,
    solid_uv: [f32; 2],
    circle_uv: [[f32; 2]; 2],
    pack_x: u32,
    pack_y: u32,
    shelf_h: u32,
}

impl GlyphRenderer {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("glyph atlas"),
            size: wgpu::Extent3d {
                width: ATLAS_SIZE,
                height: ATLAS_SIZE,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &[0xffu8],
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(1),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );
        let solid_uv = [0.5 / ATLAS_SIZE as f32, 0.5 / ATLAS_SIZE as f32];

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("glyph sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("glyph bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("glyph bind group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("glyph shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("glyph pipeline layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("glyph pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<GlyphVertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 8,
                            shader_location: 1,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 16,
                            shader_location: 2,
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

        let vertex_capacity = 6 * 4096;
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("glyph vertices"),
            size: (vertex_capacity * std::mem::size_of::<GlyphVertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut renderer = Self {
            pipeline,
            bind_group,
            texture,
            vertex_buffer,
            vertex_capacity,
            vertex_count: 0,
            cache: HashMap::new(),
            solid_uv,
            circle_uv: [[0.0, 0.0], [0.0, 0.0]],
            pack_x: 2,
            pack_y: 0,
            shelf_h: 0,
        };
        renderer.bake_circle(queue);
        renderer
    }

    fn bake_circle(&mut self, queue: &wgpu::Queue) {
        const D: u32 = 64;
        let r = D as f32 / 2.0;
        let mut buf = vec![0u8; (D * D) as usize];
        for py in 0..D {
            for px in 0..D {
                let dx = px as f32 + 0.5 - r;
                let dy = py as f32 + 0.5 - r;
                let dist = (dx * dx + dy * dy).sqrt();
                let cov = (r - dist + 0.5).clamp(0.0, 1.0);
                buf[(py * D + px) as usize] = (cov * 255.0) as u8;
            }
        }
        if let Some((x, y)) = self.pack(D, D) {
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &self.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d { x, y, z: 0 },
                    aspect: wgpu::TextureAspect::All,
                },
                &buf,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(D),
                    rows_per_image: Some(D),
                },
                wgpu::Extent3d {
                    width: D,
                    height: D,
                    depth_or_array_layers: 1,
                },
            );
            let atlas = ATLAS_SIZE as f32;
            self.circle_uv = [
                [(x as f32 + 0.5) / atlas, (y as f32 + 0.5) / atlas],
                [
                    (x as f32 + D as f32 - 0.5) / atlas,
                    (y as f32 + D as f32 - 0.5) / atlas,
                ],
            ];
        }
    }

    fn pack(&mut self, w: u32, h: u32) -> Option<(u32, u32)> {
        if self.pack_x + w > ATLAS_SIZE {
            self.pack_x = 0;
            self.pack_y += self.shelf_h + PAD;
            self.shelf_h = 0;
        }
        if self.pack_y + h > ATLAS_SIZE {
            return None;
        }
        let x = self.pack_x;
        let y = self.pack_y;
        self.pack_x += w + PAD;
        self.shelf_h = self.shelf_h.max(h);
        Some((x, y))
    }

    fn glyph(
        &mut self,
        queue: &wgpu::Queue,
        fonts: &FontManager,
        ch: char,
        bold: bool,
        italic: bool,
    ) -> Option<GlyphInfo> {
        if let Some(cached) = self.cache.get(&(ch, bold, italic)) {
            return *cached;
        }

        let info = match fonts.rasterize(ch, bold, italic) {
            Some(raster) => match self.pack(raster.width, raster.height) {
                Some((x, y)) => {
                    queue.write_texture(
                        wgpu::TexelCopyTextureInfo {
                            texture: &self.texture,
                            mip_level: 0,
                            origin: wgpu::Origin3d { x, y, z: 0 },
                            aspect: wgpu::TextureAspect::All,
                        },
                        &raster.coverage,
                        wgpu::TexelCopyBufferLayout {
                            offset: 0,
                            bytes_per_row: Some(raster.width),
                            rows_per_image: Some(raster.height),
                        },
                        wgpu::Extent3d {
                            width: raster.width,
                            height: raster.height,
                            depth_or_array_layers: 1,
                        },
                    );
                    let atlas = ATLAS_SIZE as f32;
                    Some(GlyphInfo {
                        uv_min: [x as f32 / atlas, y as f32 / atlas],
                        uv_max: [
                            (x + raster.width) as f32 / atlas,
                            (y + raster.height) as f32 / atlas,
                        ],
                        width: raster.width as f32,
                        height: raster.height as f32,
                        bearing_left: raster.bearing_left,
                        bearing_top: raster.bearing_top,
                    })
                }
                None => None,
            },
            None => None,
        };

        self.cache.insert((ch, bold, italic), info);
        info
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
        self.pack_x = 2;
        self.pack_y = 0;
        self.shelf_h = 0;
    }

    #[allow(clippy::too_many_arguments)]
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        header: &Header,
        blocks: &[&Block],
        fonts: &FontManager,
        theme: &Theme,
        screen_w: f32,
        screen_h: f32,
    ) {
        let cell_w = fonts.cell_w;
        let cell_h = fonts.cell_h;
        let ascent = fonts.ascent_px;

        let to_clip_x = |px: f32| (px / screen_w) * 2.0 - 1.0;
        let to_clip_y = |py: f32| 1.0 - (py / screen_h) * 2.0;

        let mut verts: Vec<GlyphVertex> = Vec::new();

        let mut push_quad = |x0: f32,
                             y0: f32,
                             x1: f32,
                             y1: f32,
                             uv_min: [f32; 2],
                             uv_max: [f32; 2],
                             c: [f32; 4]| {
            let l = to_clip_x(x0);
            let r = to_clip_x(x1);
            let t = to_clip_y(y0);
            let b = to_clip_y(y1);
            let v = |pos: [f32; 2], uv: [f32; 2]| GlyphVertex { pos, uv, color: c };
            verts.push(v([l, t], [uv_min[0], uv_min[1]]));
            verts.push(v([r, t], [uv_max[0], uv_min[1]]));
            verts.push(v([l, b], [uv_min[0], uv_max[1]]));
            verts.push(v([r, t], [uv_max[0], uv_min[1]]));
            verts.push(v([r, b], [uv_max[0], uv_max[1]]));
            verts.push(v([l, b], [uv_min[0], uv_max[1]]));
        };
        let solid = self.solid_uv;
        let circle = self.circle_uv;

        for block in blocks {
            let grid = block.shell.grid();
            let lines = grid.visible_lines();
            let cursor = if block.shell.scroll_offset() == 0 {
                Some(block.shell.cursor())
            } else {
                None
            };

            push_quad(
                block.x,
                block.y,
                block.x + block.width as f32,
                block.y + block.height as f32,
                solid,
                solid,
                block.bg_color,
            );

            for (row_idx, row) in lines.iter().enumerate() {
                for (col_idx, cell) in row.iter().enumerate() {
                    if let Some(bg) = theme.resolve_bg(cell.bg) {
                        let x0 = block.x + col_idx as f32 * cell_w;
                        let y0 = block.y + row_idx as f32 * cell_h;
                        push_quad(x0, y0, x0 + cell_w, y0 + cell_h, solid, solid, bg);
                    }
                }
            }

            if let Some((crow, ccol)) = cursor {
                let x0 = block.x + ccol as f32 * cell_w;
                let y0 = block.y + crow as f32 * cell_h;
                push_quad(
                    x0,
                    y0,
                    x0 + cell_w,
                    y0 + cell_h,
                    solid,
                    solid,
                    theme.cursor,
                );
            }

            for (row_idx, row) in lines.iter().enumerate() {
                for (col_idx, cell) in row.iter().enumerate() {
                    if cell.ch == ' ' || cell.ch == '\0' {
                        continue;
                    }
                    let Some(info) = self.glyph(queue, fonts, cell.ch, cell.bold, cell.italic)
                    else {
                        continue;
                    };
                    let fg = if cursor == Some((row_idx, col_idx)) {
                        theme.cursor_text
                    } else {
                        theme.resolve_fg(cell.fg, cell.bold)
                    };
                    let x0 = block.x + col_idx as f32 * cell_w + info.bearing_left;
                    let y0 = block.y + row_idx as f32 * cell_h + ascent + info.bearing_top;
                    push_quad(
                        x0,
                        y0,
                        x0 + info.width,
                        y0 + info.height,
                        info.uv_min,
                        info.uv_max,
                        fg,
                    );
                }
            }

            if block.border_top {
                push_quad(
                    block.x,
                    block.y,
                    block.x + block.width as f32,
                    block.y + block.border_thickness,
                    solid,
                    solid,
                    block.border_color,
                );
            }
            if block.border_bottom {
                let y1 = block.y + block.height as f32;
                push_quad(
                    block.x,
                    y1 - block.border_thickness,
                    block.x + block.width as f32,
                    y1,
                    solid,
                    solid,
                    block.border_color,
                );
            }
        }

        push_quad(0.0, 0.0, screen_w, header.height, solid, solid, header.bg_color);

        let title_y = (header.height - cell_h) / 2.0;
        let mut cx = 12.0;
        for ch in header.title.chars() {
            if ch != ' ' {
                if let Some(info) = self.glyph(queue, fonts, ch, false, false) {
                    let gx = cx + info.bearing_left;
                    let gy = title_y + ascent + info.bearing_top;
                    push_quad(
                        gx,
                        gy,
                        gx + info.width,
                        gy + info.height,
                        info.uv_min,
                        info.uv_max,
                        header.title_color,
                    );
                }
            }
            cx += cell_w;
        }

        for btn in [
            HeaderButton::Minimize,
            HeaderButton::Maximize,
            HeaderButton::Close,
        ] {
            let (bx, by, bw, bh) = header.button_rect(btn, screen_w);
            if header.hovered == Some(btn) {
                let hbg = if btn == HeaderButton::Close {
                    header.close_hover
                } else {
                    header.button_hover
                };
                let d = (bw.min(bh) - 10.0).max(4.0);
                let sx = bx + (bw - d) / 2.0;
                let sy = by + (bh - d) / 2.0;
                push_quad(sx, sy, sx + d, sy + d, circle[0], circle[1], hbg);
            }
            if let Some(info) = self.glyph(queue, fonts, Header::icon(btn), false, false) {
                let gx = bx + (bw - info.width) / 2.0;
                let gy = by + (bh - info.height) / 2.0;
                push_quad(
                    gx,
                    gy,
                    gx + info.width,
                    gy + info.height,
                    info.uv_min,
                    info.uv_max,
                    header.icon_color,
                );
            }
        }

        self.vertex_count = verts.len() as u32;
        if verts.is_empty() {
            return;
        }

        if verts.len() > self.vertex_capacity {
            let new_cap = verts.len().next_power_of_two();
            self.vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("glyph vertices"),
                size: (new_cap * std::mem::size_of::<GlyphVertex>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.vertex_capacity = new_cap;
        }

        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&verts));
    }

    pub fn draw<'pass>(&'pass self, pass: &mut wgpu::RenderPass<'pass>) {
        if self.vertex_count == 0 {
            return;
        }
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.draw(0..self.vertex_count, 0..1);
    }
}
