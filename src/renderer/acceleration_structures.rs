use ash::Device;
use ash::extensions::khr::AccelerationStructure;
use ash::vk::{PhysicalDeviceMemoryProperties, Queue};
use log::debug;
use crate::renderer::acceleration_structures::top_level_acceleration_structures::TopLevelAccelerationStructures;
use crate::renderer::acceleration_structures::triangle_bottom_level_acceleration_structure::TriangleBottomLevelAccelerationStructure;
use crate::renderer::backends::Backends;
use crate::renderer::backends::commands::Commands;
use crate::scene::Scene;

pub mod triangle_bottom_level_acceleration_structure;
pub mod top_level_acceleration_structures;

pub struct AccelerationStructures<'a> {
    backends: &'a Backends,
    pub acceleration_structure: AccelerationStructure,
}

impl<'a> AccelerationStructures<'a>{
    pub fn new(backends: &'a Backends) -> Self {
        debug!("create AccelerationStructures");

        let acceleration_structure
            = AccelerationStructure::new(&backends.instance, &backends.device);

        Self {
            backends: &backends,
            acceleration_structure,
        }
    }

    pub fn create_triangle_blas(
        &self,
        graphics_queue: Queue,
    ) -> TriangleBottomLevelAccelerationStructure {
        debug!("create triangle blas");

        TriangleBottomLevelAccelerationStructure::new(
            self.backends,
            &self.acceleration_structure,
            graphics_queue
        )
    }

    pub fn create_tlas(
        &self,
        scene: Scene,
        graphics_queue: Queue,
    ) -> TopLevelAccelerationStructures {
        debug!("create tlas");

        TopLevelAccelerationStructures::new(
            self.backends,
            &self.acceleration_structure,
            scene,
            graphics_queue
        )
    }
}

