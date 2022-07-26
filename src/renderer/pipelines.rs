use std::ffi::{CStr, CString};
use ash::{Device, Instance, vk};
use ash::extensions::khr::{AccelerationStructure, RayTracingPipeline};
use ash::vk::{AccelerationStructureNV, DeferredOperationKHR, DescriptorBufferInfo, DescriptorImageInfo, DescriptorPoolCreateInfo, DescriptorPoolSize, DescriptorSetAllocateInfo, DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorSetLayoutCreateInfo, DescriptorSetVariableDescriptorCountAllocateInfo, DescriptorType, Extent2D, ImageLayout, ImageView, PhysicalDevice, PhysicalDeviceProperties2, PhysicalDeviceRayTracingPipelinePropertiesKHR, Pipeline, PipelineCache, PipelineLayout, PipelineLayoutCreateInfo, PipelineShaderStageCreateInfo, PushConstantRange, Queue, RayTracingPipelineCreateInfoKHR, RayTracingShaderGroupCreateInfoKHR, RayTracingShaderGroupTypeKHR, SHADER_UNUSED_KHR, ShaderModule, ShaderStageFlags, WHOLE_SIZE, WriteDescriptorSet, WriteDescriptorSetAccelerationStructureKHR};
use bytes::Buf;
use log::debug;
use crate::buffers::Buffers;
use crate::constants::{FRAGMENT_SHADER_ENTRY_NAME, MISS_SHADER_ENTRY_NAME, MISS_SHADER_ENTRY_NAME_BYTE, RAY_GENERATION_SHADER_ENTRY_NAME, RAY_GENERATION_SHADER_ENTRY_NAME_BYTE, SPHERE_CLOSEST_HIT_SHADER_ENTRY_NAME, SPHERE_CLOSEST_HIT_SHADER_ENTRY_NAME_BYTE, SPHERE_INTERSECTION_SHADER_ENTRY_NAME, SPHERE_INTERSECTION_SHADER_ENTRY_NAME_BYTE, TRIANGLE_ANY_HIT_SHADER_ENTRY_NAME, TRIANGLE_ANY_HIT_SHADER_ENTRY_NAME_BYTE, TRIANGLE_CLOSEST_HIT_SHADER_ENTRY_NAME, TRIANGLE_CLOSEST_HIT_SHADER_ENTRY_NAME_BYTE, VERTEX_SHADER_ENTRY_NAME};
use crate::renderer::acceleration_structures::AccelerationStructures;
use crate::renderer::acceleration_structures::top_level_acceleration_structures::TopLevelAccelerationStructures;
use crate::renderer::acceleration_structures::triangle_bottom_level_acceleration_structure::TriangleBottomLevelAccelerationStructure;
use crate::renderer::backends::Backends;
use crate::renderer::mesh_buffer::MeshBuffer;
use crate::renderer::render_passes::RenderPasses;
use crate::renderer::shader_module::ShaderModules;

pub struct Pipelines<'a> {
    pub device: &'a Device,
    pub pipeline: Pipeline,

    pub(crate) ray_tracing_pipeline: RayTracingPipeline,
    pub(crate) ray_tracing_pipeline_properties: PhysicalDeviceRayTracingPipelinePropertiesKHR,
}

