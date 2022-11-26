use std::{cell::RefCell, collections::HashMap, ops::Div};

use cgmath::Point2;
use framework::{
    framework::TextureId, Box2d, Framework, RgbaTexture2D, Texture, TextureConfiguration,
    TextureUsage,
};

#[derive(Debug)]
struct ChunkLayerMutabledata {
    chunks: HashMap<Point2<i64>, TextureId>,
    bounds: Box2d,
}

impl ChunkLayerMutabledata {
    fn new() -> Self {
        Self {
            chunks: HashMap::new(),
            bounds: Box2d::default(),
        }
    }
}

#[derive(Debug)]
pub struct ChunkedLayer {
    label: String,
    chunk_size: u32,
    inner_chunk_data: RefCell<ChunkLayerMutabledata>,
}

impl ChunkedLayer {
    pub(crate) fn new(label: &str, chunk_size: u32) -> Self {
        Self {
            label: label.to_owned(),
            chunk_size,
            inner_chunk_data: RefCell::new(ChunkLayerMutabledata::new()),
        }
    }
    pub fn chunk(&self, pos: Point2<f32>, framework: &mut Framework) -> TextureId {
        let chunk_index = pos.cast::<i64>().unwrap().div(self.chunk_size as i64);
        let mut map = self.inner_chunk_data.borrow_mut();
        if !map.chunks.contains_key(&chunk_index) {
            Self::create_chunk(
                &self.label,
                self.chunk_size,
                chunk_index,
                framework,
                &mut map.chunks,
            );
        }
        map.bounds.expand_with_point(pos);

        map.chunks.get(&chunk_index).cloned().unwrap()
    }

    fn create_chunk(
        label: &str,
        size: u32,
        pos: Point2<i64>,
        framework: &mut Framework,
        map: &mut HashMap<Point2<i64>, TextureId>,
    ) {
        let chunk_texture = framework.allocate_texture2d(
            RgbaTexture2D::empty((size, size)),
            TextureConfiguration {
                label: Some(format!("Chunked Layer '{:?}' texture {:?}", label, pos).as_str()),
                usage: TextureUsage::RWRT,
                mip_count: None,
            },
        );
        map.insert(pos, chunk_texture);
    }

    pub(crate) fn bounds(&self) -> Box2d {
        self.inner_chunk_data.borrow().bounds.clone()
    }
}
