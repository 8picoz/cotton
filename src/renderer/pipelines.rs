use std::ffi::{CStr, CString};
use ash::{Device, vk};
use ash::vk::{Extent2D, Pipeline, PipelineShaderStageCreateInfo, ShaderModule, ShaderStageFlags};
use crate::constants::{FRAGMENT_SHADER_ENTRY_NAME, MISS_SHADER_ENTRY_NAME, RAY_GENERATION_SHADER_ENTRY_NAME, SPHERE_CLOSEST_HIT_SHADER_ENTRY_NAME, SPHERE_INTERSECTION_SHADER_ENTRY_NAME, TRIANGLE_ANY_HIT_SHADER_ENTRY_NAME, TRIANGLE_CLOSEST_HIT_SHADER_ENTRY_NAME, VERTEX_SHADER_ENTRY_NAME};
use crate::renderer::render_passes::RenderPasses;

pub struct Pipelines<'a> {
    pub device: &'a Device,
}

impl Pipelines {
    //with raytracing
    pub fn new(device: &Device, shader_module: ShaderModule, swapchain_extent: Extent2D, render_passes: &RenderPasses) -> Self {

        let

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

        Self {
            device,

        }
    }
}