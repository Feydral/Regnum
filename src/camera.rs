use glam::{
    Mat4, Vec3,
    camera::rh::{proj::directx, view},
};
use winit::keyboard::KeyCode;

pub struct Camera {
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub fov_y: f32,
    pub near: f32,
}

impl Camera {
    pub fn new(position: Vec3, yaw_deg: f32, pitch_deg: f32) -> Self {
        Self {
            position,
            yaw: yaw_deg.to_radians(),
            pitch: pitch_deg.to_radians(),
            fov_y: 70f32.to_radians(),
            near: 0.05,
        }
    }

    pub fn forward(&self) -> Vec3 {
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        Vec3::new(cp * cy, sp, cp * sy)
    }

    pub fn right(&self) -> Vec3 {
        self.forward().cross(Vec3::Y).normalize()
    }

    pub fn view(&self) -> Mat4 {
        view::look_to_mat4(self.position, self.forward(), Vec3::Y)
    }

    pub fn projection(&self, aspect: f32) -> Mat4 {
        directx::perspective_infinite_reverse(self.fov_y, aspect, self.near)
    }

    pub fn view_proj(&self, aspect: f32) -> Mat4 {
        self.projection(aspect) * self.view()
    }
}

pub struct CameraController {
    pub speed: f32,
    pub sensitivity: f32,

    forward: bool,
    back: bool,
    left: bool,
    right: bool,
    mouse_delta: (f32, f32),
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            speed,
            sensitivity,
            forward: false,
            back: false,
            left: false,
            right: false,
            mouse_delta: (0.0, 0.0),
        }
    }

    pub fn process_key(&mut self, code: KeyCode, pressed: bool) -> bool {
        let flag = match code {
            KeyCode::KeyW => &mut self.forward,
            KeyCode::KeyS => &mut self.back,
            KeyCode::KeyA => &mut self.left,
            KeyCode::KeyD => &mut self.right,
            _ => return false,
        };
        *flag = pressed;
        true
    }

    pub fn process_mouse(&mut self, dx: f64, dy: f64) {
        self.mouse_delta.0 += dx as f32;
        self.mouse_delta.1 += dy as f32;
    }

    pub fn reset(&mut self) {
        *self = Self::new(self.speed, self.sensitivity);
    }

    pub fn update(&mut self, camera: &mut Camera, dt: f32) {
        let (dx, dy) = std::mem::take(&mut self.mouse_delta);
        camera.yaw += dx * self.sensitivity;
        camera.pitch =
            (camera.pitch - dy * self.sensitivity).clamp(-89f32.to_radians(), 89f32.to_radians());

        let axis = |pos: bool, neg: bool| pos as i8 as f32 - neg as i8 as f32;
        let wish = camera.forward() * axis(self.forward, self.back)
            + camera.right() * axis(self.right, self.left);

        camera.position += wish.normalize_or_zero() * self.speed * dt;
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_proj: Mat4,
}

pub struct GpuCamera {
    pub layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    buffer: wgpu::Buffer,
}

impl GpuCamera {
    pub fn new(device: &wgpu::Device) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Camera Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Uniform Buffer"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        Self {
            layout,
            bind_group,
            buffer,
        }
    }

    pub fn update(&self, queue: &wgpu::Queue, camera: &Camera, aspect: f32) {
        let uniform = CameraUniform {
            view_proj: camera.view_proj(aspect),
        };
        queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(&uniform));
    }
}
