#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Hash)]
pub struct LayerIndex(pub u16);

#[derive(Clone, PartialEq, PartialOrd, Eq, Hash)]
pub enum LayerTree {
    SingleLayer(LayerIndex),
    Group(Vec<LayerIndex>),
}
pub struct RootLayer(pub Vec<LayerTree>);
