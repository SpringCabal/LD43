pub const MINIMAP_SCALE: f32 = 1.3;
pub const VIEWPORT_WORLD_X: f32 = 8000.0 - 3300.0;
pub const VIEWPORT_WORLD_Z: f32 = 8100.0 - 3400.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Viewport {
    pub origin_x: f32,
    pub origin_z: f32,
    pub width: f32,
    pub height: f32,
    pub scale: f32,
}

pub fn viewport_for_position(position_x: f32, position_z: f32) -> Viewport {
    Viewport {
        origin_x: position_x - VIEWPORT_WORLD_X * 0.5,
        origin_z: position_z - VIEWPORT_WORLD_Z * 0.5,
        width: VIEWPORT_WORLD_X,
        height: VIEWPORT_WORLD_Z,
        scale: MINIMAP_SCALE,
    }
}

pub fn centered_viewport(map_size_x: f32, map_size_z: f32) -> Viewport {
    Viewport {
        origin_x: (map_size_x - VIEWPORT_WORLD_X) * 0.5,
        origin_z: (map_size_z - VIEWPORT_WORLD_Z) * 0.5,
        width: VIEWPORT_WORLD_X,
        height: VIEWPORT_WORLD_Z,
        scale: MINIMAP_SCALE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centered_viewport_follows_player_interior() {
        let viewport = viewport_for_position(5580.0, 7100.0);

        assert_eq!(viewport.origin_x, 5580.0 - VIEWPORT_WORLD_X * 0.5);
        assert_eq!(viewport.origin_z, 7100.0 - VIEWPORT_WORLD_Z * 0.5);
        assert_eq!(viewport.width, VIEWPORT_WORLD_X);
        assert_eq!(viewport.height, VIEWPORT_WORLD_Z);
        assert_eq!(viewport.scale, MINIMAP_SCALE);
    }

    #[test]
    fn north_edge_keeps_player_centered() {
        let viewport = viewport_for_position(5580.0, 1000.0);

        assert_eq!(viewport.origin_x, 5580.0 - VIEWPORT_WORLD_X * 0.5);
        assert_eq!(viewport.origin_z, 1000.0 - VIEWPORT_WORLD_Z * 0.5);
        assert_eq!(viewport.width, VIEWPORT_WORLD_X);
        assert_eq!(viewport.height, VIEWPORT_WORLD_Z);
        assert_eq!(viewport.scale, MINIMAP_SCALE);
    }

    #[test]
    fn centered_viewport_is_in_the_middle_of_the_map() {
        let viewport = centered_viewport(12288.0, 12288.0);

        assert_eq!(viewport.origin_x + viewport.width * 0.5, 6144.0);
        assert_eq!(viewport.origin_z + viewport.height * 0.5, 6144.0);
    }
}
