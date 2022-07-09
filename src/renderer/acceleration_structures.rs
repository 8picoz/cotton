use ash::Device;
use ash::extensions::khr::AccelerationStructure;
use ash::vk::{AabbPositionsKHR, AccelerationStructureBuildGeometryInfoKHR, AccelerationStructureBuildRangeInfoKHR, AccelerationStructureBuildTypeKHR, AccelerationStructureCreateInfoKHR, AccelerationStructureGeometryAabbsDataKHR, AccelerationStructureGeometryDataKHR, AccelerationStructureGeometryKHR, AccelerationStructureGeometryTrianglesDataKHR, AccelerationStructureTypeKHR, Buffer, BufferUsageFlags, BuildAccelerationStructureFlagsKHR, BuildAccelerationStructureModeKHR, CommandBuffer, CommandBufferBeginInfo, CommandBufferUsageFlags, CommandPool, DeviceOrHostAddressConstKHR, DeviceOrHostAddressKHR, DeviceSize, Fence, GeometryFlagsKHR, GeometryTypeKHR, IndexType, MemoryPropertyFlags, PhysicalDeviceMemoryProperties, Queue, SubmitInfo};
use crate::buffer::Buffers;
use crate::renderer::backends::Backends;
use classical_raytracer::Vertex;
use classical_raytracer_shader::vertex::Vertex;
use crate::renderer::commands::Commands;

pub struct AccelerationStructures {
    pub acceleration_structure: AccelerationStructure,

}

impl AccelerationStructures {
    pub fn new(
        backends: &Backends,
        device_memory_properties: PhysicalDeviceMemoryProperties,
        commands: Commands,
        graphics_queue: Queue,
    ) -> Self {
        let acceleration_structure
            = AccelerationStructure::new(&backends.instance, &backends.device);

        let bottom_acceleration = Self::create_bottom_acceleration(
            &backends.device,
            device_memory_properties,
            &acceleration_structure,
            commands,
            graphics_queue,
        );

        Self {
            acceleration_structure,
        }
    }

