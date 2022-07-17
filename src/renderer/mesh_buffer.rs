use ash::Device;
use ash::vk::{BufferUsageFlags, DeviceSize, MemoryPropertyFlags, PhysicalDeviceMemoryProperties};
use classical_raytracer_shader::vertex::Vertex;
use crate::buffer::Buffers;

pub struct MeshBuffer<'a> {
    device: &'a Device,
    pub vertex_stride: u64,
    pub max_vertex: u32,
    pub vertex_buffer: Buffers<'a>,
    pub index_buffer: Buffers<'a>,
}

impl MeshBuffer<'_> {
    pub fn new(
        device: &Device,
        vertices: Vec<Vertex>,
        indices: Vec<u32>,
        device_memory_properties: PhysicalDeviceMemoryProperties,
    ) -> Self {
        let vertex_stride = std::mem::size_of::<Vertex>();
        let vertex_buffer_size = vertex_stride * vertices.len();
        let max_vertex = vertices.len() as u32 - 1;

        let mut vertex_buffer = Buffers::new(
            device,
            device_memory_properties,
            vertex_buffer_size as DeviceSize,
            BufferUsageFlags::STORAGE_BUFFER
                | BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR,
            MemoryPropertyFlags::HOST_VISIBLE
                | MemoryPropertyFlags::HOST_COHERENT,
        );

        vertex_buffer.store(&vertices);

        let index_buffer_size = std::mem::size_of::<u32>() * indices.len();

        let mut index_buffer = Buffers::new(
            device,
            device_memory_properties,
            index_buffer_size as DeviceSize,
            BufferUsageFlags::STORAGE_BUFFER
                | BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR,
            MemoryPropertyFlags::HOST_VISIBLE
                | MemoryPropertyFlags::HOST_COHERENT,
        );

        index_buffer.store(&indices);

        let vertex_stride = vertex_stride as u64;

        Self {
            device,
            vertex_stride,
            max_vertex,
            vertex_buffer,
            index_buffer,
        }
    }
}