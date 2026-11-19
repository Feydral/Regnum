use glam::Vec3;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: Vec3,
    pub color: Vec3,
}

impl Vertex {
    pub const ATTRIBUTES: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        0 => Float32x3,
        1 => Float32x3
    ];

    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

pub fn cube(half: f32) -> (Vec<Vertex>, Vec<u16>) {
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

    (vertices, indices)
}
