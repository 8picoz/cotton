pub const DEFAULT_WINDOW_WIDTH: u32 = 1920;
pub const DEFAULT_WINDOW_HEIGHT: u32 = 1080;

pub const MAX_FRAMES_IN_FLIGHT: u32 = 1;

pub const APPLICATION_NAME: &str = "cotton";

pub const VERTEX_SHADER_ENTRY_NAME: &str = "main_vertex";
pub const FRAGMENT_SHADER_ENTRY_NAME: &str = "main_fragment";
pub const RAY_GENERATION_SHADER_ENTRY_NAME: &str = "main_ray_generation";
pub const MISS_SHADER_ENTRY_NAME: &str = "main_miss";
pub const SPHERE_INTERSECTION_SHADER_ENTRY_NAME: &str = "sphere_intersection";
pub const SPHERE_CLOSEST_HIT_SHADER_ENTRY_NAME: &str = "sphere_closest_hit";
pub const TRIANGLE_CLOSEST_HIT_SHADER_ENTRY_NAME: &str = "triangle_closest_hit";
pub const TRIANGLE_ANY_HIT_SHADER_ENTRY_NAME: &str = "triangle_any_hit";