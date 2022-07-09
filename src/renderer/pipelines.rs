use std::ffi::{CStr, CString};
use ash::{Device, Instance, vk};
use ash::extensions::khr::{AccelerationStructure, RayTracingPipeline};
use ash::vk::{AccelerationStructureNV, DeferredOperationKHR, DescriptorSetLayoutBinding, DescriptorSetLayoutCreateInfo, Extent2D, PhysicalDevice, PhysicalDeviceProperties2, PhysicalDeviceRayTracingPipelinePropertiesKHR, Pipeline, PipelineCache, PipelineLayout, PipelineLayoutCreateInfo, PipelineShaderStageCreateInfo, PushConstantRange, RayTracingPipelineCreateInfoKHR, RayTracingShaderGroupCreateInfoKHR, RayTracingShaderGroupTypeKHR, SHADER_UNUSED_KHR, ShaderModule, ShaderStageFlags};
use crate::constants::{FRAGMENT_SHADER_ENTRY_NAME, MISS_SHADER_ENTRY_NAME, RAY_GENERATION_SHADER_ENTRY_NAME, SPHERE_CLOSEST_HIT_SHADER_ENTRY_NAME, SPHERE_INTERSECTION_SHADER_ENTRY_NAME, TRIANGLE_ANY_HIT_SHADER_ENTRY_NAME, TRIANGLE_CLOSEST_HIT_SHADER_ENTRY_NAME, VERTEX_SHADER_ENTRY_NAME};
use crate::renderer::acceleration_structures::TriangleAccelerationStructure;
use crate::renderer::backends::Backends;
use crate::renderer::render_passes::RenderPasses;

pub struct Pipelines<'a> {
    pub device: &'a Device,
    pub pipeline: Pipeline,

    pub(crate) acceleration_structures: TriangleAccelerationStructure<'a>,
    pub(crate) ray_tracing_pipeline: RayTracingPipeline,
    pub(crate) ray_tracing_pipeline_properties: PhysicalDeviceRayTracingPipelinePropertiesKHR,
}

impl Pipelines<'_> {
    //with raytracing
    pub fn new(backends: &Backends, shader_module: ShaderModule, swapchain_extent: Extent2D, render_passes: &RenderPasses) -> Self {

        let ray_generation_stage_info = PipelineShaderStageCreateInfo::builder()
            .stage(ShaderStageFlags::RAYGEN_KHR)
            .module(shader_module)
            .name(CString::new(RAY_GENERATION_SHADER_ENTRY_NAME).unwrap().as_c_str())
            .build();

        let miss_stage_info = PipelineShaderStageCreateInfo::builder()
            .stage(ShaderStageFlags::MISS_KHR)
            .module(shader_module)
            .name(CString::new(MISS_SHADER_ENTRY_NAME).unwrap().as_c_str())
            .build();

        let sphere_intersection_stage_info = PipelineShaderStageCreateInfo::builder()
            .stage(ShaderStageFlags::INTERSECTION_KHR)
            .module(shader_module)
            .name(CString::new(SPHERE_INTERSECTION_SHADER_ENTRY_NAME).unwrap().as_c_str())
            .build();

        let sphere_closest_hit_stage_info = PipelineShaderStageCreateInfo::builder()
            .stage(ShaderStageFlags::CLOSEST_HIT_KHR)
            .module(shader_module)
            .name(CString::new(SPHERE_CLOSEST_HIT_SHADER_ENTRY_NAME).unwrap().as_c_str())
            .build();

        let triangle_closest_hit_stage_info = PipelineShaderStageCreateInfo::builder()
            .stage(ShaderStageFlags::CLOSEST_HIT_KHR)
            .module(shader_module)
            .name(CString::new(TRIANGLE_CLOSEST_HIT_SHADER_ENTRY_NAME).unwrap().as_c_str())
            .build();

        let triangle_any_hit_stage_info = PipelineShaderStageCreateInfo::builder()
            .stage(ShaderStageFlags::ANY_HIT_KHR)
            .module(shader_module)
            .name(CString::new(TRIANGLE_ANY_HIT_SHADER_ENTRY_NAME).unwrap().as_c_str())
            .build();

        let shader_stages = [
            ray_generation_stage_info,
            miss_stage_info,
            sphere_intersection_stage_info,
            sphere_closest_hit_stage_info,
            triangle_closest_hit_stage_info,
            triangle_any_hit_stage_info,
        ];

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

        let acceleration_structures = TriangleAccelerationStructure::new();

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
            acceleration_structures,
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

    fn create_pipeline_layout(device: &Device, bindings: &[DescriptorSetLayoutBinding]) -> PipelineLayout {
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

        unsafe { device.create_pipeline_layout(&layout_create_info, None).unwrap() }
    }
}