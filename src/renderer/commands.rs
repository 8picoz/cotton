use ash::Device;
use ash::vk::{CommandBuffer, CommandBufferAllocateInfo, CommandBufferLevel, CommandPool, CommandPoolCreateFlags, CommandPoolCreateInfo};
use crate::constants::MAX_FRAMES_IN_FLIGHT;

struct Commands<'a> {
    device: &'a Device,
    pub command_pool: CommandPool,
    pub command_buffers: Vec<CommandBuffer>,
}

impl<'a> Commands<'a> {
    pub fn new(device: &'a Device, queue_family_index: u32) -> Self {
        let command_pool = Self::create_command_pool(device, queue_family_index);

        let command_buffers = Self::create_command_buffers(device, command_pool, MAX_FRAMES_IN_FLIGHT);

        Self {
            device,
            command_pool,
            command_buffers,
        }
    }

    fn create_command_pool(device: &Device, queue_family_index: u32) -> CommandPool {
        let command_pool_create_info = CommandPoolCreateInfo::builder()
            .queue_family_index(queue_family_index)
            .flags(CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .build();

        unsafe {
            device.create_command_pool(&command_pool_create_info, None).unwrap()
        }
    }

    fn create_command_buffers(device: &Device, command_pool: CommandPool, size: u32) -> Vec<CommandBuffer> {
        let command_buffer_alloc_info = CommandBufferAllocateInfo::builder()
            .command_pool(command_pool)
            .level(CommandBufferLevel::PRIMARY)
            .command_buffer_count(size)
            .build();

        unsafe {
            device.allocate_command_buffers(&command_buffer_alloc_info).unwrap()
        }
    }
}