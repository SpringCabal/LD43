pub fn location(spring: &spring::Spring<'_, spring::Ui>, shader: u32, name: &str) -> Option<i32> {
    spring.gfx().get_uniform_location(shader, name).ok()
}

pub fn set_int(
    spring: &spring::Spring<'_, spring::Ui>,
    shader: u32,
    name: &str,
    value: i32,
) -> bool {
    let Some(location) = location(spring, shader, name) else {
        return false;
    };
    spring.gfx().uniform_int(location, &[value], 1).is_ok()
}
