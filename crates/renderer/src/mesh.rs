use regnum_assets::Mesh;
use wgpu::util::DeviceExt;

use crate::gpu_assets::GpuAsset;

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

    pub fn draw(&self, render_pass: &mut wgpu::RenderPass<'_>, instance: u32) {
        if self.index_count == 0 {
            return;
        }

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.index_count, 0, instance..instance + 1);
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
