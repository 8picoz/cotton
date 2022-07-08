use ash::Device;
use ash::vk::{Buffer, BufferCreateInfo, BufferUsageFlags, DeviceMemory, DeviceSize, MemoryAllocateFlags, MemoryAllocateFlagsInfo, MemoryAllocateInfo, MemoryMapFlags, MemoryPropertyFlags, PhysicalDeviceMemoryProperties, SharingMode};

pub struct Buffers<'a> {
    pub device: &'a Device,
    pub raw: Buffer,
    pub size: DeviceSize,
    pub memory: DeviceMemory,
}

impl<'a> Buffers<'a> {
    pub fn new(
        size: DeviceSize,
        usage: BufferUsageFlags,
        memory_properties: MemoryPropertyFlags,
        device: &'a Device,
        device_memory_properties: PhysicalDeviceMemoryProperties,
    ) -> Self {
        let buffer_info = BufferCreateInfo::builder()
            .size(size)
            .usage(usage)
            .sharing_mode(SharingMode::EXCLUSIVE)
            .build();

        let raw = unsafe {
            device.create_buffer(&buffer_info, None).unwrap()
        };

        //メモリサイズやアライメントなどの確保に必要な情報を持つ構造体
        let memory_requirements = unsafe {
            device.get_buffer_memory_requirements(raw)
        };

        let mut memory_type_index = 0;

        for i in 0..device_memory_properties.memory_type_count {
            if (type_bits & 1) == 1 {
                //PhysicalDeviceMemoryPropertiesの中からmemory_propertiesと同一のものを探す
                if (device_memory_properties.memory_types[i as usize].property_flags & memory_properties) == memory_properties {
                    memory_type_index = i;
                }
            }
        }

        let mut memory_allocate_flags_info = MemoryAllocateFlagsInfo::builder()
            //SHADER_DEVICE_ADDRESSの指定とvkGetDeviceMemoryOpaqueCaptureAddressでアドレスを取得できるようになる
            .flags(MemoryAllocateFlags::DEVICE_ADDRESS)
            .build();


        let mut allocate_info = MemoryAllocateInfo::builder();

        //SHADER_DEVICE_ADDRESSはvkGetBufferDeviceAddressからバッファのデバイスアドレスを取得することができ、それを使用することでシェーダー内からアクセスすることが出来る
        if usage.contains(BufferUsageFlags::SHADER_DEVICE_ADDRESS) {
            allocate_info = allocate_info.push_next(&mut memory_allocate_flags_info);
        }

        let allocate_info = allocate_info
            .allocation_size(memory_requirements.size)
            .memory_type_index(memory_type_index)
            .build();

        let memory = unsafe {
            device.allocate_memory(&allocate_info, None).unwrap()
        };

        unsafe {
            device.bind_buffer_memory(raw, memory, 0).unwrap()
        }

        Self {
            device,
            raw,
            size,
            memory,
        }
    }

    fn map(&mut self, size: DeviceSize) -> *mut std::ffi::c_void {
        unsafe {
            self.device
                .map_memory(self.memory, 0, size, MemoryMapFlags::empty())
                .unwrap()
        }
    }

    fn unmap(&self) {
        unsafe {
            self.device.unmap_memory(self.memory)
        }
    }
}

impl Drop for Buffers<'_> {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_buffer(self.raw, None);
            self.device.free_memory(self.memory, None);
        }
    }
}