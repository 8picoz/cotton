use ash::extensions::khr::AccelerationStructure;
use ash::vk::AabbPositionsKHR;
use crate::renderer::backends::Backends;

pub struct AccelerationStructures {
    pub acceleration_structure: AccelerationStructure,

}

impl AccelerationStructures {
    pub fn new(backends: &Backends) -> Self {
        let acceleration_structure
            = AccelerationStructure::new(&backends.instance, &backends.device);

        //bottom-level
        let aabb = AabbPositionsKHR::builder()
            .min_x(-1.0)
            .max_x(1.0)
            .min_y(-1.0)
            .min_z(-1.0)
            .max_z(1.0)
            .build();


        Self {
            acceleration_structure,
        }
    }

    pub fn create_bottom_acceleration() ->  {

    }
}