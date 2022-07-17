#![cfg_attr(
    target_os = "spirv",
    no_std,
    feature(register_attr),
    register_attr(spirv)
)]

pub mod vertex;
