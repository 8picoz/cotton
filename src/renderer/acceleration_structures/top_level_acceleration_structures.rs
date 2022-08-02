use ash::Device;
use ash::extensions::khr::AccelerationStructure;
use ash::vk::{AccelerationStructureBuildGeometryInfoKHR, AccelerationStructureBuildRangeInfoKHR, AccelerationStructureBuildTypeKHR, AccelerationStructureCreateInfoKHR, AccelerationStructureGeometryDataKHR, AccelerationStructureGeometryInstancesDataKHR, AccelerationStructureGeometryKHR, AccelerationStructureKHR, AccelerationStructureTypeKHR, AccessFlags, BufferUsageFlags, BuildAccelerationStructureFlagsKHR, BuildAccelerationStructureModeKHR, CommandBufferAllocateInfo, CommandBufferBeginInfo, CommandBufferLevel, CommandBufferUsageFlags, DependencyFlags, DeviceOrHostAddressConstKHR, DeviceOrHostAddressKHR, Fence, GeometryTypeKHR, MemoryBarrier, MemoryPropertyFlags, PipelineStageFlags, Queue, SubmitInfo};
use crate::buffers::Buffers;
use crate::renderer::backends::Backends;
use crate::scene::Scene;

pub struct TopLevelAccelerationStructures<'a> {
    backends: &'a Backends,
    acceleration_structure: &'a AccelerationStructure,
    pub top_level_acceleration_structure_khr: AccelerationStructureKHR,
    pub top_level_acceleration_structure_buffer: Buffers<'a>,
}

impl<'a> TopLevelAccelerationStructures<'a> {
    pub fn new(
        backends: &'a Backends,
        acceleration_structure: &'a AccelerationStructure,
        scene: Scene,
        graphics_queue: Queue,
    ) -> Self {
        let build_range_info = AccelerationStructureBuildRangeInfoKHR::builder()
            .first_vertex(0)
            //BLASの個数
            .primitive_count(scene.instances.len() as u32)
            .primitive_offset(0)
            .transform_offset(0)
            .build();

        let command_pool = backends.create_graphics_command_pool();
        let command_buffers = backends.create_command_buffers(command_pool, 1);
        let build_command_buffer = command_buffers[0];

        let instances = AccelerationStructureGeometryInstancesDataKHR::builder()
            .array_of_pointers(false)
            .data(DeviceOrHostAddressConstKHR {
                device_address: scene.instance_buffer.get_buffer_address()
            })
            .build();

        let geometry = AccelerationStructureGeometryKHR::builder()
            .geometry_type(GeometryTypeKHR::INSTANCES)
            .geometry(AccelerationStructureGeometryDataKHR{
                instances
            })
            .build();

        let geometries = [geometry];

        let mut build_info = AccelerationStructureBuildGeometryInfoKHR::builder()
            .ty(AccelerationStructureTypeKHR::TOP_LEVEL)
            .flags(BuildAccelerationStructureFlagsKHR::PREFER_FAST_TRACE)
            .geometries(&geometries)
            .mode(BuildAccelerationStructureModeKHR::BUILD)
            .build();

        let memory_requirements = unsafe {
            acceleration_structure.get_acceleration_structure_build_sizes(
                AccelerationStructureBuildTypeKHR::DEVICE,
                &build_info,
                &[build_range_info.primitive_count],
            )
        };

        let top_level_acceleration_structure_buffer = Buffers::new(
            &backends.device,
            backends.device_memory_properties,
            memory_requirements.acceleration_structure_size,
            BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR
                | BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | BufferUsageFlags::STORAGE_BUFFER,
            MemoryPropertyFlags::DEVICE_LOCAL
        );

        let accel_create_info = AccelerationStructureCreateInfoKHR::builder()
            .ty(build_info.ty)
            .size(memory_requirements.acceleration_structure_size)
            .buffer(top_level_acceleration_structure_buffer.buffer)
            .offset(0)
            .build();

        let top_level_acceleration_structure_khr = unsafe {
            acceleration_structure
                .create_acceleration_structure(&accel_create_info, None)
                .unwrap()
        };

        build_info.dst_acceleration_structure = top_level_acceleration_structure_khr;

        let scratch_buffer = Buffers::new(
            &backends.device,
            backends.device_memory_properties,
            memory_requirements.build_scratch_size,
            BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | BufferUsageFlags::STORAGE_BUFFER,
            MemoryPropertyFlags::DEVICE_LOCAL
        );

        build_info.scratch_data = DeviceOrHostAddressKHR {
            device_address: scratch_buffer.get_buffer_address()
        };

        unsafe {
            backends.device
                .begin_command_buffer(
                    build_command_buffer,
                    &CommandBufferBeginInfo::builder()
                        .flags(CommandBufferUsageFlags::ONE_TIME_SUBMIT)
                        .build(),
                )
                .unwrap();

            let memory_barrier = MemoryBarrier::builder()
                .src_access_mask(AccessFlags::TRANSFER_WRITE)
                .dst_access_mask(AccessFlags::ACCELERATION_STRUCTURE_WRITE_KHR)
                .build();

            backends.device.cmd_pipeline_barrier(
                build_command_buffer,
                PipelineStageFlags::TRANSFER,
                PipelineStageFlags::ACCELERATION_STRUCTURE_BUILD_KHR,
                DependencyFlags::empty(),
                &[memory_barrier],
                &[],
                &[]
            );

            acceleration_structure.cmd_build_acceleration_structures(
                build_command_buffer,
                &[build_info],
                &[&[build_range_info]]
            );
            backends.device.end_command_buffer(build_command_buffer).unwrap();
            backends.device.queue_submit(
                graphics_queue,
                &[SubmitInfo::builder()
                    .command_buffers(&[build_command_buffer])
                    .build()],
                Fence::null()
            ).expect("failed queue submit");

            backends.device.queue_wait_idle(graphics_queue).unwrap();
            backends.device.free_command_buffers(command_pool, &command_buffers);
            backends.device.destroy_command_pool(command_pool, None);
        }

        Self {
            backends,
            acceleration_structure,
            top_level_acceleration_structure_khr,
            top_level_acceleration_structure_buffer,
        }
    }
}

impl Drop for TopLevelAccelerationStructures<'_> {
    fn drop(&mut self) {
        //todo!()
    }
}