use anyhow::Result;
use nalgebra::Vector3;
use std::sync::Arc;
use tokio::io::AsyncWrite;
use tokio::sync::RwLock;

use crate::game::Chunk;
use crate::game::ChunkCoords;

use super::packet_builder::PacketBuilder;

pub enum ServerEvent {
    LoadChunk(ChunkCoords, Arc<RwLock<Chunk>>),
    KeepAlive(u64),
    PlayerPosition(Vector3<f32>)
}

impl ServerEvent {
    pub async fn write_to<W: AsyncWrite>(&self, writer: &mut W) -> Result<()>
        where W: Unpin
    {
        match self {
            ServerEvent::LoadChunk(coords, chunk) => {
                let chunk = chunk.read().await;
                let mut packet = PacketBuilder::new(0x20);
                packet
                    .add_bytes(&coords.0.to_be_bytes())
                    .add_bytes(&coords.1.to_be_bytes())
                    .add_bytes(&[1])
                    .add_varint(chunk.get_sections_bitmask() as u32)
                    .add_nbt(&chunk.get_heightmap())
                    .add_varint(1024);
                for biome in chunk.get_biome_map() {
                    packet.add_varint(biome as u32);
                }
                let chunk_data = chunk.get_data();
                packet
                    .add_varint(chunk_data.len() as u32)
                    .add_bytes(chunk_data.as_slice())
                    .add_varint(0)
                    .write(writer).await
            }
            ServerEvent::KeepAlive(id) => {
                PacketBuilder::new(0x1F)
                    .add_bytes(&id.to_be_bytes())
                    .write(writer).await
            }
            ServerEvent::PlayerPosition(pos) => {
                PacketBuilder::new(0x34)
                    .add_bytes(&(pos.x as f64).to_be_bytes())
                    .add_bytes(&(pos.y as f64).to_be_bytes())
                    .add_bytes(&(pos.z as f64).to_be_bytes())
                    .add_bytes(&0f32.to_be_bytes()) // Yaw
                    .add_bytes(&0f32.to_be_bytes()) // Pitch
                    .add_bytes(&[0b11000]) // Rotation relative, position absolute
                    .add_varint(0) // Teleport ID, used by client to confirm
                    .write(writer).await
            }
        }
    }
}