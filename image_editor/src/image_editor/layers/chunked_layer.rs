use std::{collections::HashMap, ops::Div};

use cgmath::{point2, ElementWise, Point2};
use framework::{
    framework::TextureId, Box2d, Framework, RgbaTexture2D, Texture, TextureConfiguration,
    TextureUsage,
};

#[derive(Debug)]
pub struct ChunkedLayer {
    label: String,
    chunk_size: u32,
    chunks: HashMap<Point2<i64>, TextureId>,
    bounds: Box2d,
}

#[derive(Debug)]
pub struct ChunkDiff {
    diff: HashMap<Point2<i64>, Option<TextureId>>,
}

impl ChunkDiff {
    pub fn new() -> Self {
        ChunkDiff {
            diff: HashMap::new(),
        }
    }

    pub(crate) fn update_with_chunk(
        &mut self,
        chunk_layer: &ChunkedLayer,
        chunk_index: &Point2<i64>,
        framework: &mut Framework,
    ) {
        if !self.diff.contains_key(chunk_index) {
            if chunk_layer.chunks.contains_key(chunk_index) {
                // modified
                let chunk_clone = chunk_layer.clone_chunk_texture(chunk_index, framework);
                self.diff.insert(*chunk_index, Some(chunk_clone));
            } else {
                //created
                self.diff.insert(*chunk_index, None);
            }
        }
    }

    pub fn apply_to_chunked_layer(&self, layer: &mut ChunkedLayer) -> Self {
        log::trace!("ChunkDiff: Applying diff: \n\t{:?}", self.diff);
        log::trace!("ChunkDiff: Layer before diff: ");
        log::trace!("\t{:?}", layer.chunks);
        let mut inverted_diff = Self::new();
        for (index, diff) in self.diff.iter() {
            if let Some(texture) = diff {
                // either created or modified
                let old = layer.chunks.insert(index.clone(), texture.clone());
                inverted_diff.diff.insert(*index, old);
            } else {
                let old = layer
                    .chunks
                    .remove(&index)
                    .expect("Diff: chunk should be deleted but it doesn't exist");
                inverted_diff.diff.insert(*index, Some(old));
            }
        }
        log::trace!("ChunkDiff: inverted diff: \n\t{:?}", inverted_diff.diff);
        log::trace!("ChunkDiff: Layer after diff: \n\t{:?}", layer.chunks);
        inverted_diff
    }
    pub fn join(&mut self, other: &ChunkDiff) {
        for (index, diff) in other.diff.iter() {
            if !self.diff.contains_key(index) {
                // Ignore the latest updates (e.g created, afterwards modified, we should only pick the created)
                self.diff.insert(*index, diff.clone());
            }
        }
    }

    pub fn take(&mut self) -> Self {
        let mut taken = ChunkDiff::new();
        taken.join(self);

        self.diff.clear();
        taken
    }
}

impl ChunkedLayer {
    pub(crate) fn new(label: &str, chunk_size: u32) -> Self {
        Self {
            label: label.to_owned(),
            chunk_size,
            chunks: HashMap::new(),
            bounds: Box2d::default(),
        }
    }
    pub fn chunk(&self, pos: Point2<f32>) -> Option<TextureId> {
        let chunk_index = pos.cast::<i64>().unwrap().div(self.chunk_size as i64);
        self.chunks.get(&chunk_index).cloned()
    }

