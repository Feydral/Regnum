use glam::Vec3;
use wgpu::util::DeviceExt;

use crate::vertex::Vertex;

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u16>) -> Self {
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
            let base = vertices.len() as u16;

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
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Mesh Vertex Buffer"),
            contents: bytemuck::cast_slice(&mesh.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Mesh Index Buffer"),
            contents: bytemuck::cast_slice(&mesh.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            vertex_buffer,
            index_buffer,
            index_count: mesh.indices.len() as u32,
        }
    }

    pub fn draw(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.index_count, 0, 0..1);
    }
}
