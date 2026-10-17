pub fn ground_at(
    spring: &spring::Spring<'_, spring::Ui>,
    x: f32,
    y: f32,
) -> Option<spring::Float3> {
    spring.camera().ground_at(x, y).ok().flatten()
}
pub fn unit_at(spring: &spring::Spring<'_, spring::Ui>, x: f32, y: f32) -> Option<i32> {
    spring.camera().unit_at(x, y).ok().flatten()
}
pub fn screen(spring: &spring::Spring<'_, spring::Ui>) -> Option<(f32, f32)> {
    springcabal_control::pointer::mouse(spring).map(|mouse| (mouse.x, mouse.y))
}