impl<'a> Pipelines<'a> {
    //with raytracing
    pub fn new(
        backends: &'a Backends,
        shader_modules: ShaderModules,
        swapchain_extent: Extent2D,
        render_passes: &RenderPasses,

        //asとvertexとindexをまとめたほうが良い
        mesh_buffer: &MeshBuffer,
        top_level_acceleration_structures: TopLevelAccelerationStructures,

        graphics_queue: Queue,
        target_image_view: ImageView,
    ) -> Self {
        debug!("create pipeline");

        //Descriptor Binding

        //参考にしたものから3番目のmaterial bufferを削除
        let bindings = [
            DescriptorSetLayoutBinding::builder()
                .descriptor_count(1)
                .descriptor_type(vk::DescriptorType::ACCELERATION_STRUCTURE_KHR)
                .stage_flags(vk::ShaderStageFlags::RAYGEN_KHR)
                .binding(0)
                .build(),
            DescriptorSetLayoutBinding::builder()
                .descriptor_count(1)
                .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                .stage_flags(vk::ShaderStageFlags::RAYGEN_KHR)
                .binding(1)
                .build(),
            //VertexBuffer
            DescriptorSetLayoutBinding::builder()
                .descriptor_count(1)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .stage_flags(vk::ShaderStageFlags::CLOSEST_HIT_KHR)
                .binding(3)
                .build(),
            //IndexBuffer
            DescriptorSetLayoutBinding::builder()
                .descriptor_count(1)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .stage_flags(vk::ShaderStageFlags::CLOSEST_HIT_KHR)
                .binding(4)
                .build(),
        ];

        let (
            pipeline_layout,
            descriptor_set_layout
        ) = Self::create_pipeline_layout(&backends.device, &bindings);

        let descriptor_sizes = [
            DescriptorPoolSize {
                ty: DescriptorType::ACCELERATION_STRUCTURE_KHR,
                descriptor_count: 1,
            },
            DescriptorPoolSize {
                ty: DescriptorType::STORAGE_IMAGE,
                descriptor_count: 1,
            },
            DescriptorPoolSize {
                ty: DescriptorType::STORAGE_BUFFER,
                descriptor_count: 1,
            },
            DescriptorPoolSize {
                ty: DescriptorType::STORAGE_BUFFER,
                descriptor_count: 1,
            },
        ];

        let descriptor_pool_info = DescriptorPoolCreateInfo::builder()
            .pool_sizes(&descriptor_sizes)
            .max_sets(1)
            .build();

        let descriptor_pool = unsafe {
            backends.device.create_descriptor_pool(&descriptor_pool_info, None).unwrap()
        };

        let descriptor_counts = [1];

        let mut count_allocate_info = DescriptorSetVariableDescriptorCountAllocateInfo::builder()
            .descriptor_counts(&descriptor_counts)
            .build();

        let descriptor_sets = unsafe {
            //Descriptorはそもそも複数存在し、またDescriptorSetも複数存在する(?)
            backends.device.allocate_descriptor_sets(
                &DescriptorSetAllocateInfo::builder()
                    .descriptor_pool(descriptor_pool)
                    .set_layouts(&[descriptor_set_layout])
                    .push_next(&mut count_allocate_info)
                    .build()
            ).unwrap()
        };

        let descriptor_set = descriptor_sets[0];

        let tlas_structs = [top_level_acceleration_structures.top_level_acceleration_structure_khr];

        let mut acceleration_structure_info = WriteDescriptorSetAccelerationStructureKHR::builder()
            .acceleration_structures(&tlas_structs)
            .build();

        let mut acceleration_structure_write = WriteDescriptorSet::builder()
            .dst_set(descriptor_set)
            .dst_binding(0)
            .dst_array_element(0)
            .descriptor_type(DescriptorType::ACCELERATION_STRUCTURE_KHR)
            .push_next(&mut acceleration_structure_info)
            .build();

        //AccelerationStructureだとdescriptor_countが自動セットされないので手動で設定する必要がある
        acceleration_structure_write.descriptor_count = 1;

        let image_info = [DescriptorImageInfo::builder()
            .image_layout(ImageLayout::GENERAL)
            .image_view(target_image_view)
            .build()
        ];

        let image_write = WriteDescriptorSet::builder()
            .dst_set(descriptor_set)
            .dst_binding(1)
            .dst_array_element(0)
            .descriptor_type(DescriptorType::STORAGE_IMAGE)
            .build();

        let vertex_info = [DescriptorBufferInfo::builder()
            .buffer(mesh_buffer.vertex_buffer.buffer)
            .range(WHOLE_SIZE)
            .build()
        ];

        let vertex_write = WriteDescriptorSet::builder()
            .dst_set(descriptor_set)
            .dst_binding(2)
            .dst_array_element(0)
            .descriptor_type(DescriptorType::STORAGE_BUFFER)
            .buffer_info(&vertex_info)
            .build();

        let index_info = [DescriptorBufferInfo::builder()
            .buffer(mesh_buffer.index_buffer.buffer)
            .range(WHOLE_SIZE)
            .build()
        ];

        let index_write = WriteDescriptorSet::builder()
            .dst_set(descriptor_set)
            .dst_binding(3)
            .dst_array_element(0)
            .descriptor_type(DescriptorType::STORAGE_BUFFER)
            .buffer_info(&index_info)
            .build();

        unsafe {
            backends.device.update_descriptor_sets(
                &[
                    acceleration_structure_write,
                    image_write,
                    vertex_write,
                    index_write,
                ],
                &[],
            )
        }

        //ここから

        //stage

        let shader_stages = unsafe {
            let ray_generation_stage_info = PipelineShaderStageCreateInfo::builder()
                .stage(ShaderStageFlags::RAYGEN_KHR)
                .module(shader_modules.shader_module)
                .name(CStr::from_bytes_with_nul(RAY_GENERATION_SHADER_ENTRY_NAME_BYTE).unwrap())
                .build();

            let miss_stage_info = PipelineShaderStageCreateInfo::builder()
                .stage(ShaderStageFlags::MISS_KHR)
                .module(shader_modules.shader_module)
                .name(CStr::from_bytes_with_nul(MISS_SHADER_ENTRY_NAME_BYTE).unwrap())
                .build();

            let sphere_intersection_stage_info = PipelineShaderStageCreateInfo::builder()
                .stage(ShaderStageFlags::INTERSECTION_KHR)
                .module(shader_modules.shader_module)
                .name(CStr::from_bytes_with_nul(SPHERE_INTERSECTION_SHADER_ENTRY_NAME_BYTE).unwrap())
                .build();

            let sphere_closest_hit_stage_info = PipelineShaderStageCreateInfo::builder()
                .stage(ShaderStageFlags::CLOSEST_HIT_KHR)
                .module(shader_modules.shader_module)
                .name(CStr::from_bytes_with_nul(SPHERE_CLOSEST_HIT_SHADER_ENTRY_NAME_BYTE).unwrap())
                .build();

            let triangle_closest_hit_stage_info = PipelineShaderStageCreateInfo::builder()
                .stage(ShaderStageFlags::CLOSEST_HIT_KHR)
                .module(shader_modules.shader_module)
                .name(CStr::from_bytes_with_nul(TRIANGLE_CLOSEST_HIT_SHADER_ENTRY_NAME_BYTE).unwrap())
                .build();

            let triangle_any_hit_stage_info = PipelineShaderStageCreateInfo::builder()
                .stage(ShaderStageFlags::ANY_HIT_KHR)
                .module(shader_modules.shader_module)
                .name(CStr::from_bytes_with_nul(TRIANGLE_ANY_HIT_SHADER_ENTRY_NAME_BYTE).unwrap())
                .build();

            [
                ray_generation_stage_info,
                miss_stage_info,
                sphere_intersection_stage_info,
                sphere_closest_hit_stage_info,
                triangle_closest_hit_stage_info,
                triangle_any_hit_stage_info,
            ]
        };

        let shader_groups = vec![
            //ray_generation
            RayTracingShaderGroupCreateInfoKHR::builder()
                //RayTracingShaderGroupTypeKHRのGENERALはray_generation, miss, callableのどれかの時に使用
                .ty(RayTracingShaderGroupTypeKHR::GENERAL)
                //shader_stageのindex
                .general_shader(0)
                .closest_hit_shader(SHADER_UNUSED_KHR)
                .any_hit_shader(SHADER_UNUSED_KHR)
                .intersection_shader(SHADER_UNUSED_KHR)
                .build(),
            //miss
            RayTracingShaderGroupCreateInfoKHR::builder()
                .ty(RayTracingShaderGroupTypeKHR::GENERAL)
                .general_shader(1)
                .closest_hit_shader(SHADER_UNUSED_KHR)
                .any_hit_shader(SHADER_UNUSED_KHR)
                .intersection_shader(SHADER_UNUSED_KHR)
                .build(),
            //sphere closest and intersection
            RayTracingShaderGroupCreateInfoKHR::builder()
                .ty(RayTracingShaderGroupTypeKHR::PROCEDURAL_HIT_GROUP)
                .general_shader(SHADER_UNUSED_KHR)
                .closest_hit_shader(3)
                .any_hit_shader(SHADER_UNUSED_KHR)
                .intersection_shader(2)
                .build(),
            //triangle closest and intersection
            RayTracingShaderGroupCreateInfoKHR::builder()
                .ty(RayTracingShaderGroupTypeKHR::TRIANGLES_HIT_GROUP)
                .general_shader(SHADER_UNUSED_KHR)
                .closest_hit_shader(4)
                .any_hit_shader(5)
                //UNUSEDにするとデフォルトでtriangleが使用される？
                .intersection_shader(SHADER_UNUSED_KHR)
                .build(),
        ];

        let (rt_pipeline_properties, rt_pipeline)
            = Self::create_raytracing_structure(&backends.instance, backends.physical_device, &backends.device);

        let pipeline = unsafe {
            rt_pipeline.create_ray_tracing_pipelines(
                DeferredOperationKHR::null(),
                PipelineCache::null(),
                &[
                    RayTracingPipelineCreateInfoKHR::builder()
                        .stages(&shader_stages)
                        .groups(&shader_groups)
                        .max_pipeline_ray_recursion_depth(0)
                        .layout(pipeline_layout)
                        .build()
                ],
                None,
                //なんでVecで帰ってくる？
            ).unwrap()[0]
        };

        Self {
            device: &backends.device,
            pipeline,
            ray_tracing_pipeline_properties: rt_pipeline_properties,
            ray_tracing_pipeline: rt_pipeline,
        }
    }

