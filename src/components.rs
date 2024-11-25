#[derive(Debug, Clone, Copy)]
pub struct Position(pub f32, pub f32, pub f32); // Position stored in (x, y, degrees).

#[derive(Debug, Clone, Copy)]
pub struct Velocity(pub f32, pub f32, pub f32); // Position stored in (x, y, degrees).

#[derive(Debug, Clone)]
pub struct Draw(pub macroquad::color::Color, pub Vec<(f32, f32)>); // Drawables are vectors consisting of four points, a (x1, y1) and (x2, y2). Draw lines based on these and the rotation.

#[derive(Debug, Clone, Copy)]
pub enum CollidableType {
    PLAYER,
    ASTEROID,
    BULLET,
}

#[derive(Debug, Clone, Copy)]
pub struct Collidable(pub f32, pub CollidableType); // Just storing the radius of the object's hitbox.

#[derive(Debug, Clone, Copy)]
pub struct Controllable(); // For allowing this entity to be controlled.
