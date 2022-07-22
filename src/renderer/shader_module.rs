use std::io::Bytes;
use std::ptr;
use ash::Device;
use ash::vk::{ShaderModule, ShaderModuleCreateFlags, ShaderModuleCreateInfo, StructureType};

pub struct ShaderModules<'a> {
    device: &'a Device,
    pub shader_module: ShaderModule,
}

impl<'a> ShaderModules<'a> {
    pub fn new(
        device: &'a Device,
        code: &[u8]
    ) -> Self {
        let shader_module_create_info = ShaderModuleCreateInfo {
            code_size: code.len(),
            p_code: code.as_ptr() as *const u32,
            ..Default::default()
        };

        let shader_module = unsafe {
            device.create_shader_module(&shader_module_create_info, None).unwrap()
        };

        Self {
            device,
            shader_module,
        }
    }
}

impl Drop for ShaderModules<'_> {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_shader_module(self.shader_module, None);
        }
    }
}
