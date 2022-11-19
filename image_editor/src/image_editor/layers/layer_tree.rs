use super::LayerId;

#[derive(Clone, PartialEq, PartialOrd, Eq, Hash)]
pub enum LayerTree {
    SingleLayer(LayerId),
    Group(Vec<LayerId>),
}
pub struct RootLayer(pub Vec<LayerTree>);
