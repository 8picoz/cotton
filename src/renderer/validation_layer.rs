use std::ffi::{c_char, c_void, CStr, CString};
use ash::{Entry, Instance, vk};
use ash::extensions::ext::DebugUtils;
use ash::prelude::VkResult;
use ash::vk::{DebugUtilsMessageSeverityFlagsEXT, DebugUtilsMessageTypeFlagsEXT, DebugUtilsMessengerCreateInfoEXT, DebugUtilsMessengerEXT};
use log::{debug, info};
use tobj::LoadError::NormalParseError;

pub const REQUIRED_LAYERS: [&str; 1] = ["VK_LAYER_KHRONOS_validation"];

pub struct ValidationLayer {
    debug_utils: DebugUtils,
    debug_utils_messenger_ext: DebugUtilsMessengerEXT,
}

impl ValidationLayer {
    pub fn require_debug_utils_extension_names_c_char() -> Vec<*const c_char> {
        vec![DebugUtils::name().as_ptr()]
    }

    pub fn require_validation_layer_extension_names_cstring() -> Vec<CString> {
        REQUIRED_LAYERS.map(|item| CString::new(item)).into_iter().collect()
    }

    pub fn require_validation_layer_extension_names_c_char() -> Vec<*const c_char> {
        Self::require_validation_layer_extension_names_cstring()
            .iter()
            .map(|item| item.as_ptr())
            .into_iter()
            .collect()
    }

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

    pub fn check_validation_layer_support(entry: &Entry) {
        for required in Self::require_validation_layer_extension_names_cstring() {
            let found = entry
                .enumerate_instance_layer_properties()
                .unwrap()
                .iter()
                .any(|layer| {
                    let name = unsafe { CStr::from_ptr(layer.layer_name.as_ptr()) };
                    //let name = name.to_str().expect("Failed to get layer name pointer");
                    required.as_c_str() == name
                });

            if !found {
                panic!("Validation layer not supported: {:?}", required);
            }
        }

        info!("Validation layer supported");
    }

    pub fn populate_debug_messenger_create_info() -> DebugUtilsMessengerCreateInfoEXT {
        DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(
                DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                    | DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | DebugUtilsMessageSeverityFlagsEXT::ERROR,
            )
            .message_type(
                DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
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
    message_severity: DebugUtilsMessageSeverityFlagsEXT,
    message_type: DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut c_void,
) -> vk::Bool32 {
    let severity = match message_severity {
        DebugUtilsMessageSeverityFlagsEXT::VERBOSE => "VERBOSE",
        DebugUtilsMessageSeverityFlagsEXT::WARNING => "WARNING",
        DebugUtilsMessageSeverityFlagsEXT::ERROR => "ERROR",
        DebugUtilsMessageSeverityFlagsEXT::INFO => "INFO",
        _ => "???",
    };

    let types = match message_type {
        DebugUtilsMessageTypeFlagsEXT::GENERAL => "GENERAL",
        DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => "PERFORMANCE",
        DebugUtilsMessageTypeFlagsEXT::VALIDATION => "VALIDATION",
        _ => "???",
    };

    let data = *p_callback_data;
    let message = CStr::from_ptr(data.p_message);

    debug!("validation layer: [{}, {}] {:?}", severity, types, message);

    vk::FALSE
}