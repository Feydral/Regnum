use std::sync::Arc;

use anyhow::{Context, Result};
use winit::{event_loop::ActiveEventLoop, window::Window};

use regnum_assets::AssetManager;

use crate::{
    camera::GpuCamera,
    depth::{DEPTH_FORMAT, DepthTexture},
    frame::Frame,
    gpu_assets::GpuAssets,
    mesh::Mesh,
    vertex::Vertex,
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
    gpu_meshes: GpuAssets<Mesh>,
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

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Triangle Shader"),
            source: wgpu::ShaderSource::Wgsl(
                r#"
                    struct VertexOutput {
                        @builtin(position) position: vec4<f32>,
                        @location(0) color: vec3<f32>,
                    };

                    struct Camera {
                        view_proj: mat4x4<f32>,
                    };

                    @group(0) @binding(0) var<uniform> camera: Camera;

                    @vertex
                    fn vs_main(
                        @location(0) position: vec3<f32>,
                        @location(1) color: vec3<f32>
                    ) -> VertexOutput {
                        var output: VertexOutput;
                        output.position = camera.view_proj * vec4<f32>(position, 1.0);
                        output.color = color;
                        return output;
                    }

                    @fragment
                    fn fs_main(
                        input: VertexOutput
                    ) -> @location(0) vec4<f32> {
                        return vec4<f32>(input.color, 1.0);
                    }
                    "#
                .into(),
            ),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[Some(&camera.layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Triangle Pipeline"),
            layout: Some(&pipeline_layout),

            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(Vertex::layout())],
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

            primitive: wgpu::PrimitiveState::default(),
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
            gpu_meshes: GpuAssets::default(),
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
        self.camera
            .update(&self.queue, &frame.camera, self.aspect());

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

            for handle in &frame.meshes {
                if let Some(gpu_mesh) = self.gpu_meshes.get(*handle) {
                    gpu_mesh.draw(&mut render_pass);
                }
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