    pub fn create_bottom_acceleration(
        device: &Device,
        device_memory_properties: PhysicalDeviceMemoryProperties,
        acceleration_structure: &AccelerationStructure,
        commands: Commands,
        graphics_queue: Queue,
    ) -> (AccelerationStructure, Buffers, Buffers, Buffers) {
        //とりあえず定数
        const VERTICES: [Vertex; 3] = [
            Vertex {
                position: const_vec3a!([1.0, -1.0, 0.0]),
                normal: const_vec3a!([0.0, 0.0, 1.0]),
            },
            Vertex {
                position: const_vec3a!([0.0, 1.0, 0.0]),
                normal: const_vec3a!([0.0, 0.0, 1.0]),
            },
            Vertex {
                position: const_vec3a!([-1.0, -1.0, 0.0]),
                normal: const_vec3a!([0.0, 0.0, 1.0]),
            },
        ];

        const INDICES: [u32; 3] = [0, 1, 2];

        let vertex_stride = std::mem::size_of::<Vertex>();
        let vertex_buffer_size = vertex_stride * VERTICES.len();
        let max_vertex = VERTICES.len() as u32 - 1;

        let mut vertex_buffer = Buffers::new(
            vertex_buffer_size as DeviceSize,
            BufferUsageFlags::STORAGE_BUFFER
                | BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR,
            MemoryPropertyFlags::HOST_VISIBLE
                | MemoryPropertyFlags::HOST_COHERENT,
            device,
            device_memory_properties
        );

        vertex_buffer.store(&VERTICES);

        let index_buffer_size = std::mem::size_of::<u32>() * INDICES.len();

        let mut index_buffer = Buffers::new(
            index_buffer_size as DeviceSize,
            BufferUsageFlags::STORAGE_BUFFER
                | BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR,
            MemoryPropertyFlags::HOST_VISIBLE
                | MemoryPropertyFlags::HOST_COHERENT,
            device,
            device_memory_properties
        );

        index_buffer.store(&INDICES);

        let geometry = AccelerationStructureGeometryKHR::builder()
            //Dataのタイプ
            .geometry_type(GeometryTypeKHR::TRIANGLES)
            //このASを作るためのデータ設定
            .geometry(AccelerationStructureGeometryDataKHR {
                triangles: AccelerationStructureGeometryTrianglesDataKHR::builder()
                    .vertex_data(DeviceOrHostAddressConstKHR {
                        device_address: unsafe {
                            vertex_buffer.get_buffer_address()
                        },
                    })
                    .max_vertex(max_vertex)
                    .vertex_stride(vertex_stride as u64)
                    .index_data(DeviceOrHostAddressConstKHR {
                        device_address: unsafe {
                            index_buffer.get_buffer_address()
                        }
                    })
                    .index_type(IndexType::UINT32)
                    .build(),
            })
            //OPAQUEはany-hitシェーダを呼び出さない
            //.flags(GeometryFlagsKHR::OPAQUE)
            .build();

        let build_range_info = AccelerationStructureBuildRangeInfoKHR::builder()
            .primitive_count(INDICES.len() as u32 / 3)
            .build();

        let geometries = [geometry];

        let scratch_buffer = Buffers::new(
            size_info.build_scratch_size,
            BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | BufferUsageFlags::STORAGE_BUFFER,
            MemoryPropertyFlags::DEVICE_LOCAL,
            device,
            device_memory_properties
        );

        let scratch_data = DeviceOrHostAddressKHR {
            device_address: unsafe {
                scratch_buffer.get_buffer_address()
            }
        };

        let bottom_accel_buffer = Buffers::new(
            size_info.acceleration_structure_size,
            BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR
                | BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | BufferUsageFlags::STORAGE_BUFFER,
            MemoryPropertyFlags::DEVICE_LOCAL,
            device,
            device_memory_properties,
        );

        let bottom_accel_create_info = AccelerationStructureCreateInfoKHR::builder()
            .ty(AccelerationStructureTypeKHR::BOTTOM_LEVEL)
            .size(size_info.acceleration_structure_size)
            .buffer(bottom_accel_buffer.buffer)
            .build();

        let bottom_accel = unsafe {
            acceleration_structure
                .create_acceleration_structure(&bottom_accel_create_info, None)
                .unwrap()
        };

        let mut build_info = AccelerationStructureBuildGeometryInfoKHR::builder()
            .flags(BuildAccelerationStructureFlagsKHR::PREFER_FAST_TRACE)
            .geometries(&geometries)
            .mode(BuildAccelerationStructureModeKHR::BUILD)
            .ty(AccelerationStructureTypeKHR::BOTTOM_LEVEL)
            .scratch_data(scratch_data)
            .dst_acceleration_structure(bottom_accel)
            .build();

        let memory_requirements = unsafe {
            acceleration_structure.get_acceleration_structure_build_sizes(
                AccelerationStructureBuildTypeKHR::DEVICE,
                &build_info,
                //geometriesに対応するように配列を作成する
                &[INDICES.len() as u32 / 3]
            )
        };

        let cb_begin_info = CommandBufferBeginInfo::builder()
            .flags(CommandBufferUsageFlags::ONE_TIME_SUBMIT)
            .build();

        let build_cb = commands.command_buffers[0];

        unsafe {
            device.begin_command_buffer(
                build_cb,
                &cb_begin_info
            ).unwrap();
        }

        let build_infos = &[build_info];
        let build_range_infos: &[&[_]] = &[&[build_range_info]];

        unsafe {
            acceleration_structure.cmd_build_acceleration_structures(
                build_cb,
                build_infos,
                build_range_infos,
            );
            device.end_command_buffer(build_cb).unwrap();

            device.queue_submit(
                graphics_queue,
                &[SubmitInfo::builder()
                    .command_buffers(&[build_cb])
                    .build()
                ],
                Fence::null(),
            ).expect("submit failed");

            //Queueの処理が終わるまで待機
            device.queue_wait_idle(graphics_queue).unwrap();
            device.free_command_buffers(commands.command_pool, build_cb);

            device.destroy_buffer(scratch_buffer.buffer, None);
            device.free_memory(scratch_buffer.memory, None);
        }
    }
}

impl Drop for AccelerationStructures {
    fn drop(&mut self) {
        todo!()
    }
}