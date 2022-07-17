pub const DEFAULT_WINDOW_WIDTH: u32 = 1920;
pub const DEFAULT_WINDOW_HEIGHT: u32 = 1080;

pub const MAX_FRAMES_IN_FLIGHT: u32 = 1;

pub const APPLICATION_NAME: &str = "cotton";

//マクロにしてくれ
pub const VERTEX_SHADER_ENTRY_NAME: &str = "main_vertex";
pub const VERTEX_SHADER_ENTRY_NAME_BYTE: &[u8] = b"main_vertex\0";
pub const FRAGMENT_SHADER_ENTRY_NAME: &str = "main_fragment";
pub const FRAGMENT_SHADER_ENTRY_NAME_BYTE: &[u8] = b"main_fragment\0";
pub const RAY_GENERATION_SHADER_ENTRY_NAME: &str = "main_ray_generation";
pub const RAY_GENERATION_SHADER_ENTRY_NAME_BYTE: &[u8] = b"main_ray_generation\0";
pub const MISS_SHADER_ENTRY_NAME: &str = "main_miss";
pub const MISS_SHADER_ENTRY_NAME_BYTE: &[u8] = b"main_miss\0";
pub const SPHERE_INTERSECTION_SHADER_ENTRY_NAME: &str = "sphere_intersection";
pub const SPHERE_INTERSECTION_SHADER_ENTRY_NAME_BYTE: &[u8] = b"sphere_intersection\0";
pub const SPHERE_CLOSEST_HIT_SHADER_ENTRY_NAME: &str = "sphere_closest_hit";
pub const SPHERE_CLOSEST_HIT_SHADER_ENTRY_NAME_BYTE: &[u8] = b"sphere_closest_hit\0";
pub const TRIANGLE_CLOSEST_HIT_SHADER_ENTRY_NAME: &str = "triangle_closest_hit";
pub const TRIANGLE_CLOSEST_HIT_SHADER_ENTRY_NAME_BYTE: &[u8] = b"triangle_closest_hit\0";
pub const TRIANGLE_ANY_HIT_SHADER_ENTRY_NAME: &str = "triangle_any_hit";
pub const TRIANGLE_ANY_HIT_SHADER_ENTRY_NAME_BYTE: &[u8] = b"triangle_any_hit\0";
