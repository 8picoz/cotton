use std::io::Bytes;
use std::ptr;
use ash::Device;
use ash::vk::{ShaderModule, ShaderModuleCreateFlags, ShaderModuleCreateInfo, StructureType};

pub fn create_shader_module(
    device: &Device,
    code: &[u8]
) -> ShaderModule {
    let shader_module_create_info = ShaderModuleCreateInfo {
        code_size: code.len(),
        p_code: code.as_ptr() as *const u32,
        ..Default::default()
    };

    unsafe {
        device.create_shader_module(&shader_module_create_info, None)
    }.unwrap()
}
