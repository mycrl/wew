use std::{borrow::Cow, sync::Arc};

use anyhow::Result;
use bytemuck::{Pod, Zeroable};
use wew::Rect;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    wgt::SurfaceConfiguration,
    *,
};

use winit::window::Window;

use crate::{HEIGHT, WIDTH};

static VERTEX_SHADER: &str = r#"
    struct VertexOutput {
        @builtin(position) position: vec4<f32>,
        @location(0) coords: vec2<f32>,
    };

    @vertex fn main(@location(0) position: vec2<f32>, @location(1) coords: vec2<f32>) -> VertexOutput {
        var output: VertexOutput;
        output.position = vec4<f32>(position, 0.0, 1.0);
        output.coords = vec2<f32>(coords.x, 1.0 - coords.y);
        return output;
    }
"#;

static FRAGMENT_SHADER: &str = r#"
    @group(0) @binding(0) var texture_: texture_2d<f32>;
    @group(0) @binding(1) var sampler_: sampler;
    
    @fragment fn main(@location(0) coords: vec2<f32>) -> @location(0) vec4<f32> {
        return textureSample(texture_, sampler_, coords);
    }
"#;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

impl Vertex {
    pub const INDICES: &'static [u16] = &[0, 1, 2, 2, 1, 3];

    pub const VERTICES: &'static [Vertex] = &[
        Vertex::new([-1.0, -1.0], [0.0, 0.0]),
        Vertex::new([1.0, -1.0], [1.0, 0.0]),
        Vertex::new([-1.0, 1.0], [0.0, 1.0]),
        Vertex::new([1.0, 1.0], [1.0, 1.0]),
    ];

    pub const fn new(position: [f32; 2], tex_coords: [f32; 2]) -> Self {
        Self {
            position,
            tex_coords,
        }
    }

    pub fn desc<'a>() -> VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x2,
                },
                VertexAttribute {
                    shader_location: 1,
                    format: VertexFormat::Float32x2,
                    offset: std::mem::size_of::<[f32; 2]>() as BufferAddress,
                },
            ],
        }
    }
}

#[allow(unused)]
pub struct Render {
    instance: Instance,
    surface: Surface<'static>,
    surface_config: SurfaceConfiguration<Vec<TextureFormat>>,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    texture: Texture,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    sampler: Sampler,
    bind_group_layout: BindGroupLayout,
    texture_view: TextureView,
    bind_group: BindGroup,
    pipeline: RenderPipeline,
}

impl Render {
    pub async fn new(window: Arc<Window>) -> Result<Self> {
        let instance = Instance::new(&InstanceDescriptor::default());
        let surface = instance.create_surface(window)?;
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::LowPower,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                ..Default::default()
            })
            .await?;

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                memory_hints: MemoryHints::MemoryUsage,
                required_features: adapter.features(),
                required_limits: adapter.limits(),
                ..Default::default()
            })
            .await?;

        let mut surface_config = surface.get_default_config(&adapter, WIDTH, HEIGHT).unwrap();
        surface.configure(&device, {
            surface_config.present_mode = PresentMode::Immediate;
            surface_config.format = TextureFormat::Bgra8Unorm;
            surface_config.alpha_mode = CompositeAlphaMode::Opaque;
            surface_config.usage = TextureUsages::RENDER_ATTACHMENT;

            &surface_config
        });

        let texture = device.create_texture(&TextureDescriptor {
            label: None,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            format: TextureFormat::Bgra8Unorm,
            view_formats: &[],
            size: Extent3d {
                depth_or_array_layers: 1,
                width: WIDTH,
                height: HEIGHT,
            },
        });

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(Vertex::VERTICES),
            usage: BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(Vertex::INDICES),
            usage: BufferUsages::INDEX,
        });

        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mipmap_filter: FilterMode::Nearest,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let texture_view = texture.create_view(&TextureViewDescriptor {
            dimension: Some(TextureViewDimension::D2),
            format: Some(TextureFormat::Bgra8Unorm),
            aspect: TextureAspect::All,
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            })),
            vertex: VertexState {
                entry_point: Some("main"),
                module: &device.create_shader_module(ShaderModuleDescriptor {
                    label: None,
                    source: ShaderSource::Wgsl(Cow::Borrowed(VERTEX_SHADER)),
                }),
                compilation_options: PipelineCompilationOptions::default(),
                buffers: &[Vertex::desc()],
            },
            fragment: Some(FragmentState {
                entry_point: Some("main"),
                module: &device.create_shader_module(ShaderModuleDescriptor {
                    label: None,
                    source: ShaderSource::Wgsl(Cow::Borrowed(FRAGMENT_SHADER)),
                }),
                compilation_options: PipelineCompilationOptions::default(),
                targets: &[Some(ColorTargetState {
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                    format: TextureFormat::Bgra8Unorm,
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleStrip,
                strip_index_format: Some(IndexFormat::Uint16),
                ..Default::default()
            },
            multisample: MultisampleState::default(),
            depth_stencil: None,
            multiview: None,
            cache: None,
        });

        Ok(Self {
            surface_config,
            pipeline,
            texture,
            vertex_buffer,
            index_buffer,
            sampler,
            bind_group_layout,
            texture_view,
            bind_group,
            instance,
            surface,
            adapter,
            device,
            queue,
        })
    }

    pub fn render(&mut self, buffer: &[u8], rect: &Rect) {
        if rect.width != self.texture.width() || rect.height != self.texture.height() {
            self.surface.configure(&self.device, {
                self.surface_config.width = rect.width;
                self.surface_config.height = rect.height;

                &self.surface_config
            });

            self.texture = self.device.create_texture(&TextureDescriptor {
                label: None,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                format: TextureFormat::Bgra8Unorm,
                view_formats: &[],
                size: Extent3d {
                    depth_or_array_layers: 1,
                    width: rect.width,
                    height: rect.height,
                },
            });

            self.texture_view = self.texture.create_view(&TextureViewDescriptor {
                dimension: Some(TextureViewDimension::D2),
                format: Some(TextureFormat::Bgra8Unorm),
                aspect: TextureAspect::All,
                ..Default::default()
            });

            self.bind_group = self.device.create_bind_group(&BindGroupDescriptor {
                label: None,
                layout: &self.bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&self.texture_view),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Sampler(&self.sampler),
                    },
                ],
            });
        }

        self.queue.write_texture(
            TexelCopyTextureInfo {
                aspect: TextureAspect::All,
                texture: &self.texture,
                mip_level: 0,
                origin: Origin3d {
                    x: rect.x,
                    y: rect.y,
                    z: 0,
                },
            },
            buffer,
            TexelCopyBufferLayout {
                bytes_per_row: Some(rect.width * 4),
                rows_per_image: Some(rect.height),
                offset: 0,
            },
            Extent3d {
                width: rect.width,
                height: rect.height,
                depth_or_array_layers: 1,
            },
        );

        if let Ok(output) = self.surface.get_current_texture() {
            let view = output
                .texture
                .create_view(&TextureViewDescriptor::default());

            let mut encoder = self
                .device
                .create_command_encoder(&CommandEncoderDescriptor { label: None });

            {
                let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Clear(Color::BLACK),
                            store: StoreOp::Store,
                        },
                    })],
                    ..Default::default()
                });

                render_pass.set_pipeline(&self.pipeline);
                render_pass.set_bind_group(0, Some(&self.bind_group), &[]);
                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);
                render_pass.draw_indexed(0..Vertex::INDICES.len() as u32, 0, 0..1);
            }

            self.queue.submit(Some(encoder.finish()));
            output.present();
        }
    }
}
