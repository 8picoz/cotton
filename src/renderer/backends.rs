use std::ffi::{c_void, CStr, CString};
use std::os::raw::c_char;
use std::collections::HashSet;
use std::default::default;
use ash::vk;
use ash::{Device, Entry, Instance};
use ash::extensions::ext::DebugUtils;
use ash::extensions::khr::{AccelerationStructure, DeferredHostOperations, RayTracingPipeline, Surface, Swapchain, Win32Surface};
use ash::vk::{DebugUtilsMessengerCreateInfoEXT, DeviceCreateInfo, DeviceQueueCreateInfo, ExtScalarBlockLayoutFn, KhrGetMemoryRequirements2Fn, KhrSpirv14Fn, PhysicalDevice, PhysicalDeviceAccelerationStructureFeaturesKHR, PhysicalDeviceFeatures, PhysicalDeviceFeatures2, PhysicalDeviceRayTracingPipelineFeaturesKHR, PhysicalDeviceVulkan12Features, PhysicalDeviceVulkanMemoryModelFeatures, Queue};
use log::info;
use tobj::LoadError::NormalParseError;
use crate::renderer::queue_family_indices::QueueFamilyIndices;
use crate::renderer::surfaces::Surfaces;
use crate::renderer::validation_layer::{REQUIRED_LAYERS, ValidationLayer};
use crate::window_handlers::WindowHandlers;

pub struct Backends {
    pub entry: Entry,
    pub instance: Instance,
    pub physical_device: PhysicalDevice,
    pub device: Device,
    pub surfaces: Option<Surfaces>,
    queue_family_indices: QueueFamilyIndices,
}

impl Backends {
    //with surface
    pub fn new(window_handlers: &WindowHandlers , enable_validation_layer: bool) -> anyhow::Result<Self> {
        let entry = unsafe { Entry::load()? };
        let instance = Self::create_instance(&entry, enable_validation_layer)?;

        let surfaces = Surfaces::new(&instance, &entry, &window_handlers.window);

        let physical_device = Self::pick_physical_device(&instance, &surfaces,
        &[
            Swapchain::name(),
            AccelerationStructure::name(),
            DeferredHostOperations::name(),
            RayTracingPipeline::name(),
        ]);

        let queue_family_indices = QueueFamilyIndices::new(&instance, Some(&surfaces), physical_device);

        let device = Self::create_logical_device(
            &instance,
            physical_device,
            &queue_family_indices,
            enable_validation_layer,
        );

        Ok(Self {
            entry,
            instance,
            physical_device,
            device,
            surfaces: Some(surfaces),
            queue_family_indices,
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

        let mut debug_extensions_names = ValidationLayer::require_debug_utils_extension_names_c_char();

        if enable_validation_layer {
            extension_names.append(&mut debug_extensions_names);
        }

        let mut instance_create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&extension_names);

        let validation_extension_names = ValidationLayer::require_validation_layer_extension_names_cstring();
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

    //with surface
    fn create_logical_device(
        instance: &Instance,
        physical_device: PhysicalDevice,
        queue_family_indices: &QueueFamilyIndices,
        enable_validation_layer: bool,
    ) -> Device {
        //with surface
        let queue_create_info = [
            DeviceQueueCreateInfo::builder()
                .queue_family_index(queue_family_indices.graphics_family.unwrap())
                .queue_priorities(&[1.0f32])
                .build(),
            DeviceQueueCreateInfo::builder()
                .queue_family_index(queue_family_indices.present_family.unwrap())
                .queue_priorities(&[1.0f32])
                .build()
        ];

        let mut features2 = PhysicalDeviceFeatures2::default();
        unsafe { instance.get_physical_device_features2(physical_device, &mut features2) };

        let mut features12 = PhysicalDeviceVulkan12Features::builder()
            //8bitのsigned intがサポートされているかどうか
            .shader_int8(true)
            //vkGetBufferDeviceAddressを使用して取得したアドレスを返してStorageBufferに対して悪世することが出来るかをサポートしているかどうか
            .buffer_device_address(true)
            //Vulkan専用のメモリモデルがサポートされているかどうか
            .vulkan_memory_model(true)
            .build();

        let mut as_feature = PhysicalDeviceAccelerationStructureFeaturesKHR::builder()
            .acceleration_structure(true)
            .build();

        let mut raytracing_pipeline = PhysicalDeviceRayTracingPipelineFeaturesKHR::builder()
            .ray_tracing_pipeline(true)
            .build();

        let mut extension_names = vec![
            Swapchain::name().as_ptr(),
            RayTracingPipeline::name().as_ptr(),
            AccelerationStructure::name().as_ptr(),
            //非同期処理をするに当たってのlockを提供する(?)
            DeferredHostOperations::name().as_ptr(),
            //Vulkan1.2時代はSpir-vの1.4がサポートされているかは環境次第だった？
            //KhrSpirv14Fn::name().as_ptr(),

            //uniform bufferやらstorage bufferなどにC言語ライクな構造体を対応させることが出来る
            //非スカラー型などに対応できる
            ExtScalarBlockLayoutFn::name().as_ptr(),
            //VkMemoryRequirementsやVkSparseImageMemoryRequirements構造体に対してsTypeやpNextを生やすようにする
            KhrGetMemoryRequirements2Fn::name().as_ptr(),
        ];
        
        let mut device_create_info = DeviceCreateInfo::builder()
            .push_next(&mut features2)
            .push_next(&mut features12)
            .push_next(&mut as_feature)
            .push_next(&mut raytracing_pipeline)
            .enabled_extension_names(&extension_names)
            .queue_create_infos(&queue_create_info);

        if enable_validation_layer {
            let mut validation_extension_names = ValidationLayer::require_validation_layer_extension_names_c_char();

            device_create_info
                .enabled_layer_names(validation_extension_names.as_slice());
        }

        let device_create_info = device_create_info.build();

        unsafe {
            instance.create_device(physical_device, &device_create_info, None)
                .expect("Failed to create logical Device")
        }
    }

    pub fn create_graphics_queue(
        &self,
        queue_index: u32,
    ) -> Queue {
        unsafe { self.device.get_device_queue(
            self.queue_family_indices.graphics_family.expect("Failed to create graphics family queue"),
                            queue_index
        )}
    }

    pub fn create_present_queue(
        &self,
        queue_index: u32,
    ) -> Queue {
        unsafe { self.device.get_device_queue(
            self.queue_family_indices.present_family.expect("Failed to create graphics family queue"),
            queue_index
        )}
    }
}

impl Drop for Backends {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_device(None);

            if let Some(surfaces) = self.surfaces.as_ref() {
                surfaces.surface.destroy_surface(surfaces.surface_khr, None);
            }

            self.instance.destroy_instance(None);
        }
    }
}