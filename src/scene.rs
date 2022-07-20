use ash::Device;
use ash::vk::{AccelerationStructureInstanceKHR, AccelerationStructureReferenceKHR, Buffer, BufferCopy, BufferUsageFlags, DeviceAddress, DeviceSize, GeometryInstanceFlagsKHR, MemoryPropertyFlags, Packed24_8, PhysicalDeviceMemoryProperties, TransformMatrixKHR};
use crate::buffers::Buffers;
use crate::renderer::acceleration_structures::triangle_bottom_level_acceleration_structure::TriangleBottomLevelAccelerationStructure;

struct Scene<'a> {
    device: &'a Device,
    instances: Vec<AccelerationStructureInstanceKHR>,
    instance_buffer: Buffers<'a>,
}

impl<'a> Scene<'a> {
    pub fn build_scene(
        device: &'a Device,
        device_memory_properties: PhysicalDeviceMemoryProperties,
        //色々なモデルに対応したい場合はここを複数受け取れるように
        triangle_bottom_acceleration_structure_handle: DeviceAddress,
    ) -> Self {
        let size = 1.0;
        let pos_x = 0.0;
        let pos_y = 1.0;
        let pos_z = 0.0;

        //現在はインスタンス一つのみ対応
        let instance = Self::create_triangle_instance(
            triangle_bottom_acceleration_structure_handle,
            //TODO: SRT行列対応
            TransformMatrixKHR {
                matrix: [
                    size, 0.0, 0.0, pos_x,
                    0.0, size, 0.0, pos_y,
                    0.0, 0.0, size, pos_z
                ]
            }
        );

        let instances = vec![instance];

        let instance_buffer_size =
            std::mem::size_of::<AccelerationStructureInstanceKHR>() * instances.len();

        let mut instance_buffer = Buffers::new(
            device,
            device_memory_properties,
            instance_buffer_size as DeviceSize,
            BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR,
            MemoryPropertyFlags::HOST_VISIBLE
                | MemoryPropertyFlags::HOST_COHERENT
                | MemoryPropertyFlags::DEVICE_LOCAL
        );

        instance_buffer.store(&instances);

        Self {
            device,
            instances,
            instance_buffer,
        }
    }

    fn create_triangle_instance(
        handle: DeviceAddress,
        transform: TransformMatrixKHR,
    ) -> AccelerationStructureInstanceKHR {
        AccelerationStructureInstanceKHR {
            transform,
            //Packed24_8はu32で24bitの型を表現するもの
            //instance_custom_indexが24bit、maskが8
            //maskは他のinstanceと交差判定を行うかどうか
            instance_custom_index_and_mask: Packed24_8::new(0, 0xff),
            //instance_shader_binding_table_record_offsetが24bit、flagsが8bit
            instance_shader_binding_table_record_offset_and_flags: Packed24_8::new(
                0,
                GeometryInstanceFlagsKHR::FORCE_OPAQUE.as_raw() as u8,
            ),
            acceleration_structure_reference: AccelerationStructureReferenceKHR {
                device_handle: handle
            },
        }
    }
}
