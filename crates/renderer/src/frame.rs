use regnum_assets::Model;

use crate::camera::Camera;

pub struct Frame {
    pub camera: Camera,
    pub models: Vec<Model>,
}
