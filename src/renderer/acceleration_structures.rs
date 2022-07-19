use ash::Device;
use ash::extensions::khr::AccelerationStructure;
use ash::vk::{PhysicalDeviceMemoryProperties, Queue};
use crate::renderer::acceleration_structures::triangle_bottom_level_acceleration_structure::TriangleBottomLevelAccelerationStructure;
use crate::renderer::backends::Backends;
use crate::renderer::backends::commands::Commands;

pub mod triangle_bottom_level_acceleration_structure;
pub mod instances;

pub struct AccelerationStructures<'a> {
    backends: &'a Backends,
    pub acceleration_structure: AccelerationStructure,
}

impl<'a> AccelerationStructures<'a>{
    pub fn new(backends: &'a Backends) -> Self {
        let acceleration_structure
            = AccelerationStructure::new(&backends.instance, &backends.device);

        Self {
            backends: &backends,
            acceleration_structure,
        }
    }

    pub fn create_triangle_blas(
        &self,
        device_memory_properties: PhysicalDeviceMemoryProperties,
        commands: &Commands,
        graphics_queue: Queue,
    ) -> TriangleBottomLevelAccelerationStructure {
        TriangleBottomLevelAccelerationStructure::new(
            self.backends,
            &self.acceleration_structure,
            device_memory_properties,
            commands,
            graphics_queue
        )
    }
}

