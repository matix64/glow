use legion::*;
use systems::{CommandBuffer};
use crate::buckets::{EntityTracker, EntityEvent};
use crate::entities::EntityId;
use crate::net::PlayerConnection;
use crate::entities::{Position, Rotation};
use crate::events::ClientEvent;
use super::remove_player::remove_player;
use crate::entities::Name;

#[system(for_each)]
pub fn receive_events(entity: &Entity, id: &EntityId, conn: &mut PlayerConnection, 
    name: &Name, position: &mut Position, rotation: &mut Rotation,
    cmd: &mut CommandBuffer, #[resource] tracker: &mut EntityTracker) 
{
    for event in conn.receive() {
        match event {
            ClientEvent::Disconnect(reason) => {
                println!("{} disconnected, reason: {}", name.0, reason);
                let entity = *entity;
                cmd.exec_mut(move |world, resources| {
                    remove_player(entity, world, resources);
                });
            }
            ClientEvent::Move(new_pos) => {
                tracker.move_entity(id.0, *entity, position.0, new_pos);
                position.0 = new_pos;
            }
            ClientEvent::Rotate(yaw, pitch) => {
                tracker.send_event(&position.0, 
                    EntityEvent::Rotate{ id: id.0, pitch, yaw }
                );
                *rotation = Rotation(yaw, pitch);
            }
            ClientEvent::BreakBlock(x, y, z) => {
                println!("Alguien rompio un bloquesito en {}, {}, {}", x, y, z);
            }
        }
    }
}