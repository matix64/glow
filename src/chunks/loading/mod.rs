use std::iter::repeat_with;

use anvil_nbt::CompoundTag;
use anvil_region::position::RegionChunkPosition;
use anvil_region::position::RegionPosition;
use anvil_region::provider::RegionProvider;
use async_trait::async_trait;
use anvil_region::provider::FolderRegionProvider;
use tokio::task;
use crate::common::block::Block;

use super::Chunk;
use super::ChunkCoords;
use super::chunk::CHUNK_HEIGHT;
use super::chunk_source::ChunkSource;
use super::palette::Palette;
use super::section::SECTION_LENGTH;
use super::section::Section;

pub struct AnvilChunkLoader;

impl AnvilChunkLoader {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ChunkSource for AnvilChunkLoader {
    async fn load_chunk(&self, coords: ChunkCoords) -> Option<Chunk> {
        let chunk = read_chunk(coords).await?;
        let section_tags = chunk
            .get_compound_tag("Level").unwrap()
            .get_compound_tag_vec("Sections").unwrap();
        let mut sections: Vec<Option<Section>> = 
            repeat_with(|| None)
            .take(CHUNK_HEIGHT / SECTION_LENGTH)
            .collect();
        for tag in section_tags {
            if let Ok(palette) = 
                tag.get_compound_tag_vec("Palette") 
            {
                let palette_entries: Vec<Block> = palette.into_iter()
                    .map(|tag| tag.get_str("Name").unwrap())
                    .map(|name| Block::from_name(name).unwrap())
                    .collect();
                let palette = Palette::from_entries(palette_entries.as_slice());
                let blocks = tag.get_i64_vec("BlockStates").unwrap();
                let section = Section::from_raw(blocks.clone(), palette);
                let y = tag.get_i8("Y").unwrap() as usize;
                sections[y] = Some(section);
            }
        }
        Some(Chunk::from_sections(sections))
    }
}

async fn read_chunk(coords: ChunkCoords) -> Option<CompoundTag> {
    let ChunkCoords(chunk_x, chunk_y) = coords;
    let region_position = 
        RegionPosition::from_chunk_position(chunk_x, chunk_y);
    let chunk_position = 
        RegionChunkPosition::from_chunk_position(chunk_x, chunk_y);
    task::spawn_blocking(move || {
        let provider = FolderRegionProvider::new("world/region");
        let mut region = provider.get_region(region_position).ok()?;
        region.read_chunk(chunk_position).ok()
    }).await.ok()?
}