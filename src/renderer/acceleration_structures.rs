use ash::Device;
use ash::extensions::khr::AccelerationStructure;
use ash::vk::{AabbPositionsKHR, AccelerationStructureBuildGeometryInfoKHR, AccelerationStructureBuildRangeInfoKHR, AccelerationStructureBuildTypeKHR, AccelerationStructureCreateInfoKHR, AccelerationStructureGeometryAabbsDataKHR, AccelerationStructureGeometryDataKHR, AccelerationStructureGeometryKHR, AccelerationStructureGeometryTrianglesDataKHR, AccelerationStructureKHR, AccelerationStructureTypeKHR, Buffer, BufferUsageFlags, BuildAccelerationStructureFlagsKHR, BuildAccelerationStructureModeKHR, CommandBuffer, CommandBufferBeginInfo, CommandBufferUsageFlags, CommandPool, DeviceOrHostAddressConstKHR, DeviceOrHostAddressKHR, DeviceSize, Fence, GeometryFlagsKHR, GeometryTypeKHR, IndexType, MemoryPropertyFlags, PhysicalDeviceMemoryProperties, Queue, SubmitInfo};
use glam::{const_vec3a, vec3a, Vec3A};
use crate::buffers::Buffers;
use crate::renderer::backends::Backends;
use classical_raytracer_shader::vertex::Vertex;
use crate::renderer::commands::Commands;
use crate::renderer::mesh_buffer::MeshBuffer;

pub struct TriangleAccelerationStructure<'a> {
    device: &'a Device,
    pub acceleration_structure: AccelerationStructure,
    pub bottom_acceleration_structure: AccelerationStructureKHR,
    pub top_acceleration_structure: AccelerationStructureKHR,
    pub mesh_buffer: MeshBuffer<'a>,
}

impl<'a> TriangleAccelerationStructure<'a> {
    pub fn new(
        backends: &'a Backends,
        device_memory_properties: PhysicalDeviceMemoryProperties,
        commands: &Commands,
        graphics_queue: Queue,
    ) -> Self {
        let acceleration_structure
            = AccelerationStructure::new(&backends.instance, &backends.device);

        //とりあえず定数(三角形)
        //TODO: 外部から入力できるようにする
        let vertices = vec![
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

        let indices = vec![0, 1, 2];

        let mesh_buffer = MeshBuffer::new(&backends.device, vertices, indices, device_memory_properties);

        //TODO: このbottom asをモデルごとに作成するようにしてtop asと紐づける
        let (
            bottom_acceleration_structure,
            bottom_accel_buffer
        ) = Self::create_bottom_acceleration(
            &backends.device,
            device_memory_properties,
            &acceleration_structure,
            &mesh_buffer,
            commands,
            graphics_queue,
        );

        Self {
            device: &backends.device,
            acceleration_structure,
            bottom_acceleration_structure,
            top_acceleration_structure: Default::default(),
            mesh_buffer,
        }
    }

    pub fn create_bottom_acceleration(
        device: &'a Device,
        device_memory_properties: PhysicalDeviceMemoryProperties,
        acceleration_structure: &AccelerationStructure,
        mesh_buffer: &MeshBuffer,
        commands: &Commands,
        graphics_queue: Queue,
    ) -> (AccelerationStructureKHR, Buffers<'a>) {
        let geometry = AccelerationStructureGeometryKHR::builder()
            //Dataのタイプ
            .geometry_type(GeometryTypeKHR::TRIANGLES)
            //このASを作るためのデータ設定
            .geometry(AccelerationStructureGeometryDataKHR {
                triangles: AccelerationStructureGeometryTrianglesDataKHR::builder()
                    .vertex_data(DeviceOrHostAddressConstKHR {
                        device_address: unsafe {
                            mesh_buffer.vertex_buffer.get_buffer_address()
                        },
                    })
                    .max_vertex(mesh_buffer.max_vertex)
                    .vertex_stride(mesh_buffer.vertex_stride)
                    .index_data(DeviceOrHostAddressConstKHR {
                        device_address: unsafe {
                            mesh_buffer.index_buffer.get_buffer_address()
                        }
                    })
                    .index_type(IndexType::UINT32)
                    .build(),
            })
            //OPAQUEはany-hitシェーダを呼び出さない
            //.flags(GeometryFlagsKHR::OPAQUE)
            .build();

        let build_range_info = AccelerationStructureBuildRangeInfoKHR::builder()
            .primitive_count(mesh_buffer.indices_count / 3)
            .build();

        let geometries = [geometry];

        let build_info = AccelerationStructureBuildGeometryInfoKHR::builder()
            //ASのビルドよりもトレースの処理速度を優先する
            .flags(BuildAccelerationStructureFlagsKHR::PREFER_FAST_TRACE)
            .geometries(&geometries)
            .mode(BuildAccelerationStructureModeKHR::BUILD)
            .ty(AccelerationStructureTypeKHR::BOTTOM_LEVEL)
            .build();

        let memory_requirements = unsafe {
            acceleration_structure.get_acceleration_structure_build_sizes(
                AccelerationStructureBuildTypeKHR::DEVICE,
                &build_info,
                //geometriesに対応するように配列を作成する
                &[mesh_buffer.indices_count / 3]
            )
        };

        let scratch_buffer = Buffers::new(
            device,
            device_memory_properties,
            memory_requirements.build_scratch_size,
            BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | BufferUsageFlags::STORAGE_BUFFER,
            MemoryPropertyFlags::DEVICE_LOCAL,
        );

        let scratch_data = DeviceOrHostAddressKHR {
            device_address: unsafe {
                scratch_buffer.get_buffer_address()
            }
        };

        let bottom_accel_buffer = Buffers::new(
            device,
            device_memory_properties,
            memory_requirements.acceleration_structure_size,
            BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR
                | BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | BufferUsageFlags::STORAGE_BUFFER,
            MemoryPropertyFlags::DEVICE_LOCAL,
        );

        let bottom_accel_create_info = AccelerationStructureCreateInfoKHR::builder()
            .ty(AccelerationStructureTypeKHR::BOTTOM_LEVEL)
            .size(memory_requirements.acceleration_structure_size)
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
            device.free_command_buffers(commands.command_pool, &[build_cb]);

            device.destroy_buffer(scratch_buffer.buffer, None);
            device.free_memory(scratch_buffer.memory, None);
        }

        (bottom_accel, bottom_accel_buffer)
    }
}

impl Drop for TriangleAccelerationStructure<'_> {
    fn drop(&mut self) {
        todo!()
    }
}