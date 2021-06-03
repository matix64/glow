use legion::*;
use nalgebra::vector;
use crate::buckets::EntityTracker;
use crate::buckets::events::{EntityEvent, EntityEventData};
use crate::entities::EntityId;
use crate::net::PlayerConnection;
use crate::entities::{Position, Rotation};
use crate::net::packets::play::ServerboundPacket;
use crate::chunks::{Block, World as ChunkWorld};
use super::disconnections::DisconnectionQueue;

#[system(for_each)]
pub fn receive_events(entity: &Entity, id: &EntityId, conn: &mut PlayerConnection, 
    position: &mut Position, rotation: &mut Rotation, #[resource] chunks: &ChunkWorld, 
    #[resource] disconnections: &DisconnectionQueue, #[resource] tracker: &mut EntityTracker) 
{
    for event in conn.receive() {
        match event {
            ServerboundPacket::PlayerPosition { x, y, z, .. } => {
                let new_position = vector!(x, y, z);
                tracker.move_entity(id.0, *entity, position.0, new_position);
                tracker.send_event(&new_position, EntityEvent {
                    id: id.0,
                    data: EntityEventData::Move {
                        delta: new_position - position.0,
                    }
                });
                position.0 = new_position;
            },
            ServerboundPacket::PlayerRotation { yaw, pitch, .. } => {
                tracker.send_event(&position.0, 
                    EntityEvent {
                        id: id.0,
                        data: EntityEventData::Rotate{ pitch, yaw },
                    }
                );
                tracker.send_event(&position.0, 
                    EntityEvent {
                        id: id.0,
                        data: EntityEventData::RotateHead{ yaw },
                    }
                );
                *rotation = Rotation(yaw, pitch);
            },
            ServerboundPacket::PlayerPositionAndRotation {
                x, y, z, yaw, pitch, ..
            } => {
                let new_position = vector!(x, y, z);
                tracker.move_entity(id.0, *entity, position.0, new_position);
                tracker.send_event(&new_position, 
                    EntityEvent {
                        id: id.0,
                        data: EntityEventData::MoveRotate {
                            delta: new_position - position.0,
                            yaw,
                            pitch,
                        }
                    }
                );
                tracker.send_event(&position.0, 
                    EntityEvent {
                        id: id.0,
                        data: EntityEventData::RotateHead{ yaw },
                    }
                );
                position.0 = new_position;
            },
            ServerboundPacket::PlayerDigging {
                status, position: (x, y, z), face
            } => {
                match status {
                    0 => {
                        chunks.set_block(x, y, z, Block::Air);
                    },
                    2 => {
                        chunks.set_block(x, y, z, Block::Air);
                    },
                    _ => (),
                }
            },
            ServerboundPacket::Disconnect { reason } => {
                disconnections.send(*entity, reason);
            },
        }
    }
}