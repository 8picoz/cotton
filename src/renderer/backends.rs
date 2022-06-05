use ash::{Device, Entry, Instance};
use ash::extensions::ext::DebugUtils;
use ash::vk::PhysicalDevice;

pub struct Backends {
    pub entry: Entry,
    pub instance: Instance,
    pub physical_device: PhysicalDevice,
    pub device: Device,
}

impl Backends {
    pub fn new() -> anyhow::Result<Self> {
        let entry = unsafe { Entry::load()? };
        let instance = Self::create_instance()?;

    }
}

impl Drop for Backends {
    fn drop(&mut self) {

    }
}