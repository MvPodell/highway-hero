use crate::WGPU;
use crate::gpu::USE_STORAGE;
use bytemuck::{Pod, Zeroable};
use std::borrow::Cow;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub struct GPUCamera {
    pub(crate)screen_pos: [f32; 2],
    pub(crate)screen_size: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod)]
pub struct GPUSprite {
    pub(crate)screen_region: [f32; 4],
    pub(crate)sheet_region: [f32; 4],
    //cell_region:[f32; 16],
}

#[allow(dead_code)]
pub struct SpriteGroup {
    tex: wgpu::Texture,
    sprite_buffer: wgpu::Buffer,
    sprites: Vec<GPUSprite>,
    camera: GPUCamera,
    camera_buffer: wgpu::Buffer,
    tex_bind_group: wgpu::BindGroup,
    sprite_bind_group: wgpu::BindGroup,
}

pub struct SpriteRenderer {
    pub(crate)pipeline: wgpu::RenderPipeline,
    pub(crate)sprite_bind_group_layout: wgpu::BindGroupLayout,
    pub(crate)texture_bind_group_layout: wgpu::BindGroupLayout,
    pub(crate)groups: Vec<SpriteGroup>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SpriteOption {
    Storage,
    Uniform,
    VertexBuffer,
}

#[cfg(all(not(feature = "uniforms"), not(feature = "vbuf")))]
pub const SPRITES: SpriteOption = SpriteOption::Storage;
#[cfg(feature = "uniforms")]
const SPRITES: SpriteOption = SpriteOption::Uniform;
#[cfg(feature = "vbuf")]
const SPRITES: SpriteOption = SpriteOption::VertexBuffer;
#[cfg(all(feature = "vbuf", feature = "uniform"))]
compile_error!("Can't choose both vbuf and uniform sprite features");


impl SpriteRenderer {
    pub(crate) fn new(gpu: &WGPU) -> Self {
        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
            });

        let texture_bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    // It needs the first entry for the texture and the second for the sampler.
                    // This is like defining a type signature.
                    entries: &[
                        // The texture binding
                        wgpu::BindGroupLayoutEntry {
                            // This matches the binding in the shader
                            binding: 0,
                            // Only available in the fragment shader
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            // It's a texture binding
                            ty: wgpu::BindingType::Texture {
                                // We can use it with float samplers
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                // It's being used as a 2D texture
                                view_dimension: wgpu::TextureViewDimension::D2,
                                // This is not a multisampled texture
                                multisampled: false,
                            },
                            count: None,
                        },
                        // The sampler binding
                        wgpu::BindGroupLayoutEntry {
                            // This matches the binding in the shader
                            binding: 1,
                            // Only available in the fragment shader
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            // It's a sampler
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            // No count
                            count: None,
                        },
                    ],
                });
        // The camera binding
        let camera_layout_entry = wgpu::BindGroupLayoutEntry {
            // This matches the binding in the shader
            binding: 0,
            // Available in vertex shader
            visibility: wgpu::ShaderStages::VERTEX,
            // It's a buffer
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            // No count, not a buffer array binding
            count: None,
        };
        let sprite_bind_group_layout = if USE_STORAGE {
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[
                        camera_layout_entry,
                        wgpu::BindGroupLayoutEntry {
                            // This matches the binding in the shader
                            binding: 1,
                            // Available in vertex shader
                            visibility: wgpu::ShaderStages::VERTEX,
                            // It's a buffer
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            // No count, not a buffer array binding
                            count: None,
                        },
                    ],
                })
        } else {
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[camera_layout_entry],
                })
        };
        let pipeline_layout = gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&sprite_bind_group_layout, &texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let pipeline = gpu
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: if USE_STORAGE {
                        "vs_storage_main"
                    } else {
                        "vs_vbuf_main"
                    },
                    buffers: if USE_STORAGE {
                        &[]
                    } else {
                        &[wgpu::VertexBufferLayout {
                            array_stride: std::mem::size_of::<GPUSprite>() as u64,
                            step_mode: wgpu::VertexStepMode::Instance,
                            attributes: &[
                                wgpu::VertexAttribute {
                                    format: wgpu::VertexFormat::Float32x4,
                                    offset: 0,
                                    shader_location: 0,
                                },
                                wgpu::VertexAttribute {
                                    format: wgpu::VertexFormat::Float32x4,
                                    offset: std::mem::size_of::<[f32; 4]>() as u64,
                                    shader_location: 1,
                                },
                            ],
                        }]
                    },
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(gpu.config.format.into())],
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
            });

        Self {
            pipeline,
            groups: Vec::default(),
            sprite_bind_group_layout,
            texture_bind_group_layout,
        }
    }
}