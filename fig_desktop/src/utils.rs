use serde::{
    Deserialize,
    Serialize,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rect<U, V> {
    pub x: U,
    pub y: U,
    pub width: V,
    pub height: V,
}
