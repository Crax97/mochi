#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Hash)]
pub(crate) struct LayerIndex(pub u16);
pub(crate) enum LayerTree {
    SingleLayer(LayerIndex),
    Group(Vec<LayerIndex>),
}
pub(crate) struct RootLayer(pub Vec<LayerTree>);
