mod params;

mod houses;
mod mouse;
mod projectiles;
mod staff;

pub use houses::place as place_house_lights;
pub use mouse::update as update_mouse_light;
pub use projectiles::update as update_projectile_lights;
pub use staff::update as update_staff_light;
