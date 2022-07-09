use spirv_std::glam::Vec3A;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Vertex {
    //Aがついている型はSIMDが使用される
    pub position: Vec3A,
    pub normal: Vec3A,
}