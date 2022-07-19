use ash::Device;
use crate::renderer::acceleration_structures::triangle_bottom_level_acceleration_structure::TriangleBottomLevelAccelerationStructure;

struct Scene<'a> {
    device: &'a Device,
    triangle_bottom_level_acceleration_structure: TriangleBottomLevelAccelerationStructure<'a>,
}

impl Scene {

}
