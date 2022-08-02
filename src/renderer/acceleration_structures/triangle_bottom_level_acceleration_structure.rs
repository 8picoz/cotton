use ash::Device;
use ash::extensions::khr::AccelerationStructure;
use ash::vk::{AabbPositionsKHR, AccelerationStructureBuildGeometryInfoKHR, AccelerationStructureBuildRangeInfoKHR, AccelerationStructureBuildTypeKHR, AccelerationStructureCreateInfoKHR, AccelerationStructureDeviceAddressInfoKHR, AccelerationStructureGeometryAabbsDataKHR, AccelerationStructureGeometryDataKHR, AccelerationStructureGeometryKHR, AccelerationStructureGeometryTrianglesDataKHR, AccelerationStructureInstanceKHR, AccelerationStructureKHR, AccelerationStructureTypeKHR, Buffer, BufferUsageFlags, BuildAccelerationStructureFlagsKHR, BuildAccelerationStructureModeKHR, CommandBuffer, CommandBufferBeginInfo, CommandBufferUsageFlags, CommandPool, DeviceAddress, DeviceOrHostAddressConstKHR, DeviceOrHostAddressKHR, DeviceSize, Fence, GeometryFlagsKHR, GeometryTypeKHR, IndexType, MemoryPropertyFlags, PhysicalDeviceMemoryProperties, Queue, SubmitInfo};
use glam::{const_vec3a, vec3a, Vec3A};
use log::debug;
use crate::buffers::Buffers;
use crate::renderer::backends::Backends;
use classical_raytracer_shader::Vertex;
use crate::renderer::mesh_buffer::MeshBuffer;

//instanceを作って

//TLASを作成する

pub struct TriangleBottomLevelAccelerationStructure<'a> {
    backends: &'a Backends,
    pub acceleration_structure: &'a AccelerationStructure,
    pub bottom_acceleration_structure: AccelerationStructureKHR,
    pub bottom_acceleration_buffer: Buffers<'a>,
    pub mesh_buffer: MeshBuffer<'a>,
}

impl<'a> TriangleBottomLevelAccelerationStructure<'a> {
    pub fn new(
        backends: &'a Backends,
        acceleration_structure: &'a AccelerationStructure,
        graphics_queue: Queue,
    ) -> Self {

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

        let mesh_buffer = MeshBuffer::new(&backends.device, vertices, indices, backends.device_memory_properties);

        //TODO: このbottom asをモデルごとに作成するようにしてtop asと紐づける
        let (
            bottom_acceleration_structure,
            bottom_acceleration_buffer
        ) = Self::create_bottom_acceleration(
            &backends,
            backends.device_memory_properties,
            &acceleration_structure,
            &mesh_buffer,
            graphics_queue,
        );

        Self {
            backends,
            acceleration_structure,
            bottom_acceleration_structure,
            bottom_acceleration_buffer,
            mesh_buffer,
        }
    }

    fn create_bottom_acceleration(
        backends: &'a Backends,
        device_memory_properties: PhysicalDeviceMemoryProperties,
        acceleration_structure: &AccelerationStructure,
        mesh_buffer: &MeshBuffer,
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
            &backends.device,
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
            &backends.device,
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

        let command_pool = backends.create_graphics_command_pool();
        let command_buffers = backends.create_command_buffers(command_pool, 1);
        let build_cb = command_buffers[0];

        unsafe {
            backends.device.begin_command_buffer(
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
            backends.device.end_command_buffer(build_cb).unwrap();

            backends.device.queue_submit(
                graphics_queue,
                &[SubmitInfo::builder()
                    .command_buffers(&[build_cb])
                    .build()
                ],
                Fence::null(),
            ).expect("submit failed");

            //Queueの処理が終わるまで待機
            backends.device.queue_wait_idle(graphics_queue).unwrap();
            backends.device.free_command_buffers(command_pool, &command_buffers);
            backends.device.destroy_command_pool(command_pool, None);

            backends.device.destroy_buffer(scratch_buffer.buffer, None);
            backends.device.free_memory(scratch_buffer.memory, None);
        }

        (bottom_accel, bottom_accel_buffer)
    }

    pub fn get_device_address_info(&self) -> DeviceAddress {
        let address_info = AccelerationStructureDeviceAddressInfoKHR::builder()
            .acceleration_structure(self.bottom_acceleration_structure)
            .build();

        unsafe {
            self.acceleration_structure.get_acceleration_structure_device_address(&address_info)
        }
    }

}

impl Drop for TriangleBottomLevelAccelerationStructure<'_> {
    fn drop(&mut self) {
        //todo!()
    }
}