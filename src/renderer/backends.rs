use std::ffi::{c_void, CStr, CString};
use std::os::raw::c_char;
use std::collections::HashSet;
use ash::vk;
use ash::{Device, Entry, Instance};
use ash::extensions::ext::DebugUtils;
use ash::extensions::khr::{AccelerationStructure, DeferredHostOperations, RayTracingPipeline, Surface, Swapchain, Win32Surface};
use ash::vk::{CommandPool, CommandPoolCreateFlags, CommandPoolCreateInfo, DebugUtilsMessengerCreateInfoEXT, DeviceCreateInfo, DeviceQueueCreateInfo, ExtScalarBlockLayoutFn, KhrGetMemoryRequirements2Fn, KhrSpirv14Fn, PhysicalDevice, PhysicalDeviceAccelerationStructureFeaturesKHR, PhysicalDeviceBufferDeviceAddressFeatures, PhysicalDeviceDescriptorIndexingFeaturesEXT, PhysicalDeviceFeatures, PhysicalDeviceFeatures2, PhysicalDeviceImagelessFramebufferFeaturesKHR, PhysicalDeviceMemoryProperties, PhysicalDeviceProperties2, PhysicalDeviceRayTracingPipelineFeaturesKHR, PhysicalDeviceRayTracingPipelinePropertiesKHR, PhysicalDeviceScalarBlockLayoutFeaturesEXT, PhysicalDeviceShaderFloat16Int8Features, PhysicalDeviceVulkan12Features, PhysicalDeviceVulkanMemoryModelFeatures, PhysicalDeviceVulkanMemoryModelFeaturesKHR, Queue};
use log::{debug, info};
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
    pub command_pool: CommandPool,

    pub(crate) device_memory_properties: PhysicalDeviceMemoryProperties,
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

        let device_memory_properties = unsafe {
            instance.get_physical_device_memory_properties(physical_device)
        };

        let command_pool = Self::create_command_pool(
            &device,
            queue_family_indices.graphics_family.unwrap()
        );

        Ok(Self {
            entry,
            instance,
            physical_device,
            device,
            surfaces: Some(surfaces),
            command_pool,
            device_memory_properties,
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

        let mut scalar_block = PhysicalDeviceScalarBlockLayoutFeaturesEXT::default();
        //ディスクリプタ配列を扱えるように
        let mut descriptor_indexing = PhysicalDeviceDescriptorIndexingFeaturesEXT::default();
        let mut imageless_framebuffer = PhysicalDeviceImagelessFramebufferFeaturesKHR::default();
        let mut shader_float16_int8 = PhysicalDeviceShaderFloat16Int8Features::default();
        let mut vulkan_memory_model = PhysicalDeviceVulkanMemoryModelFeaturesKHR::default();
        //バッファにおいてデバイスアドレスを使用可能に
        let mut get_buffer_device_address_features = PhysicalDeviceBufferDeviceAddressFeatures::default();

        //Raytracing
        let mut acceleration_structure_features = PhysicalDeviceAccelerationStructureFeaturesKHR::default();
        let mut ray_tracing_pipeline_features = PhysicalDeviceRayTracingPipelineFeaturesKHR::default();

        let mut features2 = PhysicalDeviceFeatures2::builder()
            .push_next(&mut scalar_block)
            .push_next(&mut descriptor_indexing)
            .push_next(&mut imageless_framebuffer)
            .push_next(&mut shader_float16_int8)
            .push_next(&mut vulkan_memory_model)
            .push_next(&mut get_buffer_device_address_features)
            .push_next(&mut acceleration_structure_features)
            .push_next(&mut ray_tracing_pipeline_features)
            .build();

        unsafe { instance.get_physical_device_features2(physical_device, &mut features2) };

        debug!("{:#?}", &scalar_block);
        debug!("{:#?}", &descriptor_indexing);
        debug!("{:#?}", &imageless_framebuffer);
        debug!("{:#?}", &shader_float16_int8);
        debug!("{:#?}", &vulkan_memory_model);
        debug!("{:#?}", &get_buffer_device_address_features);
        debug!("{:#?}", &acceleration_structure_features);
        debug!("{:#?}", &ray_tracing_pipeline_features);

        /*
        //PhysicalDeviceVulkan12Featuresは上部のPhysicalDeviceFeatures2のpNextに繋げばその機能がサポートされてるかの確認として使うことが出来るし、
        //DeviceCreateInfoに繋げばその機能の有効化にも使える
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
        */

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
            /*
            .push_next(&mut features12)
            .push_next(&mut as_feature)
            .push_next(&mut raytracing_pipeline)
            */
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

    fn create_command_pool(device: &Device, graphics_family_index: u32) -> CommandPool {
        let command_pool_create_info = CommandPoolCreateInfo::builder()
            .queue_family_index(graphics_family_index)
            .flags(CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .build();

        unsafe {
            device.create_command_pool(&command_pool_create_info, None).unwrap()
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

    pub fn display_support_extension(&self) {
        unsafe {
            let extension_properties = self.physical_device
                .instance
                .raw
                .enumerate_device_extension_properties(pdevice.raw)?;
            debug!("Extension properties:\n{:#?}", &extension_properties);

            let supported_extensions: HashSet<String> = extension_properties
                .iter()
                .map(|ext| {
                    std::ffi::CStr::from_ptr(ext.extension_name.as_ptr() as *const c_char)
                        .to_string_lossy()
                        .as_ref()
                        .to_owned()
                })
                .collect();

            for &ext in &device_extension_names {
                let ext = std::ffi::CStr::from_ptr(ext).to_string_lossy();
                if !supported_extensions.contains(ext.as_ref()) {
                    panic!("Device extension not supported: {}", ext);
                }
            }
        }
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