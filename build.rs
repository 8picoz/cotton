use spirv_builder::{Capability, MetadataPrintout, SpirvBuilder};

const VULKAN_TARGET: &str = "spirv-unknown-vulkan1.2";

fn main() -> Result<(), anyhow::Error> {
    SpirvBuilder::new("shaders/classical_raytracer_shader", VULKAN_TARGET)
        .capability(Capability::RayTracingKHR)
        .extension("SPV_KHR_ray_tracing")
        .print_metadata(MetadataPrintout::Full)
        .build()?;

    Ok(())
}