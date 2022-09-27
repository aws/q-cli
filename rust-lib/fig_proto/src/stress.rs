pub use crate::proto::stress::*;

pub fn build_serverbound(inner: serverbound::Inner) -> Serverbound {
    Serverbound { inner: Some(inner) }
}

pub fn build_clientbound(inner: clientbound::Inner) -> Clientbound {
    Clientbound { inner: Some(inner) }
}
