use serde::{
    Deserialize,
    Serialize,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
// todo: rename to LogicalFrame
// A logical rect, where the origin point is the top left corner.
pub struct Rect<U, V> {
    pub x: U,
    pub y: U,
    pub width: V,
    pub height: V,
}
