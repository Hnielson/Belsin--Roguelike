use specs::prelude::*;
use super::{Map, Position, BlocksTile};

pub struct MapIndexSystem {}

impl<'a> System<'a> for MapIndexSystem {
    type SystemData = ( WriteExpect<'a, Map>,
                        ReadStorage<'a, Position>,
                        ReadStorage<'a, BlocksTile>,
                        Entities<'a>,);

    fn run(&mut self, data : Self::SystemData) {
        let (mut map, position, blockers, entities) = data;

        map.populate_blocked();
        map.clear_content_index();
        // Adds all entities with a blockedtile component to the blocked list
        for (entity, position) in (&entities, &position).join() {
            let idx = map.xy_idx(position.x, position.y);

            // If they block, update the blocking list
            let _p : Option<&BlocksTile> = blockers.get(entity);
            if let Some(_p) = _p {
                map.blocked[idx] = true;
            }

            // Push the entity to the appropriate index slot.
            // Don't clone since it's a Copy type (keep it in ecs)
            map.tile_content[idx].push(entity);
        }
    }
}