    fn create_raytracing_structure(
        instance: &Instance,
        physical_device: PhysicalDevice,
        device: &Device
    ) -> (PhysicalDeviceRayTracingPipelinePropertiesKHR, RayTracingPipeline)
    {
        let mut rt_pipeline_properties
            = PhysicalDeviceRayTracingPipelinePropertiesKHR::default();

        let mut physical_device_properties2 = PhysicalDeviceProperties2::builder()
            .push_next(&mut rt_pipeline_properties)
            .build();

        unsafe {
            instance
                .get_physical_device_properties2(physical_device, &mut physical_device_properties2);
        }

        let rt_pipeline = RayTracingPipeline::new(instance, device);

        (rt_pipeline_properties, rt_pipeline)
    }

    fn create_pipeline_layout(device: &Device, bindings: &[DescriptorSetLayoutBinding]) -> (PipelineLayout, DescriptorSetLayout) {
        let descriptor_set_layout = unsafe {
            device.create_descriptor_set_layout(
                &DescriptorSetLayoutCreateInfo::builder()
                    .bindings(&bindings)
                    .build(),
                None,
            ).unwrap()
        };

        let push_constant_range = PushConstantRange::builder()
            .offset(0)
            .size(4)
            .stage_flags(ShaderStageFlags::RAYGEN_KHR)
            .build();

        let layouts = [descriptor_set_layout];
        let layout_create_info = PipelineLayoutCreateInfo::builder()
            .set_layouts(&layouts)
            .push_constant_ranges(&[push_constant_range])
            .build();

        let pipeline_layout = unsafe { device.create_pipeline_layout(&layout_create_info, None).unwrap() };

        (pipeline_layout, descriptor_set_layout)
    }
}