    // Use this to interact with the chunk on the chunk map, filling eventual holes.
    // F: (chunk, chunk index, chunk world position, chunk was just created)
    // FIXME: Using an AABB brings issues when the AABB is created with a rotation, so
    //  we should to an OBB instead
    pub fn edit<F: FnMut(&TextureId, Point2<i64>, Point2<f32>, &mut Framework)>(
        &mut self,
        bounds: Box2d,
        mut f: F,
        framework: &mut Framework,
    ) -> ChunkDiff {
        self.bounds = self.bounds.union(&bounds);
        let first_chunk = Point2 {
            x: bounds.left() / self.chunk_size as f32,
            y: bounds.top() / self.chunk_size as f32,
        }
        .add_element_wise(point2(bounds.left().signum(), bounds.top().signum()) * 0.5)
        .cast::<i64>()
        .unwrap();
        let last_chunk = Point2 {
            x: bounds.right() / self.chunk_size as f32,
            y: bounds.bottom() / self.chunk_size as f32,
        }
        .add_element_wise(point2(bounds.right().signum(), bounds.bottom().signum()) * 0.5)
        .cast::<i64>()
        .unwrap();
        log::info!(
            "App,ChunkedLayer: left {:?} right {:?}",
            first_chunk,
            last_chunk
        );
        let mut diff = ChunkDiff::new();
        for x in first_chunk.x..=last_chunk.x {
            for y in first_chunk.y..=last_chunk.y {
                let chunk_index = point2(x, y);
                diff.update_with_chunk(self, &chunk_index, framework);

                log::info!("App,ChunkedLayer: editing chunk {:?}", chunk_index);
                let chunk_position = self.index_to_world_position(&chunk_index);
                self.allocate_chunk_if_needed(chunk_index, framework);
                let chunk = self.chunks.get(&chunk_index).unwrap();
                f(chunk, chunk_index, chunk_position, framework);
            }
        }
        diff
    }

    fn allocate_chunk_if_needed(&mut self, chunk_index: Point2<i64>, framework: &mut Framework) {
        if !self.chunks.contains_key(&chunk_index) {
            log::info!(
                "App,ChunkedLayer: Allocating a chunk at [{}, {}], chunk size: {}",
                chunk_index.x,
                chunk_index.y,
                self.chunk_size
            );
            Self::create_chunk(
                &self.label,
                self.chunk_size,
                chunk_index,
                framework,
                &mut self.chunks,
            );
        } else {
        }
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

    // Use this to e.g interact with the chunk map without filling the holes in the region.
    pub fn view<F: FnMut(&TextureId, Point2<i64>)>(&self, bounds: Box2d, mut f: F) {
        let chunks_modified =
            bounds.extents.add_element_wise(0.5).cast::<u32>().unwrap() / self.chunk_size;
        let first_chunk = bounds.center.cast::<i64>().unwrap() / self.chunk_size as i64;
        for (x, y) in (0..=chunks_modified.x).zip(0..=chunks_modified.y) {
            let chunk_index = point2(first_chunk.x + x as i64, first_chunk.y + y as i64);
            if let Some(chunk) = self.chunks.get(&chunk_index) {
                f(chunk, chunk_index);
            }
        }
    }

    // Use this to iterate all existing chunks in the map.
    // F's arguments are (chunk, chunk index, chunk position in world space)
    pub fn iterate<F: FnMut(&TextureId, Point2<i64>, Point2<f32>)>(&self, mut f: F) {
        for (index, chunk) in self.chunks.iter() {
            let chunk_position = self.index_to_world_position(index);
            f(chunk, *index, chunk_position);
        }
    }

    pub fn chunk_size(&self) -> u32 {
        self.chunk_size
    }

    pub(crate) fn bounds(&self) -> Box2d {
        self.bounds.clone()
    }

    fn index_to_world_position(&self, chunk_index: &Point2<i64>) -> Point2<f32> {
        chunk_index
            .cast::<f32>()
            .unwrap()
            .mul_element_wise(self.chunk_size as f32)
    }

    fn clone_chunk_texture(
        &self,
        chunk_index: &Point2<i64>,
        framework: &mut Framework,
    ) -> TextureId {
        let chunk_texture = self
            .chunks
            .get(chunk_index)
            .expect("clone_chunk_texture: index not found");
        framework.texture2d_copy_subregion(
            chunk_texture,
            0,
            0,
            self.chunk_size(),
            self.chunk_size(),
        )
    }
}
