use ash::Device;
use ash::vk::TransformMatrixKHR;

pub struct Instances<'a> {
    device: &'a Device,
    transform: TransformMatrixKHR,

}

impl Instances {
    pub fn new() -> Self {
        
    }
}

impl Drop for Instances {
    fn drop(&mut self) {
        //todo!()
    }
}