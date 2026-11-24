use std::sync::Arc;

use anyhow::{Context, Result};
use winit::{event_loop::ActiveEventLoop, window::Window};

use regnum_assets::{AssetManager, Mesh, Texture};

use crate::{
    camera::GpuCamera,
    depth::{DEPTH_FORMAT, DepthTexture},
    frame::Frame,
    gpu_assets::GpuAssets,
    instance::{InstanceBuffer, InstanceRaw, instance_layout},
    texture::GpuTexture,
    vertex::vertex_layout,
};

pub struct Renderer {
    instance: wgpu::Instance,
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,

    depth: DepthTexture,
    pipeline: wgpu::RenderPipeline,
    camera: GpuCamera,
    instances: InstanceBuffer,
    gpu_meshes: GpuAssets<Mesh>,
    gpu_textures: GpuAssets<Texture>,
}

impl Renderer {
    pub async fn new(event_loop: &ActiveEventLoop, window: Arc<Window>) -> Result<Self> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_with_display_handle(
            Box::new(event_loop.owned_display_handle()),
        ));

        let surface = instance.create_surface(window.clone())?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await?;

        let size = window.inner_size();

        let config = surface
            .get_default_config(&adapter, size.width.max(1), size.height.max(1))
            .context("No supported surface configuration")?;

        surface.configure(&device, &config);

        let depth = DepthTexture::new(&device, config.width, config.height);
        let camera = GpuCamera::new(&device);
        let instances = InstanceBuffer::new(&device);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Model Shader"),
            source: wgpu::ShaderSource::Wgsl(
                r#"
                    struct VertexOutput {
                        @builtin(position) position: vec4<f32>,
                        @location(0) normal: vec3<f32>,
                        @location(1) uv: vec2<f32>,
                    };

                    struct Camera {
                        view_proj: mat4x4<f32>,
                    };

                    @group(0) @binding(0) var<uniform> camera: Camera;
                    @group(1) @binding(0) var atlas: texture_2d<f32>;
                    @group(1) @binding(1) var atlas_sampler: sampler;

                    const LIGHT_DIRECTION: vec3<f32> = vec3<f32>(0.4, 0.8, 0.3);
                    const AMBIENT: f32 = 0.45;

                    @vertex
                    fn vs_main(
                        @location(0) position: vec3<f32>,
                        @location(1) normal: vec3<f32>,
                        @location(2) uv: vec2<f32>,
                        @location(3) model_0: vec4<f32>,
                        @location(4) model_1: vec4<f32>,
                        @location(5) model_2: vec4<f32>,
                        @location(6) model_3: vec4<f32>,
                        @location(7) normal_0: vec3<f32>,
                        @location(8) normal_1: vec3<f32>,
                        @location(9) normal_2: vec3<f32>
                    ) -> VertexOutput {
                        let model = mat4x4<f32>(model_0, model_1, model_2, model_3);
                        let normal_matrix = mat3x3<f32>(normal_0, normal_1, normal_2);
                        var output: VertexOutput;
                        output.position = camera.view_proj * model * vec4<f32>(position, 1.0);
                        output.normal = normal_matrix * normal;
                        output.uv = uv;
                        return output;
                    }

                    @fragment
                    fn fs_main(
                        input: VertexOutput
                    ) -> @location(0) vec4<f32> {
                        let diffuse = max(dot(normalize(input.normal), normalize(LIGHT_DIRECTION)), 0.0);
                        let light = AMBIENT + (1.0 - AMBIENT) * diffuse;
                        let texel = textureSample(atlas, atlas_sampler, input.uv);
                        return vec4<f32>(texel.rgb * light, 1.0);
                    }
                    "#
                .into(),
            ),
        });

        let texture_layout = GpuTexture::bind_group_layout(&device);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[Some(&camera.layout), Some(&texture_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Model Pipeline"),
            layout: Some(&pipeline_layout),

            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(vertex_layout()), Some(instance_layout())],
                compilation_options: Default::default(),
            },

            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),

            primitive: wgpu::PrimitiveState {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Greater),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        Ok(Self {
            instance,
            window,
            surface,
            device,
            queue,
            config,
            depth,
            pipeline,
            camera,
            instances,
            gpu_meshes: GpuAssets::default(),
            gpu_textures: GpuAssets::default(),
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }

        self.config.width = width;
        self.config.height = height;

        self.surface.configure(&self.device, &self.config);
        self.depth = DepthTexture::new(&self.device, width, height);
    }

    pub fn aspect(&self) -> f32 {
        self.config.width as f32 / self.config.height as f32
    }

    pub fn render(&mut self, frame: &Frame, assets: &mut AssetManager) -> Result<()> {
        let surface_texture = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture) => texture,

            wgpu::CurrentSurfaceTexture::Suboptimal(texture) => {
                drop(texture);
                self.surface.configure(&self.device, &self.config);
                return Ok(());
            }

            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                return Ok(());
            }

            wgpu::CurrentSurfaceTexture::Lost => {
                self.surface = self.instance.create_surface(self.window.clone())?;

                self.surface.configure(&self.device, &self.config);
                return Ok(());
            }

            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return Ok(());
            }

            wgpu::CurrentSurfaceTexture::Validation => {
                anyhow::bail!("Surface validation error");
            }
        };

        self.gpu_meshes
            .sync(&self.device, &self.queue, assets.storage_mut::<Mesh>());
        self.gpu_textures
            .sync(&self.device, &self.queue, assets.storage_mut::<Texture>());
        self.camera
            .update(&self.queue, &frame.camera, self.aspect());

        let draws: Vec<_> = frame
            .models
            .iter()
            .filter_map(|model| {
                Some((
                    self.gpu_meshes.get(model.mesh)?,
                    self.gpu_textures.get(model.texture)?,
                    InstanceRaw::from(&model.transform),
                ))
            })
            .collect();
        let instances: Vec<InstanceRaw> = draws.iter().map(|draw| draw.2).collect();
        self.instances.update(&self.device, &self.queue, &instances);

        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),

                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],

                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0.0),
                        store: wgpu::StoreOp::Discard,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.camera.bind_group, &[]);

            render_pass.set_vertex_buffer(1, self.instances.buffer().slice(..));

            for (index, (gpu_mesh, gpu_texture, _)) in draws.iter().enumerate() {
                render_pass.set_bind_group(1, &gpu_texture.bind_group, &[]);
                gpu_mesh.draw(&mut render_pass, index as u32);
            }
        }

        self.queue.submit([encoder.finish()]);

        self.window.pre_present_notify();
        self.queue.present(surface_texture);

        Ok(())
    }

    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }
}
