use std::collections::HashMap;

use crate::font::FontManager;

pub const ATLAS_SIZE: u32 = 1024;
pub const PAD: u32 = 1;

// INFO: Dimensions and UV bounds for a rasterized character glyph in the texture atlas.
#[derive(Clone, Copy, Debug)]
pub struct GlyphInfo {
    pub uv_min: [f32; 2],
    pub uv_max: [f32; 2],
    pub width: f32,
    pub height: f32,
    pub bearing_left: f32,
    pub bearing_top: f32,
}

// INFO: Manages texture atlas allocation, circle baking for rounded corners, and glyph caching.
pub struct GlyphAtlas {
    pub texture: wgpu::Texture,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    pub solid_uv: [f32; 2],
    pub circle_uv: [[f32; 2]; 2],
    cache: HashMap<(char, bool, bool, u32), Option<GlyphInfo>>,
    pack_x: u32,
    pack_y: u32,
    shelf_h: u32,
}

impl GlyphAtlas {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
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

        // Initialize 1x1 white pixel at top-left for solid quads
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

        let mut atlas = Self {
            texture,
            bind_group_layout,
            bind_group,
            solid_uv,
            circle_uv: [[0.0, 0.0], [0.0, 0.0]],
            cache: HashMap::new(),
            pack_x: 2,
            pack_y: 0,
            shelf_h: 0,
        };
        atlas.bake_circle(queue);
        atlas
    }

    // INFO: Renders and packs an antialiased circle for fast rounded quad geometry.
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

    // INFO: Simple shelf-packing algorithm allocating dynamic sub-rectangles in the atlas.
    pub fn pack(&mut self, w: u32, h: u32) -> Option<(u32, u32)> {
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

    // INFO: Rasterizes or looks up cached glyph information for the specified character and styling.
    pub fn glyph(
        &mut self,
        queue: &wgpu::Queue,
        fonts: &FontManager,
        ch: char,
        bold: bool,
        italic: bool,
    ) -> Option<GlyphInfo> {
        let size_key = (fonts.size() * 100.0).round() as u32;
        if let Some(cached) = self.cache.get(&(ch, bold, italic, size_key)) {
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

        self.cache.insert((ch, bold, italic, size_key), info);
        info
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
        self.pack_x = 2;
        self.pack_y = 0;
        self.shelf_h = 0;
    }
}
