use spirv_builder::{MetadataPrintout, SpirvBuilder};

const VULKAN_TARGET: &str = "spirv-unknown-vulkan1.2";

fn main() -> Result<(), anyhow::Error> {
    SpirvBuilder::new("shaders/classical_raytracer", VULKAN_TARGET)
        .print_metadata(MetadataPrintout::Full)
        .build()?;

    Ok(())
}