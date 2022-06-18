use std::ffi::{c_void, CStr, CString};
use std::os::raw::c_char;
use std::collections::HashSet;
use ash::vk;
use ash::{Device, Entry, Instance};
use ash::extensions::ext::DebugUtils;
use ash::extensions::khr::{Surface, Win32Surface};
use ash::vk::{DebugUtilsMessengerCreateInfoEXT, DeviceCreateInfo, DeviceQueueCreateInfo, PhysicalDevice, PhysicalDeviceFeatures, PhysicalDeviceVulkanMemoryModelFeatures};
use log::info;
use tobj::LoadError::NormalParseError;
use crate::renderer::queue_family_indices::QueueFamilyIndices;
use crate::renderer::surfaces::Surfaces;
use crate::renderer::validation_layer::ValidationLayer;
use crate::window_handlers::WindowHandlers;

pub struct Backends {
    pub entry: Entry,
    pub instance: Instance,
    pub physical_device: PhysicalDevice,
    pub device: Device,
    pub surfaces: Option<Surfaces>,
}

impl Backends {
    //with surface
    pub fn new(window_handlers: &WindowHandlers , enable_validation_layer: bool) -> anyhow::Result<Self> {
        let entry = unsafe { Entry::load()? };
        let instance = Self::create_instance(&entry, enable_validation_layer)?;

        let surfaces = Surfaces::new(&instance, &entry, &window_handlers.window);

        let physical_device = Self::pick_physical_device(&instance, &surfaces,
        &[
            ash::extensions::khr::AccelerationStructure::name(),
            ash::extensions::khr::DeferredHostOperations::name(),
            ash::extensions::khr::RayTracingPipeline::name(),
        ]);


        let queue_family_indices = QueueFamilyIndices::new(&instance, Some(&surfaces), physical_device);


        let device = Self::create_logical_device(
            &instance,
            physical_device,
            queue_family_indices.graphics_family.expect("Graphics family does not exist.")
        );

        Ok(Self {
            entry,
            instance,
            physical_device,
            device,
            surfaces: Some(surfaces),
        })
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

        let mut debug_extensions_names = ValidationLayer::require_debug_utils_extension_names_ptr();

        if enable_validation_layer {
            extension_names.append(&mut debug_extensions_names);
        }

        let mut instance_create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&extension_names);

        let validation_extension_names = ValidationLayer::require_validation_layer_extension_names();
        let validation_extension_names_ptr = validation_extension_names.iter().map(|name| name.as_ptr()).collect::<Vec<_>>();

        let debug_create_info = ValidationLayer::populate_debug_messenger_create_info();

        if enable_validation_layer {
            ValidationLayer::check_validation_layer_support(entry);

            instance_create_info = instance_create_info.enabled_layer_names(&validation_extension_names_ptr);

            instance_create_info.p_next = &debug_create_info as *const DebugUtilsMessengerCreateInfoEXT as *const c_void;
        }

        unsafe { Ok(entry.create_instance(&instance_create_info, None)?) }
    }

    fn pick_physical_device(
        instance: &Instance,
        //with surface
        surfaces: &Surfaces,
        extensions: &[&CStr],
    ) -> PhysicalDevice {
        let physical_devices = unsafe {
            instance
                .enumerate_physical_devices()
                .expect("You could not be retrieved a physical device")
        };
        
        let physical_device = physical_devices
            .into_iter()
            .find_map(|physical_device| {
                let indices = QueueFamilyIndices::new(instance, Some(surfaces), physical_device);

                if unsafe { instance.enumerate_device_extension_properties(physical_device) }.map(
                    |exts| {
                        let set: std::collections::HashSet<&CStr> = exts.iter()
                            .map(|ext| unsafe { CStr::from_ptr(&ext.extension_name as * const c_char) })
                            .collect();

                        extensions.iter().all(|ext| set.contains(ext))
                    }
                ) != Ok(true) {
                    return None;
                }

                //with surface
                if !indices.is_device_suitable(instance, physical_device, surfaces) {
                    return None;
                }

                Some(physical_device)
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

        let validation_extension_names = ValidationLayer::require_validation_layer_extension_names();
        let validation_extension_names_ptr = validation_extension_names.iter().map(|name| name.as_ptr()).collect::<Vec<_>>();

        let device_create_info = DeviceCreateInfo::builder()
            .push_next(&mut vulkan_memory_model_features)
            .queue_create_infos(&queue_create_info)
            .enabled_layer_names(validation_extension_names_ptr.as_slice())
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