use regnum_assets::Transform;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    model: [[f32; 4]; 4],
    normal: [[f32; 3]; 3],
}

impl From<&Transform> for InstanceRaw {
    fn from(transform: &Transform) -> Self {
        let model = transform.matrix();
        let normal = glam::Mat3::from_mat4(model).inverse().transpose();
        Self {
            model: model.to_cols_array_2d(),
            normal: normal.to_cols_array_2d(),
        }
    }
}

const ATTRIBUTES: [wgpu::VertexAttribute; 7] = wgpu::vertex_attr_array![
    3 => Float32x4,
    4 => Float32x4,
    5 => Float32x4,
    6 => Float32x4,
    7 => Float32x3,
    8 => Float32x3,
    9 => Float32x3
];

pub fn instance_layout() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &ATTRIBUTES,
    }
}

pub struct InstanceBuffer {
    buffer: wgpu::Buffer,
    capacity: usize,
}

impl InstanceBuffer {
    const INITIAL_CAPACITY: usize = 64;

    pub fn new(device: &wgpu::Device) -> Self {
        Self::with_capacity(device, Self::INITIAL_CAPACITY)
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub fn update(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        instances: &[InstanceRaw],
    ) {
        if instances.len() > self.capacity {
            *self = Self::with_capacity(device, instances.len().next_power_of_two());
        }

        if !instances.is_empty() {
            queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(instances));
        }
    }

    fn with_capacity(device: &wgpu::Device, capacity: usize) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: (capacity * std::mem::size_of::<InstanceRaw>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self { buffer, capacity }
    }
}
