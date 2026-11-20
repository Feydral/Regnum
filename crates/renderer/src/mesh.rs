use glam::Vec3;
use wgpu::util::DeviceExt;

use crate::{gpu_assets::GpuAsset, vertex::Vertex};

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        Self { vertices, indices }
    }

    pub fn cube(half: f32) -> Self {
        const FACES: [(Vec3, Vec3, Vec3, Vec3); 6] = [
            (Vec3::X, Vec3::Y, Vec3::Z, Vec3::new(1.0, 0.2, 0.2)),
            (Vec3::NEG_X, Vec3::Z, Vec3::Y, Vec3::new(0.2, 1.0, 0.2)),
            (Vec3::Y, Vec3::Z, Vec3::X, Vec3::new(0.2, 0.3, 1.0)),
            (Vec3::NEG_Y, Vec3::X, Vec3::Z, Vec3::new(1.0, 1.0, 0.2)),
            (Vec3::Z, Vec3::X, Vec3::Y, Vec3::new(1.0, 0.2, 1.0)),
            (Vec3::NEG_Z, Vec3::Y, Vec3::X, Vec3::new(0.2, 1.0, 1.0)),
        ];

        let mut vertices = Vec::with_capacity(24);
        let mut indices = Vec::with_capacity(36);

        for (n, u, v, color) in FACES {
            let base = vertices.len() as u32;

            for (su, sv) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
                vertices.push(Vertex {
                    position: (n + u * su + v * sv) * half,
                    color,
                });
            }

            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }

        Self::new(vertices, indices)
    }
}

pub struct GpuMesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
}

impl GpuMesh {
    pub fn new(device: &wgpu::Device, mesh: &Mesh) -> Self {
        Self {
            vertex_buffer: create_buffer(
                device,
                "Mesh Vertex Buffer",
                wgpu::BufferUsages::VERTEX,
                bytemuck::cast_slice(&mesh.vertices),
            ),
            index_buffer: create_buffer(
                device,
                "Mesh Index Buffer",
                wgpu::BufferUsages::INDEX,
                bytemuck::cast_slice(&mesh.indices),
            ),
            index_count: mesh.indices.len() as u32,
        }
    }

    pub fn draw(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        if self.index_count == 0 {
            return;
        }

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.index_count, 0, 0..1);
    }
}

impl GpuAsset for Mesh {
    type Gpu = GpuMesh;

    fn upload(&self, device: &wgpu::Device, _queue: &wgpu::Queue) -> GpuMesh {
        GpuMesh::new(device, self)
    }

    fn reupload(&self, gpu: &mut GpuMesh, device: &wgpu::Device, queue: &wgpu::Queue) {
        update_buffer(
            device,
            queue,
            &mut gpu.vertex_buffer,
            "Mesh Vertex Buffer",
            wgpu::BufferUsages::VERTEX,
            bytemuck::cast_slice(&self.vertices),
        );
        update_buffer(
            device,
            queue,
            &mut gpu.index_buffer,
            "Mesh Index Buffer",
            wgpu::BufferUsages::INDEX,
            bytemuck::cast_slice(&self.indices),
        );
        gpu.index_count = self.indices.len() as u32;
    }
}

fn create_buffer(
    device: &wgpu::Device,
    label: &str,
    usage: wgpu::BufferUsages,
    contents: &[u8],
) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents,
        usage: usage | wgpu::BufferUsages::COPY_DST,
    })
}

fn update_buffer(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    buffer: &mut wgpu::Buffer,
    label: &str,
    usage: wgpu::BufferUsages,
    contents: &[u8],
) {
    if contents.len() as u64 > buffer.size() {
        *buffer = create_buffer(device, label, usage, contents);
    } else if !contents.is_empty() {
        queue.write_buffer(buffer, 0, contents);
    }
}
