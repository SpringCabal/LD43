use spring::gfx;

pub fn location(shader: u32, name: &str) -> Option<i32> {
    gfx::get_uniform_location(shader, name).ok()
}

pub fn set_int(shader: u32, name: &str, value: i32) -> bool {
    let Some(location) = location(shader, name) else {
        return false;
    };
    gfx::uniform_int(location, &[value], 1).is_ok()
}
