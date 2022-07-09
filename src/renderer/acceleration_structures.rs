use ash::Device;
use ash::extensions::khr::AccelerationStructure;
use ash::vk::{AabbPositionsKHR, AccelerationStructureBuildGeometryInfoKHR, AccelerationStructureBuildRangeInfoKHR, AccelerationStructureBuildTypeKHR, AccelerationStructureCreateInfoKHR, AccelerationStructureGeometryAabbsDataKHR, AccelerationStructureGeometryDataKHR, AccelerationStructureGeometryKHR, AccelerationStructureTypeKHR, Buffer, BufferUsageFlags, BuildAccelerationStructureFlagsKHR, BuildAccelerationStructureModeKHR, DeviceOrHostAddressConstKHR, GeometryFlagsKHR, GeometryTypeKHR, MemoryPropertyFlags, PhysicalDeviceMemoryProperties};
use crate::buffer::Buffers;
use crate::renderer::backends::Backends;
use classical_raytracer::Vertex;

pub struct AccelerationStructures {
    pub acceleration_structure: AccelerationStructure,

}

impl AccelerationStructures {
    pub fn new(backends: &Backends, device_memory_properties: PhysicalDeviceMemoryProperties) -> Self {
        let acceleration_structure
            = AccelerationStructure::new(&backends.instance, &backends.device);

        let bottom_acceleration = Self::create_bottom_acceleration(
            &backends.device,
            device_memory_properties,
            &acceleration_structure
        );

        Self {
            acceleration_structure,
        }
    }

    pub fn create_bottom_acceleration(
        device: &Device,
        device_memory_properties: PhysicalDeviceMemoryProperties,
        acceleration_structure: &AccelerationStructure
    ) ->  {
        const VERTICES: [Vertex; 3] = [

        ];


        let geometry = AccelerationStructureGeometryKHR::builder()
            //Dataのタイプ
            .geometry_type(GeometryTypeKHR::AABBS)
            //このASを作るためのデータ設定
            .geometry(AccelerationStructureGeometryDataKHR {
                aabbs: AccelerationStructureGeometryAabbsDataKHR::builder()
                    .data(DeviceOrHostAddressConstKHR {
                        device_address: unsafe {
                            aabb_buffers.get_buffer_address()
                        }
                    })
                    .stride(std::mem::size_of::<AabbPositionsKHR>() as u64)
                    .build(),
            })
            //OPAQUEはany-hitシェーダを呼び出さない
            .flags(GeometryFlagsKHR::OPAQUE)
            .build();

        let build_range_info = AccelerationStructureBuildRangeInfoKHR::builder()
            .first_vertex(0)
            .primitive_count(1)
            .transform_offset(0)
            .transform_offset(0)
            .build();

        let geometries = [geometry];

        let mut build_info = AccelerationStructureBuildGeometryInfoKHR::builder()
            .flags(BuildAccelerationStructureFlagsKHR::PREFER_FAST_TRACE)
            .geometries(&geometries)
            .mode(BuildAccelerationStructureModeKHR::BUILD)
            .ty(AccelerationStructureTypeKHR::BOTTOM_LEVEL)
            .build();

        let size_info = unsafe {
            acceleration_structure.get_acceleration_structure_build_sizes(
                AccelerationStructureBuildTypeKHR::DEVICE,
                &build_info,
                &[1]
            )
        };

        let bottom_as_buffer = Buffers::new(
            size_info.acceleration_structure_size,
            BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR
                | BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | BufferUsageFlags::STORAGE_BUFFER,
            MemoryPropertyFlags::DEVICE_LOCAL,
            device,
            device_memory_properties,
        );

        let as_create_info = AccelerationStructureCreateInfoKHR::builder()
            .ty(build_info.ty)
            .size(size_info.acceleration_structure_size)
            .buffer(bottom_as_buffer.raw)
            .offset(0)
            .build();

        let bottom_as = unsafe {
            acceleration_structure
                .create_acceleration_structure(&as_create_info, None)
                .unwrap()
        };

        build_info.dst_acceleration_structure = bottom_as;

        let scratch_buffer = Buffers::new(
            size_info.build_scratch_size,
            BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | BufferUsageFlags::STORAGE_BUFFER,
            MemoryPropertyFlags::DEVICE_LOCAL,
            device,
            device_memory_properties
        );
    }
}