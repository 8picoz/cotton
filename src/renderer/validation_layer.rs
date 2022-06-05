use std::ffi::{c_void, CStr, CString};
use ash::{Entry, Instance, vk};
use ash::extensions::ext::DebugUtils;
use ash::prelude::VkResult;
use ash::vk::{DebugUtilsMessengerCreateInfoEXT, DebugUtilsMessengerEXT};
use log::debug;
use tobj::LoadError::NormalParseError;

pub const REQUIRED_LAYERS: [&str; 1] = ["VK_LAYER_KHRONOS_validation"];

pub struct ValidationLayer {
    debug_utils: DebugUtils,
    debug_utils_messenger_ext: DebugUtilsMessengerEXT,
}

impl ValidationLayer {
    pub fn new_default(entry: &Entry, instance: &Instance) -> VkResult<Self> {
        debug!("Enable validation layer");

        let debug_utils = DebugUtils::new(entry, instance);

        let create_info = Self::populate_debug_messenger_create_info();

        let debug_utils_messenger_ext = unsafe { debug_utils.create_debug_utils_messenger(&create_info, None)? };

        Ok(Self{
            debug_utils,
            debug_utils_messenger_ext,
        })
    }

    fn populate_debug_messenger_create_info() -> DebugUtilsMessengerCreateInfoEXT {
        DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                    | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
            )
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            )
            .pfn_user_callback(Some(vulkan_debug_callback))
            .build()
    }
}

impl Drop for ValidationLayer {
    fn drop(&mut self) {
        debug!("Dropping validation layer");

        unsafe {
            self.debug_utils.destroy_debug_utils_messenger(self.debug_utils_messenger_ext, None);
        }
    }
}

unsafe extern "system" fn vulkan_debug_callback(
    _message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    _message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut c_void,
) -> vk::Bool32 {
    let data = *p_callback_data;
    let message = CStr::from_ptr(data.p_message).to_string_lossy();

    debug!("validation layer: {:?}", message);

    vk::FALSE
}