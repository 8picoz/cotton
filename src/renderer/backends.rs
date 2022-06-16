use std::ffi::{CStr, CString};
use ash::vk;
use ash::{Device, Entry, Instance};
use ash::extensions::ext::DebugUtils;
use ash::extensions::khr::{Surface, Win32Surface};
use ash::vk::{DeviceCreateInfo, DeviceQueueCreateInfo, PhysicalDevice, PhysicalDeviceFeatures, PhysicalDeviceVulkanMemoryModelFeatures};
use log::info;
use tobj::LoadError::NormalParseError;
use crate::renderer::queue_family_indices::QueueFamilyIndices;
use crate::renderer::surfaces::Surfaces;
use crate::renderer::validation_layer::ValidationLayer;

pub struct Backends {
    pub entry: Entry,
    pub instance: Instance,
    pub physical_device: PhysicalDevice,
    pub device: Device,
}

impl Backends {
    //with surface
    pub fn new(surfaces: &Surfaces, enable_validation_layer: bool) -> anyhow::Result<Self> {
        let entry = unsafe { Entry::load()? };
        let instance = Self::create_instance(&entry, enable_validation_layer)?;

        let physical_device = Self::pick_physical_device(&instance, surfaces);

        let queue_family_indices = QueueFamilyIndices::new(&instance, Some(surfaces), physical_device);

        let device = Self::create_logical_device(
            &instance,
            physical_device,
            queue_family_indices.graphics_family.expect("Graphics family does not exist.")
        );
    }

    fn create_instance(entry: &Entry, enable_validation_layer: bool) -> anyhow::Result<Instance> {
        let app_info = vk::ApplicationInfo::builder()
            .application_name(CString::new(crate::constants::APPLICATION_NAME)?.as_c_str())
            .application_version(0)
            .engine_name(CString::new("No Engine")?.as_c_str())
            .engine_version(0)
            .api_version(vk::make_api_version(0, 1, 3, 0))
            .build();

        let mut extension_names = Surfaces::require_surface_extension_names();

        if enable_validation_layer {
            extension_names.append(&mut ValidationLayer::require_debug_utils_extension_names_ptr());
        }

        let mut instance_create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&extension_names);

        if enable_validation_layer {
            ValidationLayer::check_validation_layer_support(entry);

            let debug_create_info = ValidationLayer::populate_debug_messenger_create_info();

            instance_create_info = instance_create_info.enabled_layer_names(&ValidationLayer::require_validation_layer_extension_names_ptr());

            instance_create_info.p_next = &debug_create_info as *const _ as *const _;
        }

        unsafe { Ok(entry.create_instance(&instance_create_info, None)?) }
    }

    fn pick_physical_device(
        instance: &Instance,
        surfaces: &Surfaces,
    ) -> PhysicalDevice {
        let physical_devices = unsafe {
            instance
                .enumerate_physical_devices()
                .expect("You could not be retrieved a physical device")
        };
        
        let physical_device = physical_devices
            .into_iter()
            .find(|physical_device| {
                let indices = QueueFamilyIndices::new(instance, Some(surfaces), *physical_device);
                indices.is_device_suitable(instance, surfaces)
            })
            .expect("Doesn't match physical device suitable");

        let props = unsafe {
            instance.get_physical_device_properties((physical_device))
        };

        info!("Selected physical device: {:?}", unsafe {
            CStr::from_ptr(props.device_name.as_ptr())
        });

        physical_device
    }

    fn create_logical_device(
        instance: &Instance,
        physical_device: PhysicalDevice,
        queue_family_index: u32,
    ) -> Device {
        let queue_create_info = [DeviceQueueCreateInfo::builder()
            .queue_family_index(queue_family_index)
            .queue_priorities(&[1.0f32])
            .build()
        ];

        let mut vulkan_memory_model_features =
            PhysicalDeviceVulkanMemoryModelFeatures::builder()
                .vulkan_memory_model(true)
                .build();

        let device_create_info = DeviceCreateInfo::builder()
            .push_next(&mut vulkan_memory_model_features)
            .queue_create_infos(&queue_create_info)
            .enabled_layer_names(ValidationLayer::require_validation_layer_extension_names_ptr().as_slice())
            .build();

        unsafe { instance.create_device(physical_device, &device_create_info, None).expect("Failed to create logical Device") }
    }
}

impl Drop for Backends {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_device(None);

            self.instance.destroy_instance(None);
        }
    }
}