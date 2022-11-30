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
    chunks_created: Vec<Point2<i64>>,
    chunks_modified: HashMap<Point2<i64>, TextureId>,
    chunks_deleted: HashMap<Point2<i64>, TextureId>,
}

impl ChunkDiff {
    pub fn new() -> Self {
        ChunkDiff {
            chunks_created: vec![],
            chunks_modified: HashMap::new(),
            chunks_deleted: HashMap::new(),
        }
    }

    pub(crate) fn update_with_chunk(
        &mut self,
        chunk_layer: &ChunkedLayer,
        chunk_index: &Point2<i64>,
        framework: &mut Framework,
    ) {
        if let Some(_) = chunk_layer.chunks.get(&chunk_index) {
            if !self.chunks_modified.contains_key(&chunk_index)
                && !self.chunks_created.contains(&chunk_index)
            {
                let chunk_texture_copy = chunk_layer.clone_chunk_texture(&chunk_index, framework);

                self.chunks_modified
                    .insert(chunk_index.clone(), chunk_texture_copy);
            }
        } else {
            self.chunks_created.push(chunk_index.clone());
        }
    }

    pub fn apply_to_chunked_layer(
        &self,
        layer: &mut ChunkedLayer,
        framework: &mut Framework,
    ) -> Self {
        let mut inverted_diff = Self::new();
        for index_created in self.chunks_created.iter() {
            let deleted_content = layer
                .chunks
                .remove(&index_created)
                .expect("apply_diff: chunk marked as created is not existing");
            inverted_diff
                .chunks_deleted
                .insert(index_created.clone(), deleted_content);
        }
        for (index_modified, modified_texture) in self.chunks_modified.iter() {
            let current_copy = layer.clone_chunk_texture(&index_modified, framework);
            inverted_diff
                .chunks_modified
                .insert(index_modified.clone(), current_copy);
            layer
                .chunks
                .insert(*index_modified, modified_texture.clone());
        }
        for (index_deleted, deleted_content) in self.chunks_deleted.iter() {
            inverted_diff.chunks_created.push(index_deleted.clone());
            layer.chunks.insert(*index_deleted, deleted_content.clone());
        }

        inverted_diff
    }
    pub fn join(&mut self, other: ChunkDiff) {
        self.chunks_created.extend(other.chunks_created);
        for (idx, modified) in other.chunks_modified {
            if !self.chunks_created.contains(&idx) {
                self.chunks_modified.insert(idx, modified);
            }
        }
        self.chunks_deleted.extend(other.chunks_deleted);
    }

    pub fn take(&mut self) -> Self {
        let mut taken = ChunkDiff::new();
        for created in self.chunks_created.iter() {
            taken.chunks_created.push(created.clone());
        }
        for (index, modified) in self.chunks_modified.iter() {
            taken
                .chunks_modified
                .insert(index.clone(), modified.clone());
        }
        for (index, deleted) in self.chunks_deleted.iter() {
            taken.chunks_deleted.insert(index.clone(), deleted.clone());
        }
        self.chunks_created.clear();
        self.chunks_deleted.clear();
        self.chunks_modified.clear();
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
